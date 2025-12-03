use async_trait::async_trait;
use futures::{Stream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::time::Duration;
use tokio::sync::watch;
use tracing::{debug, error, warn};

use crate::modules::chat::ports::{
    CompletionRequest, CompletionResponse, FinishReason, HealthStatus, LLMChatMessage, LLMError,
    LLMPort, LLMProviderConfig, ModelInfo, ProviderInfo, ProviderType, StreamChunk, TokenUsage,
};

/// OpenAI API 适配器
pub struct OpenAIAdapter {
    client: Client,
    config: LLMProviderConfig,
    cancel_sender: watch::Sender<bool>,
}

impl OpenAIAdapter {
    /// 创建新的 OpenAI 适配器
    pub fn new(config: LLMProviderConfig) -> Result<Self, LLMError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| LLMError::NetworkError(e.to_string()))?;

        let (cancel_sender, _) = watch::channel(false);

        Ok(Self {
            client,
            config,
            cancel_sender,
        })
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
            model: request.model.clone(),
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
impl LLMPort for OpenAIAdapter {
    fn provider_id(&self) -> &str {
        &self.config.id
    }

    fn provider_info(&self) -> ProviderInfo {
        ProviderInfo {
            id: self.config.id.clone(),
            name: self.config.name.clone(),
            provider_type: ProviderType::OpenAI,
            models: vec![
                ModelInfo {
                    id: "gpt-4o".to_string(),
                    name: "GPT-4o".to_string(),
                    context_length: 128000,
                    supports_vision: true,
                    supports_functions: true,
                },
                ModelInfo {
                    id: "gpt-4o-mini".to_string(),
                    name: "GPT-4o Mini".to_string(),
                    context_length: 128000,
                    supports_vision: true,
                    supports_functions: true,
                },
                ModelInfo {
                    id: "gpt-4-turbo".to_string(),
                    name: "GPT-4 Turbo".to_string(),
                    context_length: 128000,
                    supports_vision: true,
                    supports_functions: true,
                },
                ModelInfo {
                    id: "gpt-3.5-turbo".to_string(),
                    name: "GPT-3.5 Turbo".to_string(),
                    context_length: 16385,
                    supports_vision: false,
                    supports_functions: true,
                },
            ],
        }
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, LLMError> {
        // 返回预定义的模型列表
        // 实际应用中可以调用 /v1/models 接口
        Ok(self.provider_info().models)
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LLMError> {
        let openai_request = self.to_openai_request(&request, false);

        debug!(
            "Sending OpenAI completion request: {:?}",
            openai_request.model
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
            error!("OpenAI API error: {} - {}", status, error_text);

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
            "Sending OpenAI streaming request: {:?}",
            openai_request.model
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
            error!("OpenAI API error: {} - {}", status, error_text);

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

        let cancel_receiver = self.cancel_sender.subscribe();
        let byte_stream = response.bytes_stream();

        let stream = byte_stream
            .map(move |result| result.map_err(|e| LLMError::NetworkError(e.to_string())))
            .take_while(move |_| {
                let cancelled = *cancel_receiver.borrow();
                async move { !cancelled }
            })
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
        warn!("Cancelling OpenAI request");
        let _ = self.cancel_sender.send(true);
        Ok(())
    }

    async fn health_check(&self) -> Result<HealthStatus, LLMError> {
        let start = std::time::Instant::now();

        // 发送一个简单的请求测试连接
        let request = CompletionRequest::new(
            vec![LLMChatMessage {
                role: "user".to_string(),
                content: "Hi".to_string(),
            }],
            "gpt-3.5-turbo",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sse_line() {
        let line = r#"data: {"id":"chatcmpl-123","choices":[{"delta":{"content":"Hello"}}]}"#;
        let result = OpenAIAdapter::parse_sse_line(line);
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_sse_done() {
        let line = "data: [DONE]";
        let result = OpenAIAdapter::parse_sse_line(line);
        assert!(result.is_none());
    }
}
