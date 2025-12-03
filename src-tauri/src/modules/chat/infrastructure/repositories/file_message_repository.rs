// 文件持久化消息仓储实现
//
// 使用 JSON 文件存储消息数据，提供简单的持久化方案
// 消息按会话分组存储，便于查询和管理

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::sync::RwLock;

use crate::modules::chat::domain::{Message, MessageId, SessionId};
use crate::modules::chat::ports::{
    MessageRepository, PaginatedResult, Pagination, RepositoryError,
};

/// 持久化数据结构
#[derive(Debug, Serialize, Deserialize, Default)]
struct MessageStore {
    /// 按会话 ID 分组的消息
    messages_by_session: HashMap<String, Vec<Message>>,
}

/// 文件持久化消息仓储
///
/// 将消息数据存储到 JSON 文件中，提供跨会话的数据持久化
pub struct FileMessageRepository {
    store: RwLock<MessageStore>,
    file_path: PathBuf,
}

impl FileMessageRepository {
    /// 创建新的文件消息仓储
    ///
    /// # Arguments
    /// * `data_dir` - 应用数据目录路径
    pub async fn new(data_dir: PathBuf) -> Result<Self, RepositoryError> {
        let file_path = data_dir.join("messages.json");

        // 确保目录存在
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        }

        // 尝试加载现有数据
        let store = if file_path.exists() {
            let content = fs::read_to_string(&file_path)
                .await
                .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

            serde_json::from_str(&content).unwrap_or_default()
        } else {
            MessageStore::default()
        };

        Ok(Self {
            store: RwLock::new(store),
            file_path,
        })
    }

    /// 将数据持久化到文件
    async fn persist(&self) -> Result<(), RepositoryError> {
        let store = self.store.read().await;
        let content = serde_json::to_string_pretty(&*store)
            .map_err(|e| RepositoryError::SerializationError(e.to_string()))?;

        fs::write(&self.file_path, content)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl MessageRepository for FileMessageRepository {
    async fn get(&self, id: MessageId) -> Result<Option<Message>, RepositoryError> {
        let store = self.store.read().await;
        let id_str = id.to_string();

        for messages in store.messages_by_session.values() {
            if let Some(message) = messages.iter().find(|m| m.id().to_string() == id_str) {
                return Ok(Some(message.clone()));
            }
        }

        Ok(None)
    }

    async fn save(&self, message: &Message) -> Result<(), RepositoryError> {
        {
            let mut store = self.store.write().await;
            let session_key = message.session_id().to_string();

            let messages = store
                .messages_by_session
                .entry(session_key)
                .or_insert_with(Vec::new);

            // 检查是否是更新操作
            let id_str = message.id().to_string();
            if let Some(pos) = messages.iter().position(|m| m.id().to_string() == id_str) {
                messages[pos] = message.clone();
            } else {
                messages.push(message.clone());
            }
        }
        self.persist().await
    }

    async fn delete(&self, id: MessageId) -> Result<(), RepositoryError> {
        {
            let mut store = self.store.write().await;
            let id_str = id.to_string();

            for messages in store.messages_by_session.values_mut() {
                if let Some(pos) = messages.iter().position(|m| m.id().to_string() == id_str) {
                    messages.remove(pos);
                    break;
                }
            }
        }
        self.persist().await
    }

    async fn find_by_session(
        &self,
        session_id: SessionId,
        pagination: Pagination,
    ) -> Result<PaginatedResult<Message>, RepositoryError> {
        let store = self.store.read().await;
        let session_key = session_id.to_string();

        let messages = store
            .messages_by_session
            .get(&session_key)
            .cloned()
            .unwrap_or_default();

        // 按创建时间排序（最早的在前）
        let mut sorted_messages = messages;
        sorted_messages.sort_by(|a, b| a.created_at().cmp(&b.created_at()));

        let total = sorted_messages.len();
        let offset = pagination.offset() as usize;
        let limit = pagination.limit as usize;

        let items = if offset < total {
            sorted_messages[offset..total.min(offset + limit)].to_vec()
        } else {
            Vec::new()
        };

        Ok(PaginatedResult::new(items, total, pagination))
    }

    async fn delete_by_session(&self, session_id: SessionId) -> Result<usize, RepositoryError> {
        let count;
        {
            let mut store = self.store.write().await;
            let session_key = session_id.to_string();

            count = store
                .messages_by_session
                .get(&session_key)
                .map(|v| v.len())
                .unwrap_or(0);

            store.messages_by_session.remove(&session_key);
        }
        self.persist().await?;
        Ok(count)
    }

    async fn find_last_by_session(
        &self,
        session_id: SessionId,
    ) -> Result<Option<Message>, RepositoryError> {
        let store = self.store.read().await;
        let session_key = session_id.to_string();

        let messages = store.messages_by_session.get(&session_key);

        Ok(messages
            .and_then(|msgs| msgs.iter().max_by_key(|m| m.created_at()))
            .cloned())
    }

    async fn count_by_session(&self, session_id: SessionId) -> Result<usize, RepositoryError> {
        let store = self.store.read().await;
        let session_key = session_id.to_string();

        Ok(store
            .messages_by_session
            .get(&session_key)
            .map(|v| v.len())
            .unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::chat::domain::MessageRole;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_save_and_get() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileMessageRepository::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();

        let session_id = SessionId::new();
        let message = Message::new(session_id, MessageRole::User, "Hello".to_string());
        let id = message.id();

        repo.save(&message).await.unwrap();
        let retrieved = repo.get(id).await.unwrap();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content(), "Hello");
    }

    #[tokio::test]
    async fn test_find_by_session() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileMessageRepository::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();

        let session_id = SessionId::new();

        for i in 0..5 {
            let message =
                Message::new(session_id, MessageRole::User, format!("Message {}", i));
            repo.save(&message).await.unwrap();
        }

        let result = repo
            .find_by_session(session_id, Pagination::new(1, 3))
            .await
            .unwrap();

        assert_eq!(result.items.len(), 3);
        assert_eq!(result.total, 5);
    }

    #[tokio::test]
    async fn test_delete_by_session() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileMessageRepository::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();

        let session_id = SessionId::new();

        for i in 0..3 {
            let message =
                Message::new(session_id, MessageRole::User, format!("Message {}", i));
            repo.save(&message).await.unwrap();
        }

        let deleted = repo.delete_by_session(session_id).await.unwrap();
        assert_eq!(deleted, 3);

        let count = repo.count_by_session(session_id).await.unwrap();
        assert_eq!(count, 0);
    }
}
