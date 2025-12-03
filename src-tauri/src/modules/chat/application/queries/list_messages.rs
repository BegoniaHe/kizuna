use async_trait::async_trait;
use std::sync::Arc;

use super::super::{ApplicationError, QueryHandler};
use crate::modules::chat::domain::{Message, SessionId};
use crate::modules::chat::ports::{MessageRepository, PaginatedResult, Pagination};

/// 列出消息查询
#[derive(Debug, Clone)]
pub struct ListMessagesQuery {
    pub session_id: SessionId,
    pub page: u32,
    pub limit: u32,
}

impl ListMessagesQuery {
    pub fn new(session_id: SessionId, page: u32, limit: u32) -> Self {
        Self {
            session_id,
            page,
            limit,
        }
    }

    pub fn for_session(session_id: SessionId) -> Self {
        Self {
            session_id,
            page: 1,
            limit: 50,
        }
    }
}

/// 列出消息响应
#[derive(Debug, Clone)]
pub struct ListMessagesResponse {
    pub messages: Vec<Message>,
    pub total: usize,
    pub page: u32,
    pub limit: u32,
    pub has_more: bool,
}

impl From<PaginatedResult<Message>> for ListMessagesResponse {
    fn from(result: PaginatedResult<Message>) -> Self {
        let has_more = result.has_next();
        Self {
            messages: result.items,
            total: result.total,
            page: result.page,
            limit: result.limit,
            has_more,
        }
    }
}

/// 列出消息查询处理器
pub struct ListMessagesHandler {
    message_repository: Arc<dyn MessageRepository>,
}

impl ListMessagesHandler {
    pub fn new(message_repository: Arc<dyn MessageRepository>) -> Self {
        Self { message_repository }
    }
}

#[async_trait]
impl QueryHandler<ListMessagesQuery, ListMessagesResponse> for ListMessagesHandler {
    async fn handle(
        &self,
        query: ListMessagesQuery,
    ) -> Result<ListMessagesResponse, ApplicationError> {
        let pagination = Pagination::new(query.page, query.limit);
        let result = self
            .message_repository
            .find_by_session(query.session_id, pagination)
            .await?;

        Ok(result.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::chat::domain::Session;
    use crate::modules::chat::infrastructure::InMemoryMessageRepository;

    #[tokio::test]
    async fn test_list_messages() {
        let repo = Arc::new(InMemoryMessageRepository::new());
        let handler = ListMessagesHandler::new(repo.clone());

        let session = Session::new(None, None);
        let session_id = session.id();

        // 添加消息
        for i in 0..5 {
            let msg = Message::new_user(session_id, format!("Message {}", i));
            repo.save(&msg).await.unwrap();
        }

        let query = ListMessagesQuery::new(session_id, 1, 10);
        let response = handler.handle(query).await.unwrap();

        assert_eq!(response.messages.len(), 5);
        assert_eq!(response.total, 5);
        assert!(!response.has_more);
    }

    #[tokio::test]
    async fn test_list_messages_pagination() {
        let repo = Arc::new(InMemoryMessageRepository::new());
        let handler = ListMessagesHandler::new(repo.clone());

        let session = Session::new(None, None);
        let session_id = session.id();

        // 添加消息
        for i in 0..15 {
            let msg = Message::new_user(session_id, format!("Message {}", i));
            repo.save(&msg).await.unwrap();
        }

        let query = ListMessagesQuery::new(session_id, 1, 10);
        let response = handler.handle(query).await.unwrap();

        assert_eq!(response.messages.len(), 10);
        assert_eq!(response.total, 15);
        assert!(response.has_more);
    }
}
