use async_trait::async_trait;
use std::sync::Arc;

use super::super::{ApplicationError, CommandHandler};
use crate::modules::chat::domain::SessionId;
use crate::modules::chat::ports::{MessageRepository, SessionRepository};

/// 删除会话命令
#[derive(Debug, Clone)]
pub struct DeleteSessionCommand {
    pub session_id: SessionId,
}

impl DeleteSessionCommand {
    pub fn new(session_id: SessionId) -> Self {
        Self { session_id }
    }
}

/// 删除会话命令响应
#[derive(Debug, Clone)]
pub struct DeleteSessionResponse {
    /// 删除的消息数量
    pub deleted_messages: usize,
}

/// 删除会话命令处理器
pub struct DeleteSessionHandler {
    session_repository: Arc<dyn SessionRepository>,
    message_repository: Arc<dyn MessageRepository>,
}

impl DeleteSessionHandler {
    pub fn new(
        session_repository: Arc<dyn SessionRepository>,
        message_repository: Arc<dyn MessageRepository>,
    ) -> Self {
        Self {
            session_repository,
            message_repository,
        }
    }
}

#[async_trait]
impl CommandHandler<DeleteSessionCommand, DeleteSessionResponse> for DeleteSessionHandler {
    async fn handle(
        &self,
        command: DeleteSessionCommand,
    ) -> Result<DeleteSessionResponse, ApplicationError> {
        // 验证会话存在
        let exists = self.session_repository.exists(command.session_id).await?;
        if !exists {
            return Err(ApplicationError::SessionNotFound(
                command.session_id.to_string(),
            ));
        }

        // 删除会话下的所有消息
        let deleted_messages = self
            .message_repository
            .delete_by_session(command.session_id)
            .await?;

        // 删除会话
        self.session_repository.delete(command.session_id).await?;

        Ok(DeleteSessionResponse { deleted_messages })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::chat::domain::{Message, Session};
    use crate::modules::chat::infrastructure::{
        InMemoryMessageRepository, InMemorySessionRepository,
    };

    #[tokio::test]
    async fn test_delete_session_with_messages() {
        let session_repo = Arc::new(InMemorySessionRepository::new());
        let message_repo = Arc::new(InMemoryMessageRepository::new());
        let handler = DeleteSessionHandler::new(session_repo.clone(), message_repo.clone());

        // 创建会话和消息
        let session = Session::new(Some("Test".to_string()), None);
        let session_id = session.id();
        session_repo.save(&session).await.unwrap();

        let msg1 = Message::new_user(session_id, "Hello");
        let msg2 = Message::new_assistant(session_id, "Hi", None);
        message_repo.save(&msg1).await.unwrap();
        message_repo.save(&msg2).await.unwrap();

        // 删除会话
        let command = DeleteSessionCommand::new(session_id);
        let response = handler.handle(command).await.unwrap();

        assert_eq!(response.deleted_messages, 2);

        // 验证会话已删除
        assert!(!session_repo.exists(session_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_session() {
        let session_repo = Arc::new(InMemorySessionRepository::new());
        let message_repo = Arc::new(InMemoryMessageRepository::new());
        let handler = DeleteSessionHandler::new(session_repo, message_repo);

        let command = DeleteSessionCommand::new(SessionId::new());
        let result = handler.handle(command).await;

        assert!(matches!(result, Err(ApplicationError::SessionNotFound(_))));
    }
}
