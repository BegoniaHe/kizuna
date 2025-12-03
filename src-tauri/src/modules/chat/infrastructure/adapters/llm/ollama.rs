// Ollama Adapter - Ollama Local LLM API
//
// 实现 Ollama 的聊天 API 适配器

use async_trait::async_trait;
use futures::stream::{self, Stream};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

use crate::modules::chat::ports::{
    CompletionRequest, CompletionResponse, FinishReason, HealthStatus, LLMChatMessage, LLMError,
    LLMPort, LLMProviderConfig, ModelInfo, ProviderInfo, ProviderType, StreamChunk, TokenUsage,
};

/// Ollama 聊天请求
#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

/// Ollama 聊天响应
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OllamaChatResponse {
    message: OllamaMessage,
    done: bool,
    #[serde(default)]
    total_duration: Option<u64>,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
    #[serde(default)]
    eval_count: Option<u32>,
}

/// Ollama 模型列表响应
#[derive(Debug, Deserialize)]
struct OllamaModelsResponse {
    models: Vec<OllamaModelInfo>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OllamaModelInfo {
    name: String,
    #[serde(default)]
    size: u64,
}

/// Ollama 适配器
pub struct OllamaAdapter {
    config: LLMProviderConfig,
    client: Client,
}

impl OllamaAdapter {
    pub fn new(config: LLMProviderConfig) -> Result<Self, LLMError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| LLMError::Unknown(e.to_string()))?;

        Ok(Self { config, client })
    }

    fn convert_messages(&self, messages: Vec<LLMChatMessage>) -> Vec<OllamaMessage> {
        messages
            .into_iter()
            .map(|m| OllamaMessage {
                role: m.role,
                content: m.content,
            })
            .collect()
    }
}

#[async_trait]
impl LLMPort for OllamaAdapter {
    fn provider_id(&self) -> &str {
        &self.config.id
    }

    fn provider_info(&self) -> ProviderInfo {
        ProviderInfo {
            id: self.config.id.clone(),
            name: self.config.name.clone(),
            provider_type: ProviderType::Ollama,
            models: vec![
                ModelInfo {
                    id: "llama3.2".to_string(),
                    name: "Llama 3.2".to_string(),
                    context_length: 128000,
                    supports_vision: false,
                    supports_functions: false,
                },
                ModelInfo {
                    id: "qwen2.5".to_string(),
                    name: "Qwen 2.5".to_string(),
                    context_length: 32768,
                    supports_vision: false,
                    supports_functions: false,
                },
                ModelInfo {
                    id: "mistral".to_string(),
                    name: "Mistral".to_string(),
                    context_length: 32768,
                    supports_vision: false,
                    supports_functions: false,
                },
            ],
        }
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, LLMError> {
        // 调用 Ollama API 获取本地模型列表
        let response = self
            .client
            .get(format!("{}/api/tags", self.config.base_url))
            .send()
            .await
            .map_err(|e| LLMError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Ok(self.provider_info().models); // 失败时返回默认列表
        }

        let models_response: OllamaModelsResponse = response
            .json()
            .await
            .map_err(|e| LLMError::Unknown(e.to_string()))?;

        Ok(models_response
            .models
            .into_iter()
            .map(|m| ModelInfo {
                id: m.name.clone(),
                name: m.name,
                context_length: 32768, // Ollama 默认上下文长度
                supports_vision: false,
                supports_functions: false,
            })
            .collect())
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LLMError> {
        let options = if request.temperature.is_some()
            || request.max_tokens.is_some()
            || request.stop_sequences.is_some()
        {
            Some(OllamaOptions {
                temperature: request.temperature,
                num_predict: request.max_tokens,
                stop: request.stop_sequences,
            })
        } else {
            None
        };

        let ollama_request = OllamaChatRequest {
            model: request.model.clone(),
            messages: self.convert_messages(request.messages),
            stream: false,
            options,
        };

        let response = self
            .client
            .post(format!("{}/api/chat", self.config.base_url))
            .json(&ollama_request)
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

        let ollama_response: OllamaChatResponse = response
            .json()
            .await
            .map_err(|e| LLMError::Unknown(e.to_string()))?;

        Ok(CompletionResponse {
            content: ollama_response.message.content,
            finish_reason: FinishReason::Stop,
            usage: TokenUsage {
                prompt_tokens: ollama_response.prompt_eval_count.unwrap_or(0),
                completion_tokens: ollama_response.eval_count.unwrap_or(0),
                total_tokens: ollama_response.prompt_eval_count.unwrap_or(0)
                    + ollama_response.eval_count.unwrap_or(0),
            },
        })
    }

    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, LLMError>> + Send>>, LLMError> {
        let options = if request.temperature.is_some()
            || request.max_tokens.is_some()
            || request.stop_sequences.is_some()
        {
            Some(OllamaOptions {
                temperature: request.temperature,
                num_predict: request.max_tokens,
                stop: request.stop_sequences,
            })
        } else {
            None
        };

        let ollama_request = OllamaChatRequest {
            model: request.model.clone(),
            messages: self.convert_messages(request.messages),
            stream: true,
            options,
        };

        let response = self
            .client
            .post(format!("{}/api/chat", self.config.base_url))
            .json(&ollama_request)
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

                            // 处理所有完整的 JSON 行
                            while let Some(pos) = buffer.find('\n') {
                                let line = buffer[..pos].to_string();
                                buffer.drain(..=pos);

                                if !line.is_empty() {
                                    if let Ok(response) =
                                        serde_json::from_str::<OllamaChatResponse>(&line)
                                    {
                                        if response.done {
                                            // 最后一个块包含统计信息
                                            let chunk = StreamChunk {
                                                content: String::new(),
                                                finish_reason: Some(FinishReason::Stop),
                                                usage: Some(TokenUsage {
                                                    prompt_tokens: response
                                                        .prompt_eval_count
                                                        .unwrap_or(0),
                                                    completion_tokens: response
                                                        .eval_count
                                                        .unwrap_or(0),
                                                    total_tokens: response
                                                        .prompt_eval_count
                                                        .unwrap_or(0)
                                                        + response.eval_count.unwrap_or(0),
                                                }),
                                            };
                                            return Some((Ok(chunk), (bytes_stream, buffer)));
                                        } else {
                                            // 内容块
                                            let chunk = StreamChunk {
                                                content: response.message.content,
                                                finish_reason: None,
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

    async fn cancel(&self, _request_id: &str) -> Result<(), LLMError> {
        // Ollama 不支持取消请求
        Ok(())
    }

    async fn health_check(&self) -> Result<HealthStatus, LLMError> {
        let start = std::time::Instant::now();

        match self
            .client
            .get(format!("{}/api/tags", self.config.base_url))
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
