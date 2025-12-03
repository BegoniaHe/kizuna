// OpenAI 兼容适配器基础实现
//
// 提供 OpenAI API 兼容的 LLM 适配器通用实现,减少代码重复

use async_trait::async_trait;
use futures::stream::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::time::Duration;
use tokio::sync::watch;
use tracing::{debug, error};

use crate::modules::chat::ports::{
    CompletionRequest, CompletionResponse, FinishReason, LLMError, LLMPort, ModelInfo,
    ProviderInfo, ProviderType, StreamChunk, TokenUsage,
};

/// OpenAI API 请求格式
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

/// OpenAI API 响应格式
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    #[allow(dead_code)]
    id: String,
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
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

/// 流式响应格式
#[derive(Debug, Deserialize)]
struct OpenAIStreamResponse {
    #[allow(dead_code)]
    id: String,
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

/// OpenAI 兼容适配器配置
pub struct OpenAICompatibleConfig {
    pub provider_id: String,
    pub provider_name: String,
    pub provider_type: ProviderType,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub timeout_secs: u64,
}

/// OpenAI 兼容适配器基础实现
pub struct BaseOpenAICompatibleAdapter {
    config: OpenAICompatibleConfig,
    client: Client,
    cancel_sender: watch::Sender<bool>,
}

impl BaseOpenAICompatibleAdapter {
    /// 创建新的适配器实例
    pub fn new(config: OpenAICompatibleConfig) -> Result<Self, LLMError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| LLMError::NetworkError(e.to_string()))?;

        let (cancel_sender, _) = watch::channel(false);

        Ok(Self {
            config,
            client,
            cancel_sender,
        })
    }

    /// 获取配置的只读引用
    pub fn config(&self) -> &OpenAICompatibleConfig {
        &self.config
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
        let messages: Vec<OpenAIMessage> = request
            .messages
            .iter()
            .map(|msg| OpenAIMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            })
            .collect();

        OpenAIRequest {
            model: self.config.model.clone(),
            messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: if stream { Some(true) } else { None },
        }
    }

    /// 解析 SSE 行
    fn parse_sse_line(line: &str) -> Option<OpenAIStreamResponse> {
        let line = line.trim();
        if line.is_empty() || !line.starts_with("data: ") {
            return None;
        }

        let data = &line[6..];
        if data == "[DONE]" {
            return None;
        }

        serde_json::from_str(data).ok()
    }

    /// 取消当前生成
    pub fn cancel(&self) {
        let _ = self.cancel_sender.send(true);
    }

    /// 执行非流式补全
    pub async fn complete_internal(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, LLMError> {
        let openai_request = self.to_openai_request(&request, false);

        debug!(
            "Sending request to {}: model={}",
            self.config.provider_name, self.config.model
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
            error!(
                "{} API error: {} - {}",
                self.config.provider_name, status, error_text
            );
            return Err(LLMError::ApiError {
                code: status.as_str().to_string(),
                message: error_text,
            });
        }

        let openai_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| LLMError::InvalidRequest(e.to_string()))?;

        if openai_response.choices.is_empty() {
            return Err(LLMError::ApiError {
                code: "empty_choices".to_string(),
                message: "No choices in response".to_string(),
            });
        }

        let choice = &openai_response.choices[0];
        let finish_reason = choice
            .finish_reason
            .as_ref()
            .and_then(|r| match r.as_str() {
                "stop" => Some(FinishReason::Stop),
                "length" => Some(FinishReason::Length),
                "content_filter" => Some(FinishReason::ContentFilter),
                _ => None,
            })
            .unwrap_or(FinishReason::Stop);

        Ok(CompletionResponse {
            content: choice.message.content.clone(),
            finish_reason,
            usage: openai_response
                .usage
                .map(|u| TokenUsage {
                    prompt_tokens: u.prompt_tokens,
                    completion_tokens: u.completion_tokens,
                    total_tokens: u.total_tokens,
                })
                .unwrap_or(TokenUsage {
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    total_tokens: 0,
                }),
        })
    }

    /// 执行流式补全
    pub async fn complete_stream_internal(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, LLMError>> + Send>>, LLMError> {
        let openai_request = self.to_openai_request(&request, true);

        debug!(
            "Sending streaming request to {}: model={}",
            self.config.provider_name, self.config.model
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
            error!(
                "{} API error: {} - {}",
                self.config.provider_name, status, error_text
            );
            return Err(LLMError::ApiError {
                code: status.as_str().to_string(),
                message: error_text,
            });
        }

        // 使用 unfold 代替 scan 避免生命周期问题
        use futures::stream::{self, StreamExt};

        let bytes_stream = response.bytes_stream();
        let buffer = String::new();
        let stream = stream::unfold(
            (bytes_stream, buffer),
            |(mut bytes_stream, mut buffer)| async move {
                loop {
                    match bytes_stream.next().await {
                        Some(Ok(bytes)) => {
                            buffer.push_str(&String::from_utf8_lossy(&bytes));

                            // 处理所有完整的行
                            while let Some(pos) = buffer.find('\n') {
                                let line = buffer[..pos].to_string();
                                buffer.drain(..=pos);

                                if let Some(sse_response) = Self::parse_sse_line(&line) {
                                    if let Some(choice) = sse_response.choices.first() {
                                        if let Some(content) = &choice.delta.content {
                                            let chunk = StreamChunk {
                                                content: content.clone(),
                                                finish_reason: choice
                                                    .finish_reason
                                                    .as_ref()
                                                    .and_then(|r| match r.as_str() {
                                                        "stop" => Some(FinishReason::Stop),
                                                        "length" => Some(FinishReason::Length),
                                                        "content_filter" => {
                                                            Some(FinishReason::ContentFilter)
                                                        }
                                                        _ => None,
                                                    }),
                                                usage: None,
                                            };
                                            return Some((Ok(chunk), (bytes_stream, buffer)));
                                        }
                                    }
                                }
                            }
                        }
                        Some(Err(e)) => {
                            return Some((
                                Err(LLMError::NetworkError(e.to_string())),
                                (bytes_stream, buffer),
                            ));
                        }
                        None => return None,
                    }
                }
            },
        );

        Ok(Box::pin(stream))
    }
}

// 为基础适配器实现 LLMPort trait
#[async_trait]
impl LLMPort for BaseOpenAICompatibleAdapter {
    fn provider_id(&self) -> &str {
        &self.config.provider_id
    }

    fn provider_info(&self) -> ProviderInfo {
        ProviderInfo {
            id: self.config.provider_id.clone(),
            name: self.config.provider_name.clone(),
            provider_type: self.config.provider_type.clone(),
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
        self.complete_internal(request).await
    }

    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, LLMError>> + Send>>, LLMError> {
        self.complete_stream_internal(request).await
    }

    async fn health_check(&self) -> Result<crate::modules::chat::ports::HealthStatus, LLMError> {
        // 简单的健康检查：尝试列出模型
        self.list_models().await?;
        Ok(crate::modules::chat::ports::HealthStatus {
            is_healthy: true,
            latency_ms: None,
            error_message: None,
        })
    }

    async fn cancel(&self, _request_id: &str) -> Result<(), LLMError> {
        // 发送取消信号
        let _ = self.cancel_sender.send(true);
        Ok(())
    }
}
