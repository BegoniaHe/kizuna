use async_trait::async_trait;
use std::sync::Arc;

use super::super::{ApplicationError, QueryHandler};
use crate::modules::chat::domain::Session;
use crate::modules::chat::ports::{PaginatedResult, Pagination, SessionRepository};

/// 列出会话查询
#[derive(Debug, Clone)]
pub struct ListSessionsQuery {
    pub page: u32,
    pub limit: u32,
}

impl ListSessionsQuery {
    pub fn new(page: u32, limit: u32) -> Self {
        Self { page, limit }
    }
}

impl Default for ListSessionsQuery {
    fn default() -> Self {
        Self { page: 1, limit: 20 }
    }
}

/// 列出会话响应
#[derive(Debug, Clone)]
pub struct ListSessionsResponse {
    pub sessions: Vec<Session>,
    pub total: usize,
    pub page: u32,
    pub limit: u32,
    pub has_more: bool,
}

impl From<PaginatedResult<Session>> for ListSessionsResponse {
    fn from(result: PaginatedResult<Session>) -> Self {
        let has_more = result.has_next();
        Self {
            sessions: result.items,
            total: result.total,
            page: result.page,
            limit: result.limit,
            has_more,
        }
    }
}

/// 列出会话查询处理器
pub struct ListSessionsHandler {
    session_repository: Arc<dyn SessionRepository>,
}

impl ListSessionsHandler {
    pub fn new(session_repository: Arc<dyn SessionRepository>) -> Self {
        Self { session_repository }
    }
}

#[async_trait]
impl QueryHandler<ListSessionsQuery, ListSessionsResponse> for ListSessionsHandler {
    async fn handle(
        &self,
        query: ListSessionsQuery,
    ) -> Result<ListSessionsResponse, ApplicationError> {
        let pagination = Pagination::new(query.page, query.limit);
        let result = self.session_repository.find_all(pagination).await?;

        Ok(result.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::chat::infrastructure::InMemorySessionRepository;

    #[tokio::test]
    async fn test_list_sessions() {
        let repo = Arc::new(InMemorySessionRepository::new());
        let handler = ListSessionsHandler::new(repo.clone());

        // 创建会话
        for i in 0..5 {
            let session = Session::new(Some(format!("Session {}", i)), None);
            repo.save(&session).await.unwrap();
        }

        let query = ListSessionsQuery::new(1, 10);
        let response = handler.handle(query).await.unwrap();

        assert_eq!(response.sessions.len(), 5);
        assert_eq!(response.total, 5);
        assert!(!response.has_more);
    }

    #[tokio::test]
    async fn test_list_sessions_empty() {
        let repo = Arc::new(InMemorySessionRepository::new());
        let handler = ListSessionsHandler::new(repo);

        let query = ListSessionsQuery::default();
        let response = handler.handle(query).await.unwrap();

        assert!(response.sessions.is_empty());
        assert_eq!(response.total, 0);
    }
}
