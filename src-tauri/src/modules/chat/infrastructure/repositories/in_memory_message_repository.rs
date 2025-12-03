use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::modules::chat::domain::{Message, MessageId, SessionId};
use crate::modules::chat::ports::{
    MessageRepository, PaginatedResult, Pagination, RepositoryError,
};

/// 内存消息仓储
///
/// 用于开发和测试，后续可替换为 SQLite 实现
pub struct InMemoryMessageRepository {
    /// 消息存储（按会话分组）
    messages: RwLock<HashMap<SessionId, Vec<Message>>>,
}

impl InMemoryMessageRepository {
    pub fn new() -> Self {
        Self {
            messages: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryMessageRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MessageRepository for InMemoryMessageRepository {
    async fn get(&self, id: MessageId) -> Result<Option<Message>, RepositoryError> {
        let messages = self.messages.read().await;

        for session_messages in messages.values() {
            if let Some(msg) = session_messages.iter().find(|m| m.id() == id) {
                return Ok(Some(msg.clone()));
            }
        }

        Ok(None)
    }

    async fn save(&self, message: &Message) -> Result<(), RepositoryError> {
        let mut messages = self.messages.write().await;
        let session_messages = messages.entry(message.session_id()).or_default();

        // 检查是否已存在（更新）
        if let Some(existing) = session_messages.iter_mut().find(|m| m.id() == message.id()) {
            *existing = message.clone();
        } else {
            session_messages.push(message.clone());
        }

        Ok(())
    }

    async fn delete(&self, id: MessageId) -> Result<(), RepositoryError> {
        let mut messages = self.messages.write().await;

        for session_messages in messages.values_mut() {
            session_messages.retain(|m| m.id() != id);
        }

        Ok(())
    }

    async fn find_by_session(
        &self,
        session_id: SessionId,
        pagination: Pagination,
    ) -> Result<PaginatedResult<Message>, RepositoryError> {
        let messages = self.messages.read().await;

        let session_messages = messages.get(&session_id);
        let all_messages: Vec<Message> = session_messages
            .map(|msgs| msgs.clone())
            .unwrap_or_default();

        let total = all_messages.len();
        let offset = pagination.offset() as usize;
        let limit = pagination.limit as usize;

        let items = if offset < total {
            all_messages[offset..total.min(offset + limit)].to_vec()
        } else {
            Vec::new()
        };

        Ok(PaginatedResult::new(items, total, pagination))
    }

    async fn delete_by_session(&self, session_id: SessionId) -> Result<usize, RepositoryError> {
        let mut messages = self.messages.write().await;

        if let Some(session_messages) = messages.remove(&session_id) {
            Ok(session_messages.len())
        } else {
            Ok(0)
        }
    }

    async fn find_last_by_session(
        &self,
        session_id: SessionId,
    ) -> Result<Option<Message>, RepositoryError> {
        let messages = self.messages.read().await;

        Ok(messages
            .get(&session_id)
            .and_then(|msgs| msgs.last().cloned()))
    }

    async fn count_by_session(&self, session_id: SessionId) -> Result<usize, RepositoryError> {
        let messages = self.messages.read().await;

        Ok(messages
            .get(&session_id)
            .map(|msgs| msgs.len())
            .unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_save_and_get() {
        let repo = InMemoryMessageRepository::new();
        let session_id = SessionId::new();
        let message = Message::new_user(session_id, "Hello");
        let id = message.id();

        repo.save(&message).await.unwrap();
        let retrieved = repo.get(id).await.unwrap();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content(), "Hello");
    }

    #[tokio::test]
    async fn test_find_by_session() {
        let repo = InMemoryMessageRepository::new();
        let session_id = SessionId::new();

        // 添加多条消息
        for i in 0..5 {
            let msg = Message::new_user(session_id, format!("Message {}", i));
            repo.save(&msg).await.unwrap();
        }

        let result = repo
            .find_by_session(session_id, Pagination::new(1, 10))
            .await
            .unwrap();

        assert_eq!(result.items.len(), 5);
        assert_eq!(result.total, 5);
    }

    #[tokio::test]
    async fn test_delete_by_session() {
        let repo = InMemoryMessageRepository::new();
        let session_id = SessionId::new();

        for _ in 0..3 {
            let msg = Message::new_user(session_id, "Test");
            repo.save(&msg).await.unwrap();
        }

        let deleted = repo.delete_by_session(session_id).await.unwrap();
        assert_eq!(deleted, 3);

        let count = repo.count_by_session(session_id).await.unwrap();
        assert_eq!(count, 0);
    }
}
