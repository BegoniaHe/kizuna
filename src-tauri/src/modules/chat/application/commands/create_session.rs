use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use super::super::{ApplicationError, CommandHandler};
use crate::modules::chat::domain::Session;
use crate::modules::chat::ports::SessionRepository;

/// 创建会话命令
#[derive(Debug, Clone)]
pub struct CreateSessionCommand {
    /// 会话标题（可选）
    pub title: Option<String>,
    /// 预设 ID（可选）
    pub preset_id: Option<Uuid>,
}

impl CreateSessionCommand {
    pub fn new(title: Option<String>, preset_id: Option<Uuid>) -> Self {
        Self { title, preset_id }
    }
}

/// 创建会话命令响应
#[derive(Debug, Clone)]
pub struct CreateSessionResponse {
    pub session: Session,
}

/// 创建会话命令处理器
pub struct CreateSessionHandler {
    session_repository: Arc<dyn SessionRepository>,
}

impl CreateSessionHandler {
    pub fn new(session_repository: Arc<dyn SessionRepository>) -> Self {
        Self { session_repository }
    }
}

#[async_trait]
impl CommandHandler<CreateSessionCommand, CreateSessionResponse> for CreateSessionHandler {
    async fn handle(
        &self,
        command: CreateSessionCommand,
    ) -> Result<CreateSessionResponse, ApplicationError> {
        // 创建新会话
        let session = Session::new(command.title, command.preset_id);

        // 持久化
        self.session_repository.save(&session).await?;

        Ok(CreateSessionResponse { session })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::chat::infrastructure::InMemorySessionRepository;

    #[tokio::test]
    async fn test_create_session_with_title() {
        let repo = Arc::new(InMemorySessionRepository::new());
        let handler = CreateSessionHandler::new(repo.clone());

        let command = CreateSessionCommand::new(Some("Test Session".to_string()), None);
        let response = handler.handle(command).await.unwrap();

        assert_eq!(response.session.title(), "Test Session");

        // 验证已持久化
        let saved = repo.get(response.session.id()).await.unwrap();
        assert!(saved.is_some());
    }

    #[tokio::test]
    async fn test_create_session_default_title() {
        let repo = Arc::new(InMemorySessionRepository::new());
        let handler = CreateSessionHandler::new(repo);

        let command = CreateSessionCommand::new(None, None);
        let response = handler.handle(command).await.unwrap();

        assert_eq!(response.session.title(), "新对话");
    }
}
