use async_trait::async_trait;
use std::sync::Arc;

use super::super::{ApplicationError, CommandHandler};
use crate::modules::chat::domain::{Session, SessionId};
use crate::modules::chat::ports::SessionRepository;

/// 更新会话命令
#[derive(Debug, Clone)]
pub struct UpdateSessionCommand {
    pub session_id: SessionId,
    pub title: Option<String>,
    pub preset_id: Option<Option<uuid::Uuid>>,
}

impl UpdateSessionCommand {
    pub fn new(
        session_id: SessionId,
        title: Option<String>,
        preset_id: Option<Option<uuid::Uuid>>,
    ) -> Self {
        Self {
            session_id,
            title,
            preset_id,
        }
    }
}

/// 更新会话响应
#[derive(Debug, Clone)]
pub struct UpdateSessionResponse {
    pub session: Session,
}

/// 更新会话处理器
pub struct UpdateSessionHandler {
    session_repository: Arc<dyn SessionRepository>,
}

impl UpdateSessionHandler {
    pub fn new(session_repository: Arc<dyn SessionRepository>) -> Self {
        Self { session_repository }
    }
}

#[async_trait]
impl CommandHandler<UpdateSessionCommand, UpdateSessionResponse> for UpdateSessionHandler {
    async fn handle(
        &self,
        command: UpdateSessionCommand,
    ) -> Result<UpdateSessionResponse, ApplicationError> {
        // 获取现有会话
        let mut session = self
            .session_repository
            .get(command.session_id)
            .await?
            .ok_or_else(|| {
                ApplicationError::SessionNotFound(format!(
                    "Session not found: {}",
                    command.session_id.as_uuid()
                ))
            })?;

        // 更新字段
        if let Some(title) = command.title {
            session.update_title(title);
        }

        if let Some(preset_id) = command.preset_id {
            session.update_preset(preset_id);
        }

        // 保存
        self.session_repository.save(&session).await?;

        Ok(UpdateSessionResponse { session })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::chat::infrastructure::InMemorySessionRepository;

    #[tokio::test]
    async fn test_update_session() {
        let repo = Arc::new(InMemorySessionRepository::new());
        let handler = UpdateSessionHandler::new(repo.clone());

        // 创建测试会话
        let session = Session::new(Some("Old Title".to_string()), None);
        let session_id = session.id();
        repo.save(&session).await.unwrap();

        // 更新标题
        let command = UpdateSessionCommand::new(session_id, Some("New Title".to_string()), None);

        let response = handler.handle(command).await.unwrap();
        assert_eq!(response.session.title(), "New Title");
    }
}
