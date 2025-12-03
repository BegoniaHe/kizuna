// Chat Commands - 完全重构版本
//
// 聊天相关的 Tauri 命令处理器
// 完全通过 ChatModule 的六边形架构处理业务逻辑

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::infrastructure::{AppEvent, EventBus};
use crate::modules::chat::infrastructure::LLMAdapterRegistry;
use crate::modules::chat::ports::{LLMProviderConfig, ProviderType};
use crate::modules::chat::{ChatModule, MessageId, MessageRole, SendMessageCommand, SessionId};
use crate::shared::{AppResult, Emotion, Message, MessageChunk, MessageRole as SharedMessageRole, text_to_phonemes};

/// 前端 Provider 配置
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendProviderConfig {
    pub id: String,
    pub name: String,
    pub provider_type: ProviderType,
    pub base_url: String,
    pub api_key: String,
    pub models: Vec<String>,
    #[serde(default)]
    pub is_default: bool,
}

impl From<FrontendProviderConfig> for LLMProviderConfig {
    fn from(config: FrontendProviderConfig) -> Self {
        tracing::debug!(
            "[FrontendProviderConfig->LLMProviderConfig] Converting: id={}, base_url={}, provider_type={:?}",
            config.id,
            config.base_url,
            config.provider_type
        );
        LLMProviderConfig {
            id: config.id,
            name: config.name,
            provider_type: config.provider_type,
            base_url: config.base_url,
            api_key: config.api_key,
            default_model: config
                .models
                .first()
                .cloned()
                .unwrap_or_else(|| "gpt-3.5-turbo".to_string()),
            timeout_secs: 60,
            max_retries: 3,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageRequest {
    pub session_id: Uuid,
    pub content: String,
    pub provider_config: Option<FrontendProviderConfig>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageResponse {
    pub message_id: Uuid,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StopGenerationRequest {
    pub session_id: Uuid,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegenerateRequest {
    pub session_id: Uuid,
    pub user_content: String,
    pub provider_config: Option<FrontendProviderConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetMessagesRequest {
    pub session_id: Uuid,
    pub page: u32,
    pub limit: u32,
}

/// 发送消息命令 - 使用 ChatModule 的六边形架构
#[tauri::command]
pub async fn chat_send_message(
    chat_module: State<'_, Arc<RwLock<ChatModule>>>,
    event_bus: State<'_, Arc<RwLock<EventBus>>>,
    llm_registry: State<'_, Arc<LLMAdapterRegistry>>,
    request: SendMessageRequest,
) -> AppResult<SendMessageResponse> {
    tracing::info!(
        "[chat_send_message] Processing for session: {}",
        request.session_id
    );

    let session_id_domain = SessionId::from(request.session_id);
    let content = request.content.clone();
    let provider_config = request.provider_config.clone();

    // 克隆资源用于异步任务
    let event_bus_clone = event_bus.inner().clone();
    let chat_module_clone = chat_module.inner().clone();
    let llm_registry_clone = llm_registry.inner().clone();
    let request_session_id = request.session_id;

    // 在后台任务中处理 LLM 响应(使用 ChatModule)
    tokio::spawn(async move {
        let result = process_message_with_module(
            session_id_domain,
            content,
            provider_config,
            chat_module_clone.clone(),
            event_bus_clone.clone(),
            llm_registry_clone,
        )
        .await;

        let event_bus = event_bus_clone.read().await;

        match result {
            Ok((message_id, emotion)) => {
                tracing::info!("[chat_send_message] Message processed: {}", message_id);
                event_bus.publish(AppEvent::MessageComplete {
                    session_id: request_session_id,
                    message_id: message_id.into(),
                    emotion,
                });
            }
            Err(error) => {
                tracing::error!("[chat_send_message] Error: {}", error);
                event_bus.publish(AppEvent::MessageError {
                    session_id: request_session_id,
                    error,
                });
            }
        }
    });

    Ok(SendMessageResponse {
        message_id: Uuid::new_v4(),
    })
}

/// 使用 ChatModule 处理消息和 LLM 调用
async fn process_message_with_module(
    session_id: SessionId,
    content: String,
    provider_config: Option<FrontendProviderConfig>,
    chat_module: Arc<RwLock<ChatModule>>,
    event_bus: Arc<RwLock<EventBus>>,
    llm_registry: Arc<LLMAdapterRegistry>,
) -> Result<(MessageId, Option<Emotion>), String> {
    // 从配置创建 LLM 适配器
    let provider_config = provider_config.ok_or("No provider configuration provided")?;
    let provider_id = provider_config.id.clone();
    let llm_provider_config: LLMProviderConfig = provider_config.into();

    let _llm = llm_registry
        .get_or_create(&llm_provider_config)
        .await
        .map_err(|e| format!("Failed to create LLM adapter: {}", e))?;

    // 使用 ChatModule 的 SendMessageCommand (流式)
    let command = SendMessageCommand::new(session_id, content.clone(), None, true);

    let module = chat_module.read().await;

    // 调用流式处理
    let (response, mut rx) = module
        .send_message_stream(command, &provider_id)
        .await
        .map_err(|e| e.to_string())?;

    let assistant_message_id = response.assistant_message.id();
    drop(module); // 释放锁

    // 处理流式事件
    let event_bus_read = event_bus.read().await;
    while let Some(event) = rx.recv().await {
        match event {
            crate::modules::chat::StreamEvent::Chunk(chunk) => {
                // 将文本转换为口型音素序列
                let phonemes = text_to_phonemes(&chunk);
                
                event_bus_read.publish(AppEvent::MessageChunk(MessageChunk {
                    session_id: session_id.into(),
                    content: chunk,
                    tokens: None,
                    phonemes: Some(phonemes),
                }));
            }
            crate::modules::chat::StreamEvent::Done {
                full_content,
                tokens_used: _,
            } => {
                // 分析情感
                let emotion = analyze_emotion(&full_content);
                return Ok((assistant_message_id, emotion));
            }
            crate::modules::chat::StreamEvent::Error(err) => {
                return Err(err);
            }
        }
    }

    Ok((assistant_message_id, None))
}

/// 简单的情感分析
fn analyze_emotion(content: &str) -> Option<Emotion> {
    let lower = content.to_lowercase();

    if lower.contains("happy") || lower.contains("joy") || lower.contains("😊") {
        Some(Emotion::Happy)
    } else if lower.contains("sad") || lower.contains("sorry") {
        Some(Emotion::Sad)
    } else if lower.contains("angry") || lower.contains("mad") {
        Some(Emotion::Angry)
    } else {
        Some(Emotion::Neutral)
    }
}

/// 停止生成
#[tauri::command]
pub async fn chat_stop_generation(_request: StopGenerationRequest) -> AppResult<()> {
    // TODO: 在 ChatModule 中实现取消机制
    tracing::warn!("[chat_stop_generation] Not yet implemented");
    Ok(())
}

/// 重新生成消息（不创建新的用户消息）
#[tauri::command]
pub async fn chat_regenerate(
    chat_module: State<'_, Arc<RwLock<ChatModule>>>,
    event_bus: State<'_, Arc<RwLock<EventBus>>>,
    llm_registry: State<'_, Arc<LLMAdapterRegistry>>,
    request: RegenerateRequest,
) -> AppResult<SendMessageResponse> {
    tracing::info!(
        "[chat_regenerate] Regenerating for session: {}",
        request.session_id
    );

    let session_id_domain = SessionId::from(request.session_id);
    let content = request.user_content.clone();
    let provider_config = request.provider_config.clone();

    let event_bus_clone = event_bus.inner().clone();
    let chat_module_clone = chat_module.inner().clone();
    let llm_registry_clone = llm_registry.inner().clone();
    let request_session_id = request.session_id;

    tokio::spawn(async move {
        let result = process_regenerate_with_module(
            session_id_domain,
            content,
            provider_config,
            chat_module_clone.clone(),
            event_bus_clone.clone(),
            llm_registry_clone,
        )
        .await;

        let event_bus = event_bus_clone.read().await;

        match result {
            Ok((message_id, emotion)) => {
                tracing::info!("[chat_regenerate] Message regenerated: {}", message_id);
                event_bus.publish(AppEvent::MessageComplete {
                    session_id: request_session_id,
                    message_id: message_id.into(),
                    emotion,
                });
            }
            Err(error) => {
                tracing::error!("[chat_regenerate] Error: {}", error);
                event_bus.publish(AppEvent::MessageError {
                    session_id: request_session_id,
                    error,
                });
            }
        }
    });

    Ok(SendMessageResponse {
        message_id: Uuid::new_v4(),
    })
}

/// 使用 ChatModule 处理重新生成（不保存用户消息）
async fn process_regenerate_with_module(
    session_id: SessionId,
    user_content: String,
    provider_config: Option<FrontendProviderConfig>,
    chat_module: Arc<RwLock<ChatModule>>,
    event_bus: Arc<RwLock<EventBus>>,
    llm_registry: Arc<LLMAdapterRegistry>,
) -> Result<(MessageId, Option<Emotion>), String> {
    let provider_config = provider_config.ok_or("No provider configuration provided")?;
    let provider_id = provider_config.id.clone();
    let llm_provider_config: LLMProviderConfig = provider_config.into();

    let _llm = llm_registry
        .get_or_create(&llm_provider_config)
        .await
        .map_err(|e| format!("Failed to create LLM adapter: {}", e))?;

    // 使用 regenerate 命令（不保存用户消息）
    let command = crate::modules::chat::RegenerateCommand::new(session_id, user_content, None, true);

    let module = chat_module.read().await;

    let (response, mut rx) = module
        .regenerate_stream(command, &provider_id)
        .await
        .map_err(|e| e.to_string())?;

    let assistant_message_id = response.assistant_message.id();
    drop(module);

    let event_bus_read = event_bus.read().await;
    while let Some(event) = rx.recv().await {
        match event {
            crate::modules::chat::StreamEvent::Chunk(chunk) => {
                let phonemes = text_to_phonemes(&chunk);
                
                event_bus_read.publish(AppEvent::MessageChunk(MessageChunk {
                    session_id: session_id.into(),
                    content: chunk,
                    tokens: None,
                    phonemes: Some(phonemes),
                }));
            }
            crate::modules::chat::StreamEvent::Done {
                full_content,
                tokens_used: _,
            } => {
                let emotion = analyze_emotion(&full_content);
                return Ok((assistant_message_id, emotion));
            }
            crate::modules::chat::StreamEvent::Error(e) => {
                return Err(e);
            }
        }
    }

    Err("Stream ended unexpectedly".to_string())
}

/// 获取消息列表 - 使用 ChatModule 的 Query
#[tauri::command]
pub async fn chat_get_messages(
    chat_module: State<'_, Arc<RwLock<ChatModule>>>,
    request: GetMessagesRequest,
) -> AppResult<Vec<Message>> {
    let session_id = SessionId::from(request.session_id);

    let module = chat_module.read().await;
    let query =
        crate::modules::chat::ListMessagesQuery::new(session_id, request.page, request.limit);

    let response = module
        .list_messages(query)
        .await
        .map_err(|e| crate::shared::AppError::Unknown(e.to_string()))?;

    // 转换 domain Message 到 shared Message
    let messages: Vec<Message> = response
        .messages
        .into_iter()
        .map(|msg| Message {
            id: msg.id().into(),
            session_id: request.session_id,
            role: match msg.role() {
                MessageRole::User => SharedMessageRole::User,
                MessageRole::Assistant => SharedMessageRole::Assistant,
                _ => SharedMessageRole::System,
            },
            content: msg.content().to_string(),
            tokens: None,
            emotion: msg.emotion().map(|e| match e {
                crate::modules::chat::domain::Emotion::Neutral => Emotion::Neutral,
                crate::modules::chat::domain::Emotion::Happy => Emotion::Happy,
                crate::modules::chat::domain::Emotion::Sad => Emotion::Sad,
                crate::modules::chat::domain::Emotion::Angry => Emotion::Angry,
                crate::modules::chat::domain::Emotion::Surprised => Emotion::Surprised,
                crate::modules::chat::domain::Emotion::Thinking => Emotion::Thinking,
            }),
            created_at: msg.created_at(),
        })
        .collect();

    Ok(messages)
}

/// 获取模型列表请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchModelsRequest {
    pub provider_config: FrontendProviderConfig,
}

/// 模型信息响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfoResponse {
    pub id: String,
    pub name: String,
    pub owned_by: Option<String>,
}

/// 获取 API 提供商的模型列表
#[tauri::command]
pub async fn chat_fetch_models(
    request: FetchModelsRequest,
) -> AppResult<Vec<ModelInfoResponse>> {
    tracing::info!(
        "[chat_fetch_models] Fetching models for provider: {} ({:?})",
        request.provider_config.name,
        request.provider_config.provider_type
    );

    let config = &request.provider_config;
    let base_url = config.base_url.trim_end_matches('/');
    
    // 构建请求 URL
    let url = match config.provider_type {
        ProviderType::OpenAI | ProviderType::Custom => format!("{}/models", base_url),
        ProviderType::Claude => {
            // Claude 不支持列出模型，返回预定义列表
            return Ok(vec![
                ModelInfoResponse { id: "claude-sonnet-4-20250514".to_string(), name: "Claude Sonnet 4".to_string(), owned_by: Some("anthropic".to_string()) },
                ModelInfoResponse { id: "claude-3-7-sonnet-20250219".to_string(), name: "Claude 3.7 Sonnet".to_string(), owned_by: Some("anthropic".to_string()) },
                ModelInfoResponse { id: "claude-3-5-sonnet-20241022".to_string(), name: "Claude 3.5 Sonnet".to_string(), owned_by: Some("anthropic".to_string()) },
                ModelInfoResponse { id: "claude-3-5-haiku-20241022".to_string(), name: "Claude 3.5 Haiku".to_string(), owned_by: Some("anthropic".to_string()) },
                ModelInfoResponse { id: "claude-3-opus-20240229".to_string(), name: "Claude 3 Opus".to_string(), owned_by: Some("anthropic".to_string()) },
            ]);
        }
        ProviderType::Ollama => format!("{}/api/tags", base_url),
    };

    tracing::debug!("[chat_fetch_models] Requesting: {}", url);

    let client = reqwest::Client::new();
    
    let response = match config.provider_type {
        ProviderType::Ollama => {
            client.get(&url).send().await
        }
        _ => {
            client
                .get(&url)
                .header("Authorization", format!("Bearer {}", config.api_key))
                .send()
                .await
        }
    };

    let response = response.map_err(|e| {
        tracing::error!("[chat_fetch_models] Request failed: {}", e);
        crate::shared::AppError::Unknown(format!("Failed to fetch models: {}", e))
    })?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        tracing::error!("[chat_fetch_models] API error: {} - {}", status, error_text);
        return Err(crate::shared::AppError::Unknown(format!(
            "API error: {} - {}",
            status, error_text
        )));
    }

    // 解析响应
    let models: Vec<ModelInfoResponse> = match config.provider_type {
        ProviderType::Ollama => {
            #[derive(Deserialize)]
            struct OllamaResponse {
                models: Vec<OllamaModel>,
            }
            #[derive(Deserialize)]
            struct OllamaModel {
                name: String,
            }
            let resp: OllamaResponse = response.json().await.map_err(|e| {
                crate::shared::AppError::Unknown(format!("Failed to parse response: {}", e))
            })?;
            resp.models
                .into_iter()
                .map(|m| ModelInfoResponse {
                    id: m.name.clone(),
                    name: m.name,
                    owned_by: Some("ollama".to_string()),
                })
                .collect()
        }
        _ => {
            // OpenAI 兼容格式
            #[derive(Deserialize)]
            struct OpenAIModelsResponse {
                data: Vec<OpenAIModel>,
            }
            #[derive(Deserialize)]
            struct OpenAIModel {
                id: String,
                owned_by: Option<String>,
            }
            let resp: OpenAIModelsResponse = response.json().await.map_err(|e| {
                crate::shared::AppError::Unknown(format!("Failed to parse response: {}", e))
            })?;
            resp.data
                .into_iter()
                .map(|m| ModelInfoResponse {
                    id: m.id.clone(),
                    name: m.id,
                    owned_by: m.owned_by,
                })
                .collect()
        }
    };

    tracing::info!("[chat_fetch_models] Found {} models", models.len());
    Ok(models)
}
