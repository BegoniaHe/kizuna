use async_trait::async_trait;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc;

use super::super::{ApplicationError, CommandHandler};
use super::StreamEvent;
use crate::modules::chat::domain::{EmotionAnalyzer, Message, SessionId};
use crate::modules::chat::ports::{
    CompletionRequest, LLMChatMessage, LLMPort, MessageRepository, Pagination, SessionRepository,
};

/// 重新生成命令（不创建新的用户消息）
#[derive(Debug, Clone)]
pub struct RegenerateCommand {
    /// 会话 ID
    pub session_id: SessionId,
    /// 用户消息内容（用于构建上下文，但不保存）
    pub user_content: String,
    /// 模型 ID
    pub model: Option<String>,
    /// 是否使用流式响应
    pub stream: bool,
}

impl RegenerateCommand {
    pub fn new(
        session_id: SessionId,
        user_content: impl Into<String>,
        model: Option<String>,
        stream: bool,
    ) -> Self {
        Self {
            session_id,
            user_content: user_content.into(),
            model,
            stream,
        }
    }
}

/// 重新生成响应
#[derive(Debug, Clone)]
pub struct RegenerateResponse {
    /// 助手回复
    pub assistant_message: Message,
}

/// 重新生成命令处理器
pub struct RegenerateHandler {
    session_repository: Arc<dyn SessionRepository>,
    message_repository: Arc<dyn MessageRepository>,
    llm_port: Arc<dyn LLMPort>,
    emotion_analyzer: EmotionAnalyzer,
    default_model: String,
}

impl RegenerateHandler {
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
            emotion_analyzer: EmotionAnalyzer::new(),
            default_model: default_model.into(),
        }
    }

    /// 构建聊天上下文（包括最后一条用户消息）
    async fn build_context(
        &self,
        session_id: SessionId,
        user_content: &str,
    ) -> Result<Vec<LLMChatMessage>, ApplicationError> {
        // 获取历史消息（不包括最后一条，因为我们会用传入的 user_content）
        let pagination = Pagination::new(1, 50);
        let messages = self
            .message_repository
            .find_by_session(session_id, pagination)
            .await?;

        let mut context = Vec::new();

        // 添加系统提示
        context.push(LLMChatMessage {
            role: "system".to_string(),
            content: "You are a helpful AI assistant.".to_string(),
        });

        // 添加历史消息（排除最后一条用户消息，因为我们用传入的）
        let mut history: Vec<_> = messages.items.into_iter().collect();
        
        // 1. 如果最后一条是 AI 消息（可能是我们要重新生成的那个），移除它
        while history.last().map(|m| matches!(m.role(), crate::modules::chat::domain::MessageRole::Assistant)).unwrap_or(false) {
            history.pop();
        }

        // 2. 如果最后一条是用户消息（我们要重新发送的那个），移除它
        if history.last().map(|m| matches!(m.role(), crate::modules::chat::domain::MessageRole::User)).unwrap_or(false) {
            history.pop();
        }

        for msg in history {
            context.push(LLMChatMessage {
                role: msg.role().to_openai_role().to_string(),
                content: msg.content().to_string(),
            });
        }

        // 添加当前用户消息内容
        context.push(LLMChatMessage {
            role: "user".to_string(),
            content: user_content.to_string(),
        });

        Ok(context)
    }

    /// 处理流式响应（不保存用户消息）
    pub async fn handle_stream(
        &self,
        command: RegenerateCommand,
    ) -> Result<(RegenerateResponse, mpsc::Receiver<StreamEvent>), ApplicationError> {
        // 验证会话存在
        let _session = self
            .session_repository
            .get(command.session_id)
            .await?
            .ok_or_else(|| ApplicationError::SessionNotFound(command.session_id.to_string()))?;

        // 创建助手消息（初始为空）
        let assistant_message = Message::new_assistant(command.session_id, "", None);

        // 构建上下文（不保存用户消息）
        let context = self
            .build_context(command.session_id, &command.user_content)
            .await?;

        // 创建补全请求
        let model = command.model.unwrap_or_else(|| self.default_model.clone());
        let request = CompletionRequest::new(context, model);

        // 创建响应通道
        let (tx, rx) = mpsc::channel::<StreamEvent>(32);

        // 启动流式处理
        let llm = self.llm_port.clone();
        let message_repo = self.message_repository.clone();
        let emotion_analyzer = self.emotion_analyzer.clone();
        let session_id = command.session_id;
        let assistant_msg = assistant_message.clone();

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
                                if let Some(usage) = &chunk.usage {
                                    tokens_used = Some(usage.total_tokens);
                                }
                                let _ = tx.send(StreamEvent::Chunk(chunk.content)).await;
                            }
                            Err(e) => {
                                let _ = tx.send(StreamEvent::Error(e.to_string())).await;
                                return;
                            }
                        }
                    }

                    // 分析情感
                    let emotion = emotion_analyzer.analyze(&full_content);

                    // 保存助手消息（使用预先创建的 ID）
                    let mut final_message =
                        Message::new_assistant(session_id, &full_content, emotion);
                    final_message.set_id(assistant_msg.id());

                    if let Err(e) = message_repo.save(&final_message).await {
                        let _ = tx.send(StreamEvent::Error(e.to_string())).await;
                        return;
                    }

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
            RegenerateResponse {
                assistant_message,
            },
            rx,
        ))
    }
}

#[async_trait]
impl CommandHandler<RegenerateCommand, RegenerateResponse> for RegenerateHandler {
    async fn handle(
        &self,
        command: RegenerateCommand,
    ) -> Result<RegenerateResponse, ApplicationError> {
        // 验证会话存在
        let _session = self
            .session_repository
            .get(command.session_id)
            .await?
            .ok_or_else(|| ApplicationError::SessionNotFound(command.session_id.to_string()))?;

        // 构建上下文
        let context = self
            .build_context(command.session_id, &command.user_content)
            .await?;

        // 创建补全请求
        let model = command.model.unwrap_or_else(|| self.default_model.clone());
        let request = CompletionRequest::new(context, model);

        // 调用 LLM
        let response = self.llm_port.complete(request).await?;

        // 分析情感
        let emotion = self.emotion_analyzer.analyze(&response.content);

        // 创建并保存助手消息
        let assistant_message =
            Message::new_assistant(command.session_id, &response.content, emotion);
        self.message_repository.save(&assistant_message).await?;

        Ok(RegenerateResponse { assistant_message })
    }
}
