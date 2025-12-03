use async_trait::async_trait;
use std::sync::Arc;

use super::super::{ApplicationError, QueryHandler};
use crate::modules::chat::domain::{Session, SessionId};
use crate::modules::chat::ports::SessionRepository;

/// 获取会话查询
#[derive(Debug, Clone)]
pub struct GetSessionQuery {
    pub session_id: SessionId,
}

impl GetSessionQuery {
    pub fn new(session_id: SessionId) -> Self {
        Self { session_id }
    }
}

/// 获取会话查询响应
#[derive(Debug, Clone)]
pub struct GetSessionResponse {
    pub session: Option<Session>,
}

/// 获取会话查询处理器
pub struct GetSessionHandler {
    session_repository: Arc<dyn SessionRepository>,
}

impl GetSessionHandler {
    pub fn new(session_repository: Arc<dyn SessionRepository>) -> Self {
        Self { session_repository }
    }
}

#[async_trait]
impl QueryHandler<GetSessionQuery, GetSessionResponse> for GetSessionHandler {
    async fn handle(&self, query: GetSessionQuery) -> Result<GetSessionResponse, ApplicationError> {
        let session = self.session_repository.get(query.session_id).await?;
        Ok(GetSessionResponse { session })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::chat::infrastructure::InMemorySessionRepository;

    #[tokio::test]
    async fn test_get_existing_session() {
        let repo = Arc::new(InMemorySessionRepository::new());
        let handler = GetSessionHandler::new(repo.clone());

        // 创建会话
        let session = Session::new(Some("Test".to_string()), None);
        let session_id = session.id();
        repo.save(&session).await.unwrap();

        let query = GetSessionQuery::new(session_id);
        let response = handler.handle(query).await.unwrap();

        assert!(response.session.is_some());
        assert_eq!(response.session.unwrap().title(), "Test");
    }

    #[tokio::test]
    async fn test_get_nonexistent_session() {
        let repo = Arc::new(InMemorySessionRepository::new());
        let handler = GetSessionHandler::new(repo);

        let query = GetSessionQuery::new(SessionId::new());
        let response = handler.handle(query).await.unwrap();

        assert!(response.session.is_none());
    }
}
