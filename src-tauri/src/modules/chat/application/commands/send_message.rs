use async_trait::async_trait;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc;

use super::super::{ApplicationError, CommandHandler};
use crate::modules::chat::domain::{ContextBuilder, EmotionAnalyzer, Message, Session, SessionId};
use crate::modules::chat::ports::{
    CompletionRequest, LLMChatMessage, LLMPort, MessageRepository, SessionRepository,
};

/// 发送消息命令
#[derive(Debug, Clone)]
pub struct SendMessageCommand {
    /// 会话 ID
    pub session_id: SessionId,
    /// 用户消息内容
    pub content: String,
    /// 模型 ID
    pub model: Option<String>,
    /// 是否使用流式响应
    pub stream: bool,
}

impl SendMessageCommand {
    pub fn new(
        session_id: SessionId,
        content: impl Into<String>,
        model: Option<String>,
        stream: bool,
    ) -> Self {
        Self {
            session_id,
            content: content.into(),
            model,
            stream,
        }
    }
}

/// 发送消息响应
#[derive(Debug, Clone)]
pub struct SendMessageResponse {
    /// 用户消息
    pub user_message: Message,
    /// 助手回复（非流式时完整内容，流式时初始为空）
    pub assistant_message: Message,
}

/// 流式响应事件
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// 内容块
    Chunk(String),
    /// 完成
    Done {
        full_content: String,
        tokens_used: Option<u32>,
    },
    /// 错误
    Error(String),
}

/// 发送消息命令处理器
pub struct SendMessageHandler {
    session_repository: Arc<dyn SessionRepository>,
    message_repository: Arc<dyn MessageRepository>,
    llm_port: Arc<dyn LLMPort>,
    #[allow(dead_code)] // TODO: 将在后续实现上下文构建时使用
    context_builder: ContextBuilder,
    emotion_analyzer: EmotionAnalyzer,
    default_model: String,
}

impl SendMessageHandler {
    pub fn new(
        session_repository: Arc<dyn SessionRepository>,
        message_repository: Arc<dyn MessageRepository>,
        llm_port: Arc<dyn LLMPort>,
        default_model: impl Into<String>,
    ) -> Self {
        Self {
            session_repository,
            message_repository,
            llm_port,
            context_builder: ContextBuilder::new(),
            emotion_analyzer: EmotionAnalyzer::new(),
            default_model: default_model.into(),
        }
    }

    /// 构建聊天上下文
    async fn build_context(
        &self,
        session: &Session,
        user_message: &Message,
    ) -> Result<Vec<LLMChatMessage>, ApplicationError> {
        // 获取历史消息
        let pagination = crate::modules::chat::ports::Pagination::new(1, 50);
        let messages = self
            .message_repository
            .find_by_session(session.id(), pagination)
            .await?;

        // 构建上下文
        let mut context = Vec::with_capacity(messages.items.len() + 2);

        // 添加系统提示（如果有预设）
        if let Some(_preset_id) = session.preset_id() {
            // TODO: 从预设仓储获取系统提示
            context.push(LLMChatMessage {
                role: "system".to_string(),
                content: "You are a helpful AI assistant.".to_string(),
            });
        }

        // 添加历史消息
        for msg in &messages.items {
            context.push(LLMChatMessage {
                role: msg.role().to_openai_role().to_string(),
                content: msg.content().to_string(),
            });
        }

        // 添加当前用户消息
        context.push(LLMChatMessage {
            role: "user".to_string(),
            content: user_message.content().to_string(),
        });

        Ok(context)
    }

    /// 处理流式响应
    pub async fn handle_stream(
        &self,
        command: SendMessageCommand,
    ) -> Result<(SendMessageResponse, mpsc::Receiver<StreamEvent>), ApplicationError> {
        // 验证会话存在
        let session = self
            .session_repository
            .get(command.session_id)
            .await?
            .ok_or_else(|| ApplicationError::SessionNotFound(command.session_id.to_string()))?;

        // 创建用户消息
        let user_message = Message::new_user(command.session_id, &command.content);
        self.message_repository.save(&user_message).await?;

        // 创建助手消息（初始为空）
        let assistant_message = Message::new_assistant(command.session_id, "", None);

        // 构建上下文
        let context = self.build_context(&session, &user_message).await?;

        // 创建补全请求
        let model = command.model.unwrap_or_else(|| self.default_model.clone());
        let request = CompletionRequest::new(context, model);

        // 创建响应通道
        let (tx, rx) = mpsc::channel::<StreamEvent>(32);

        // 启动流式处理
        let llm = self.llm_port.clone();
        let message_repo = self.message_repository.clone();
        let emotion_analyzer = self.emotion_analyzer.clone();
        let _msg_id = assistant_message.id();
        let session_id = command.session_id;

        tokio::spawn(async move {
            let result = llm.complete_stream(request).await;
            match result {
                Ok(mut stream) => {
                    let mut full_content = String::new();
                    let mut tokens_used = None;

                    while let Some(chunk_result) = stream.next().await {
                        match chunk_result {
                            Ok(chunk) => {
                                full_content.push_str(&chunk.content);

                                // 发送内容块
                                if tx.send(StreamEvent::Chunk(chunk.content)).await.is_err() {
                                    break;
                                }

                                // 检查是否完成
                                if chunk.finish_reason.is_some() {
                                    tokens_used = chunk.usage.map(|u| u.total_tokens);
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(StreamEvent::Error(e.to_string())).await;
                                return;
                            }
                        }
                    }

                    // 分析情感
                    let emotion = emotion_analyzer.analyze(&full_content);

                    // 保存完整的助手消息
                    let final_message = Message::new_assistant(session_id, &full_content, emotion);
                    if let Err(e) = message_repo.save(&final_message).await {
                        let _ = tx
                            .send(StreamEvent::Error(format!("Failed to save message: {}", e)))
                            .await;
                        return;
                    }

                    // 发送完成事件
                    let _ = tx
                        .send(StreamEvent::Done {
                            full_content,
                            tokens_used,
                        })
                        .await;
                }
                Err(e) => {
                    let _ = tx.send(StreamEvent::Error(e.to_string())).await;
                }
            }
        });

        Ok((
            SendMessageResponse {
                user_message,
                assistant_message,
            },
            rx,
        ))
    }
}

#[async_trait]
impl CommandHandler<SendMessageCommand, SendMessageResponse> for SendMessageHandler {
    async fn handle(
        &self,
        command: SendMessageCommand,
    ) -> Result<SendMessageResponse, ApplicationError> {
        // 验证输入
        if command.content.trim().is_empty() {
            return Err(ApplicationError::ValidationError(
                "Message content cannot be empty".to_string(),
            ));
        }

        // 验证会话存在
        let session = self
            .session_repository
            .get(command.session_id)
            .await?
            .ok_or_else(|| ApplicationError::SessionNotFound(command.session_id.to_string()))?;

        // 创建用户消息
        let user_message = Message::new_user(command.session_id, &command.content);
        self.message_repository.save(&user_message).await?;

        // 构建上下文
        let context = self.build_context(&session, &user_message).await?;

        // 创建补全请求
        let model = command.model.unwrap_or_else(|| self.default_model.clone());
        let request = CompletionRequest::new(context, model);

        // 非流式：等待完整响应
        let response = self.llm_port.complete(request).await?;

        // 分析情感
        let emotion = self.emotion_analyzer.analyze(&response.content);

        // 创建并保存助手消息
        let assistant_message =
            Message::new_assistant(command.session_id, &response.content, emotion);
        self.message_repository.save(&assistant_message).await?;

        Ok(SendMessageResponse {
            user_message,
            assistant_message,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::chat::domain::Session;
    use crate::modules::chat::infrastructure::{
        InMemoryMessageRepository, InMemorySessionRepository,
    };
    use crate::modules::chat::ports::{
        CompletionResponse, FinishReason, HealthStatus, LLMError, ModelInfo, ProviderInfo,
        ProviderType, StreamChunk, TokenUsage,
    };
    use std::pin::Pin;

    /// Mock LLM Port for testing
    struct MockLLMPort;

    #[async_trait]
    impl LLMPort for MockLLMPort {
        fn provider_id(&self) -> &str {
            "mock"
        }

        fn provider_info(&self) -> ProviderInfo {
            ProviderInfo {
                id: "mock".to_string(),
                name: "Mock Provider".to_string(),
                provider_type: ProviderType::Custom,
                models: vec![],
            }
        }

        async fn list_models(&self) -> Result<Vec<ModelInfo>, LLMError> {
            Ok(vec![])
        }

        async fn complete(
            &self,
            _request: CompletionRequest,
        ) -> Result<CompletionResponse, LLMError> {
            Ok(CompletionResponse {
                content: "Hello! How can I help you?".to_string(),
                finish_reason: FinishReason::Stop,
                usage: TokenUsage {
                    prompt_tokens: 10,
                    completion_tokens: 8,
                    total_tokens: 18,
                },
            })
        }

        async fn complete_stream(
            &self,
            _request: CompletionRequest,
        ) -> Result<
            Pin<Box<dyn futures::Stream<Item = Result<StreamChunk, LLMError>> + Send>>,
            LLMError,
        > {
            Err(LLMError::Unknown("Not implemented".to_string()))
        }

        async fn cancel(&self, _request_id: &str) -> Result<(), LLMError> {
            Ok(())
        }

        async fn health_check(&self) -> Result<HealthStatus, LLMError> {
            Ok(HealthStatus {
                is_healthy: true,
                latency_ms: Some(10),
                error_message: None,
            })
        }
    }

    #[tokio::test]
    async fn test_send_message() {
        let session_repo = Arc::new(InMemorySessionRepository::new());
        let message_repo = Arc::new(InMemoryMessageRepository::new());
        let llm = Arc::new(MockLLMPort);

        // 创建会话
        let session = Session::new(Some("Test".to_string()), None);
        let session_id = session.id();
        session_repo.save(&session).await.unwrap();

        let handler =
            SendMessageHandler::new(session_repo, message_repo.clone(), llm, "gpt-3.5-turbo");

        let command = SendMessageCommand::new(session_id, "Hello", None, false);
        let response = handler.handle(command).await.unwrap();

        assert_eq!(response.user_message.content(), "Hello");
        assert_eq!(
            response.assistant_message.content(),
            "Hello! How can I help you?"
        );

        // 验证消息已保存
        let count = message_repo.count_by_session(session_id).await.unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_send_empty_message() {
        let session_repo = Arc::new(InMemorySessionRepository::new());
        let message_repo = Arc::new(InMemoryMessageRepository::new());
        let llm = Arc::new(MockLLMPort);

        let session = Session::new(None, None);
        let session_id = session.id();
        session_repo.save(&session).await.unwrap();

        let handler = SendMessageHandler::new(session_repo, message_repo, llm, "gpt-3.5-turbo");

        let command = SendMessageCommand::new(session_id, "   ", None, false);
        let result = handler.handle(command).await;

        assert!(matches!(result, Err(ApplicationError::ValidationError(_))));
    }
}
