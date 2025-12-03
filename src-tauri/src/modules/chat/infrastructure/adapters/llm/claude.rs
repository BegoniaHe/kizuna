// Claude Adapter - Anthropic Claude Messages API
//
// 实现 Claude 的消息 API 适配器

use async_trait::async_trait;
use futures::stream::{self, Stream};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

use crate::modules::chat::ports::{
    CompletionRequest, CompletionResponse, FinishReason, HealthStatus, LLMChatMessage, LLMError,
    LLMPort, LLMProviderConfig, ModelInfo, ProviderInfo, ProviderType, StreamChunk, TokenUsage,
};

/// Claude API 请求
#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    messages: Vec<ClaudeMessage>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

/// Claude API 响应
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ClaudeResponse {
    id: String,
    content: Vec<ContentBlock>,
    stop_reason: Option<String>,
    usage: ClaudeUsage,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ClaudeUsage {
    input_tokens: u32,
    output_tokens: u32,
}

/// Claude 流式响应事件
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[allow(dead_code)]
enum ClaudeStreamEvent {
    #[serde(rename = "message_start")]
    MessageStart { message: MessageStart },
    #[serde(rename = "content_block_start")]
    ContentBlockStart { content_block: ContentBlock },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta { delta: Delta },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop,
    #[serde(rename = "message_delta")]
    MessageDelta {
        delta: MessageDelta,
        usage: ClaudeUsage,
    },
    #[serde(rename = "message_stop")]
    MessageStop,
    #[serde(rename = "ping")]
    Ping,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MessageStart {
    id: String,
    usage: ClaudeUsage,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Delta {
    #[serde(rename = "type")]
    delta_type: String,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MessageDelta {
    stop_reason: Option<String>,
}

/// Claude 适配器
pub struct ClaudeAdapter {
    config: LLMProviderConfig,
    client: Client,
}

impl ClaudeAdapter {
    pub fn new(config: LLMProviderConfig) -> Result<Self, LLMError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| LLMError::Unknown(e.to_string()))?;

        Ok(Self { config, client })
    }

    fn convert_messages(&self, messages: Vec<LLMChatMessage>) -> Vec<ClaudeMessage> {
        messages
            .into_iter()
            .filter(|m| m.role != "system") // Claude 不支持 system 消息在 messages 数组中
            .map(|m| ClaudeMessage {
                role: if m.role == "assistant" {
                    "assistant".to_string()
                } else {
                    "user".to_string()
                },
                content: m.content,
            })
            .collect()
    }

    fn map_finish_reason(&self, reason: Option<String>) -> FinishReason {
        match reason.as_deref() {
            Some("end_turn") => FinishReason::Stop,
            Some("max_tokens") => FinishReason::Length,
            Some("stop_sequence") => FinishReason::Stop,
            _ => FinishReason::Stop,
        }
    }
}

#[async_trait]
impl LLMPort for ClaudeAdapter {
    fn provider_id(&self) -> &str {
        &self.config.id
    }

    fn provider_info(&self) -> ProviderInfo {
        ProviderInfo {
            id: self.config.id.clone(),
            name: self.config.name.clone(),
            provider_type: ProviderType::Claude,
            models: vec![
                ModelInfo {
                    id: "claude-3-5-sonnet-20241022".to_string(),
                    name: "Claude 3.5 Sonnet".to_string(),
                    context_length: 200000,
                    supports_vision: true,
                    supports_functions: true,
                },
                ModelInfo {
                    id: "claude-3-opus-20240229".to_string(),
                    name: "Claude 3 Opus".to_string(),
                    context_length: 200000,
                    supports_vision: true,
                    supports_functions: true,
                },
                ModelInfo {
                    id: "claude-3-sonnet-20240229".to_string(),
                    name: "Claude 3 Sonnet".to_string(),
                    context_length: 200000,
                    supports_vision: true,
                    supports_functions: true,
                },
            ],
        }
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, LLMError> {
        Ok(self.provider_info().models)
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LLMError> {
        let claude_request = ClaudeRequest {
            model: request.model.clone(),
            messages: self.convert_messages(request.messages),
            max_tokens: request.max_tokens.unwrap_or(4096),
            temperature: request.temperature,
            stop_sequences: request.stop_sequences,
            stream: false,
        };

        let response = self
            .client
            .post(format!("{}/messages", self.config.base_url))
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&claude_request)
            .send()
            .await
            .map_err(|e| LLMError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(LLMError::ApiError {
                code: status.to_string(),
                message: error_text,
            });
        }

        let claude_response: ClaudeResponse = response
            .json()
            .await
            .map_err(|e| LLMError::Unknown(e.to_string()))?;

        let content = claude_response
            .content
            .iter()
            .filter_map(|block| block.text.clone())
            .collect::<Vec<_>>()
            .join("");

        Ok(CompletionResponse {
            content,
            finish_reason: self.map_finish_reason(claude_response.stop_reason),
            usage: TokenUsage {
                prompt_tokens: claude_response.usage.input_tokens,
                completion_tokens: claude_response.usage.output_tokens,
                total_tokens: claude_response.usage.input_tokens
                    + claude_response.usage.output_tokens,
            },
        })
    }

    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, LLMError>> + Send>>, LLMError> {
        let claude_request = ClaudeRequest {
            model: request.model.clone(),
            messages: self.convert_messages(request.messages),
            max_tokens: request.max_tokens.unwrap_or(4096),
            temperature: request.temperature,
            stop_sequences: request.stop_sequences,
            stream: true,
        };

        let response = self
            .client
            .post(format!("{}/messages", self.config.base_url))
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&claude_request)
            .send()
            .await
            .map_err(|e| LLMError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(LLMError::ApiError {
                code: status.to_string(),
                message: error_text,
            });
        }

        use futures::StreamExt;

        let bytes_stream = response.bytes_stream();
        let buffer = String::new();

        let stream = stream::unfold(
            (bytes_stream, buffer),
            |(mut bytes_stream, mut buffer)| async move {
                loop {
                    match bytes_stream.next().await {
                        Some(Ok(bytes)) => {
                            buffer.push_str(&String::from_utf8_lossy(&bytes));

                            // 处理所有完整的 SSE 事件
                            while let Some(pos) = buffer.find("\n\n") {
                                let block = buffer[..pos].to_string();
                                buffer.drain(..pos + 2);

                                // 查找 data: 行
                                for line in block.lines() {
                                    if let Some(json_str) = line.strip_prefix("data: ") {
                                        if let Ok(event) =
                                            serde_json::from_str::<ClaudeStreamEvent>(json_str)
                                        {
                                            match event {
                                                ClaudeStreamEvent::ContentBlockDelta { delta } => {
                                                    if let Some(text) = delta.text {
                                                        let chunk = StreamChunk {
                                                            content: text,
                                                            finish_reason: None,
                                                            usage: None,
                                                        };
                                                        return Some((
                                                            Ok(chunk),
                                                            (bytes_stream, buffer),
                                                        ));
                                                    }
                                                }
                                                ClaudeStreamEvent::MessageDelta {
                                                    delta,
                                                    usage,
                                                } => {
                                                    let finish = delta.stop_reason.map(|r| {
                                                        if r == "end_turn" {
                                                            FinishReason::Stop
                                                        } else if r == "max_tokens" {
                                                            FinishReason::Length
                                                        } else {
                                                            FinishReason::Stop
                                                        }
                                                    });
                                                    let chunk = StreamChunk {
                                                        content: String::new(),
                                                        finish_reason: finish,
                                                        usage: Some(TokenUsage {
                                                            prompt_tokens: usage.input_tokens,
                                                            completion_tokens: usage.output_tokens,
                                                            total_tokens: usage.input_tokens
                                                                + usage.output_tokens,
                                                        }),
                                                    };
                                                    return Some((
                                                        Ok(chunk),
                                                        (bytes_stream, buffer),
                                                    ));
                                                }
                                                _ => {}
                                            }
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

    async fn cancel(&self, _request_id: &str) -> Result<(), LLMError> {
        // Claude API 不支持取消请求
        Ok(())
    }

    async fn health_check(&self) -> Result<HealthStatus, LLMError> {
        let start = std::time::Instant::now();

        // 简单的健康检查 - 发送最小请求
        let test_request = ClaudeRequest {
            model: self.config.default_model.clone(),
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: "Hi".to_string(),
            }],
            max_tokens: 1,
            temperature: None,
            stop_sequences: None,
            stream: false,
        };

        match self
            .client
            .post(format!("{}/messages", self.config.base_url))
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&test_request)
            .send()
            .await
        {
            Ok(response) => {
                let latency = start.elapsed().as_millis() as u64;
                if response.status().is_success() {
                    Ok(HealthStatus {
                        is_healthy: true,
                        latency_ms: Some(latency),
                        error_message: None,
                    })
                } else {
                    Ok(HealthStatus {
                        is_healthy: false,
                        latency_ms: Some(latency),
                        error_message: Some(format!("API returned {}", response.status())),
                    })
                }
            }
            Err(e) => Ok(HealthStatus {
                is_healthy: false,
                latency_ms: None,
                error_message: Some(e.to_string()),
            }),
        }
    }
}
