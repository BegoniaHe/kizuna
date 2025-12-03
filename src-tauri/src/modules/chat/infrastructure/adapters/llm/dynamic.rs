// Dynamic LLM Adapter
//
// 用于处理从前端传入的动态 LLM 配置
// 这个适配器在每次请求时根据配置创建临时的 OpenAI 兼容客户端

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::time::Duration;
use tracing::{debug, error};

use crate::modules::chat::ports::{
    CompletionRequest, CompletionResponse, FinishReason, HealthStatus, LLMChatMessage, LLMError,
    LLMPort, ModelInfo, ProviderInfo, ProviderType, StreamChunk, TokenUsage,
};

/// 动态 LLM 配置 (从前端传入)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicLLMConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    #[serde(default = "default_stream")]
    pub stream: bool,
}

fn default_stream() -> bool {
    true
}

/// 动态 LLM 适配器
///
/// 与 OpenAIAdapter 不同，这个适配器可以在运行时接受不同的配置
pub struct DynamicLLMAdapter {
    config: DynamicLLMConfig,
    client: Client,
}

impl DynamicLLMAdapter {
    /// 创建新的动态 LLM 适配器
    pub fn new(config: DynamicLLMConfig) -> Result<Self, LLMError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| LLMError::NetworkError(e.to_string()))?;

        Ok(Self { config, client })
    }

    /// 获取 API URL
    fn api_url(&self, endpoint: &str) -> String {
        format!(
            "{}/{}",
            self.config.base_url.trim_end_matches('/'),
            endpoint
        )
    }

    /// 转换为 OpenAI 请求格式
    fn to_openai_request(&self, request: &CompletionRequest, stream: bool) -> OpenAIRequest {
        OpenAIRequest {
            model: self.config.model.clone(),
            messages: request
                .messages
                .iter()
                .map(|m| OpenAIMessage {
                    role: m.role.clone(),
                    content: m.content.clone(),
                })
                .collect(),
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            stop: request.stop_sequences.clone(),
            stream: Some(stream),
        }
    }

    /// 解析 SSE 行
    fn parse_sse_line(line: &str) -> Option<OpenAIStreamResponse> {
        if line.starts_with("data: ") {
            let data = &line[6..];
            if data == "[DONE]" {
                return None;
            }
            serde_json::from_str(data).ok()
        } else {
            None
        }
    }
}

#[async_trait]
impl LLMPort for DynamicLLMAdapter {
    fn provider_id(&self) -> &str {
        "dynamic"
    }

    fn provider_info(&self) -> ProviderInfo {
        ProviderInfo {
            id: "dynamic".to_string(),
            name: "Dynamic Provider".to_string(),
            provider_type: ProviderType::Custom,
            models: vec![ModelInfo {
                id: self.config.model.clone(),
                name: self.config.model.clone(),
                context_length: 128000,
                supports_vision: false,
                supports_functions: true,
            }],
        }
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, LLMError> {
        Ok(vec![ModelInfo {
            id: self.config.model.clone(),
            name: self.config.model.clone(),
            context_length: 128000,
            supports_vision: false,
            supports_functions: true,
        }])
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LLMError> {
        let openai_request = self.to_openai_request(&request, false);

        debug!(
            "Sending dynamic LLM request to {}: model={}",
            self.config.base_url, self.config.model
        );

        let response = self
            .client
            .post(self.api_url("chat/completions"))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_request)
            .send()
            .await
            .map_err(|e| LLMError::NetworkError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Dynamic LLM API error: {} - {}", status, error_text);

            if status.as_u16() == 429 {
                return Err(LLMError::RateLimitError {
                    retry_after_secs: 60,
                });
            }
            if status.as_u16() == 401 {
                return Err(LLMError::AuthenticationError("Invalid API key".to_string()));
            }

            return Err(LLMError::ApiError {
                code: status.to_string(),
                message: error_text,
            });
        }

        let openai_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| LLMError::Unknown(e.to_string()))?;

        let choice = openai_response
            .choices
            .first()
            .ok_or_else(|| LLMError::Unknown("No choices in response".to_string()))?;

        Ok(CompletionResponse {
            content: choice.message.content.clone(),
            finish_reason: match choice.finish_reason.as_deref() {
                Some("stop") => FinishReason::Stop,
                Some("length") => FinishReason::Length,
                Some("content_filter") => FinishReason::ContentFilter,
                Some("function_call") | Some("tool_calls") => FinishReason::FunctionCall,
                _ => FinishReason::Stop,
            },
            usage: TokenUsage {
                prompt_tokens: openai_response.usage.prompt_tokens,
                completion_tokens: openai_response.usage.completion_tokens,
                total_tokens: openai_response.usage.total_tokens,
            },
        })
    }

    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, LLMError>> + Send>>, LLMError> {
        let openai_request = self.to_openai_request(&request, true);

        debug!(
            "Sending dynamic LLM streaming request to {}: model={}",
            self.config.base_url, self.config.model
        );

        let response = self
            .client
            .post(self.api_url("chat/completions"))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_request)
            .send()
            .await
            .map_err(|e| LLMError::NetworkError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Dynamic LLM API error: {} - {}", status, error_text);

            if status.as_u16() == 429 {
                return Err(LLMError::RateLimitError {
                    retry_after_secs: 60,
                });
            }

            return Err(LLMError::ApiError {
                code: status.to_string(),
                message: error_text,
            });
        }

        let byte_stream = response.bytes_stream();

        let stream = byte_stream
            .map(move |result| result.map_err(|e| LLMError::NetworkError(e.to_string())))
            .flat_map(|result| {
                futures::stream::iter(match result {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        let chunks: Vec<Result<StreamChunk, LLMError>> = text
                            .lines()
                            .filter_map(Self::parse_sse_line)
                            .filter_map(|response| {
                                response.choices.first().and_then(|choice| {
                                    choice.delta.content.as_ref().map(|content| {
                                        Ok(StreamChunk {
                                            content: content.clone(),
                                            finish_reason: choice.finish_reason.as_deref().map(
                                                |r| match r {
                                                    "stop" => FinishReason::Stop,
                                                    "length" => FinishReason::Length,
                                                    _ => FinishReason::Stop,
                                                },
                                            ),
                                            usage: None,
                                        })
                                    })
                                })
                            })
                            .collect();
                        chunks
                    }
                    Err(e) => vec![Err(e)],
                })
            });

        Ok(Box::pin(stream))
    }

    async fn cancel(&self, _request_id: &str) -> Result<(), LLMError> {
        // 动态适配器不支持取消
        Ok(())
    }

    async fn health_check(&self) -> Result<HealthStatus, LLMError> {
        let start = std::time::Instant::now();

        let request = CompletionRequest::new(
            vec![LLMChatMessage {
                role: "user".to_string(),
                content: "Hi".to_string(),
            }],
            &self.config.model,
        )
        .with_max_tokens(1);

        match self.complete(request).await {
            Ok(_) => Ok(HealthStatus {
                is_healthy: true,
                latency_ms: Some(start.elapsed().as_millis() as u64),
                error_message: None,
            }),
            Err(e) => Ok(HealthStatus {
                is_healthy: false,
                latency_ms: Some(start.elapsed().as_millis() as u64),
                error_message: Some(e.to_string()),
            }),
        }
    }
}

// OpenAI API 类型定义

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamResponse {
    choices: Vec<OpenAIStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamChoice {
    delta: OpenAIDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIDelta {
    content: Option<String>,
}

/// 模拟 LLM 适配器
///
/// 用于测试或未配置真实 LLM 时的回退
pub struct MockLLMAdapter;

impl MockLLMAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockLLMAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LLMPort for MockLLMAdapter {
    fn provider_id(&self) -> &str {
        "mock"
    }

    fn provider_info(&self) -> ProviderInfo {
        ProviderInfo {
            id: "mock".to_string(),
            name: "Mock Provider (Simulation)".to_string(),
            provider_type: ProviderType::Custom,
            models: vec![ModelInfo {
                id: "mock-model".to_string(),
                name: "Mock Model".to_string(),
                context_length: 4096,
                supports_vision: false,
                supports_functions: false,
            }],
        }
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, LLMError> {
        Ok(self.provider_info().models)
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LLMError> {
        let user_content = request
            .messages
            .last()
            .map(|m| m.content.as_str())
            .unwrap_or("");

        let response_content = format!(
            "你好！我收到了你的消息：「{}」\n\n这是一个模拟的回复。要使用真正的 LLM，请在设置中配置 API Key。",
            user_content
        );

        Ok(CompletionResponse {
            content: response_content,
            finish_reason: FinishReason::Stop,
            usage: TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 50,
                total_tokens: 60,
            },
        })
    }

    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, LLMError>> + Send>>, LLMError> {
        let user_content = request
            .messages
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_default();

        let response_content = format!(
            "你好！我收到了你的消息：「{}」\n\n这是一个模拟的回复。要使用真正的 LLM，请在设置中配置 API Key。",
            user_content
        );

        // 将响应分成多个块模拟流式输出
        let chunks: Vec<String> = response_content
            .chars()
            .collect::<Vec<_>>()
            .chunks(5)
            .map(|c| c.iter().collect::<String>())
            .collect();

        let stream = futures::stream::iter(chunks.into_iter().enumerate().map(
            |(i, content)| -> Result<StreamChunk, LLMError> {
                Ok(StreamChunk {
                    content,
                    finish_reason: if i == 0 { None } else { None },
                    usage: None,
                })
            },
        ));

        Ok(Box::pin(stream))
    }

    async fn cancel(&self, _request_id: &str) -> Result<(), LLMError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<HealthStatus, LLMError> {
        Ok(HealthStatus {
            is_healthy: true,
            latency_ms: Some(1),
            error_message: None,
        })
    }
}
