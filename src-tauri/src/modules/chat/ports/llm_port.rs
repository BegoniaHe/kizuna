use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use thiserror::Error;

/// LLM 错误类型
#[derive(Debug, Error)]
pub enum LLMError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("API error: {code} - {message}")]
    ApiError { code: String, message: String },

    #[error("Rate limit exceeded, retry after {retry_after_secs}s")]
    RateLimitError { retry_after_secs: u64 },

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Context length exceeded: {used} > {max}")]
    ContextLengthExceeded { used: u32, max: u32 },

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Request cancelled")]
    Cancelled,

    #[error("Provider not available: {0}")]
    ProviderNotAvailable(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// LLM 提供商类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    OpenAI,
    Claude,
    Ollama,
    Custom,
}

/// 提供商信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub provider_type: ProviderType,
    pub models: Vec<ModelInfo>,
}

/// 模型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub context_length: u32,
    pub supports_vision: bool,
    pub supports_functions: bool,
}

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMChatMessage {
    pub role: String,
    pub content: String,
}

/// 补全请求
#[derive(Debug, Clone)]
pub struct CompletionRequest {
    /// 消息历史
    pub messages: Vec<LLMChatMessage>,
    /// 模型 ID
    pub model: String,
    /// 最大生成 token 数
    pub max_tokens: Option<u32>,
    /// 温度参数 (0.0 - 2.0)
    pub temperature: Option<f32>,
    /// 停止序列
    pub stop_sequences: Option<Vec<String>>,
    /// 请求 ID（用于取消）
    pub request_id: Option<String>,
}

impl CompletionRequest {
    pub fn new(messages: Vec<LLMChatMessage>, model: impl Into<String>) -> Self {
        Self {
            messages,
            model: model.into(),
            max_tokens: None,
            temperature: None,
            stop_sequences: None,
            request_id: None,
        }
    }

    pub fn with_max_tokens(mut self, tokens: u32) -> Self {
        self.max_tokens = Some(tokens);
        self
    }

    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }

    pub fn with_request_id(mut self, id: impl Into<String>) -> Self {
        self.request_id = Some(id.into());
        self
    }
}

/// 补全响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionResponse {
    pub content: String,
    pub finish_reason: FinishReason,
    pub usage: TokenUsage,
}

/// 流式响应块
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamChunk {
    /// 内容块
    pub content: String,
    /// 结束原因（最后一个块才有）
    pub finish_reason: Option<FinishReason>,
    /// Token 使用情况（最后一个块才有）
    pub usage: Option<TokenUsage>,
}

/// 结束原因
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
    FunctionCall,
}

/// Token 使用统计
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// 健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub latency_ms: Option<u64>,
    pub error_message: Option<String>,
}

/// LLM 服务端口 - 核心抽象接口
///
/// 所有 LLM 提供商适配器都必须实现此 trait
#[async_trait]
pub trait LLMPort: Send + Sync {
    /// 获取提供商 ID
    fn provider_id(&self) -> &str;

    /// 获取提供商信息
    fn provider_info(&self) -> ProviderInfo;

    /// 获取支持的模型列表
    async fn list_models(&self) -> Result<Vec<ModelInfo>, LLMError>;

    /// 单次补全请求
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LLMError>;

    /// 流式补全请求
    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, LLMError>> + Send>>, LLMError>;

    /// 取消正在进行的请求
    async fn cancel(&self, request_id: &str) -> Result<(), LLMError>;

    /// 健康检查
    async fn health_check(&self) -> Result<HealthStatus, LLMError>;
}

/// LLM 端口工厂 trait
///
/// 用于动态创建 LLM 适配器实例
pub trait LLMPortFactory: Send + Sync {
    /// 创建适配器实例
    fn create(&self, config: &LLMProviderConfig) -> Result<Box<dyn LLMPort>, LLMError>;

    /// 获取支持的提供商类型
    fn supported_providers(&self) -> Vec<ProviderType>;
}

/// LLM 提供商配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LLMProviderConfig {
    pub id: String,
    pub name: String,
    pub provider_type: ProviderType,
    pub base_url: String,
    pub api_key: String,
    pub default_model: String,
    pub timeout_secs: u64,
    pub max_retries: u32,
}

impl Default for LLMProviderConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            provider_type: ProviderType::OpenAI,
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: String::new(),
            default_model: "gpt-3.5-turbo".to_string(),
            timeout_secs: 60,
            max_retries: 3,
        }
    }
}
