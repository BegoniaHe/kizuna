// 文件持久化会话仓储实现
//
// 使用 JSON 文件存储会话数据，提供简单的持久化方案
// 后续可切换为 SQLite 实现以支持更复杂的查询需求

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::sync::RwLock;

use crate::modules::chat::domain::{Session, SessionId};
use crate::modules::chat::ports::{
    PaginatedResult, Pagination, RepositoryError, SessionRepository,
};

/// 持久化数据结构
#[derive(Debug, Serialize, Deserialize, Default)]
struct SessionStore {
    sessions: HashMap<String, Session>,
}

/// 文件持久化会话仓储
///
/// 将会话数据存储到 JSON 文件中，提供跨会话的数据持久化
pub struct FileSessionRepository {
    store: RwLock<SessionStore>,
    file_path: PathBuf,
}

impl FileSessionRepository {
    /// 创建新的文件会话仓储
    ///
    /// # Arguments
    /// * `data_dir` - 应用数据目录路径
    pub async fn new(data_dir: PathBuf) -> Result<Self, RepositoryError> {
        let file_path = data_dir.join("sessions.json");

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
            SessionStore::default()
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
impl SessionRepository for FileSessionRepository {
    async fn get(&self, id: SessionId) -> Result<Option<Session>, RepositoryError> {
        let store = self.store.read().await;
        Ok(store.sessions.get(&id.to_string()).cloned())
    }

    async fn save(&self, session: &Session) -> Result<(), RepositoryError> {
        {
            let mut store = self.store.write().await;
            store
                .sessions
                .insert(session.id().to_string(), session.clone());
        }
        self.persist().await
    }

    async fn delete(&self, id: SessionId) -> Result<(), RepositoryError> {
        {
            let mut store = self.store.write().await;
            store.sessions.remove(&id.to_string());
        }
        self.persist().await
    }

    async fn find_all(
        &self,
        pagination: Pagination,
    ) -> Result<PaginatedResult<Session>, RepositoryError> {
        let store = self.store.read().await;

        // 按更新时间排序（最新的在前）
        let mut all_sessions: Vec<Session> = store.sessions.values().cloned().collect();
        all_sessions.sort_by(|a, b| b.updated_at().cmp(&a.updated_at()));

        let total = all_sessions.len();
        let offset = pagination.offset() as usize;
        let limit = pagination.limit as usize;

        let items = if offset < total {
            all_sessions[offset..total.min(offset + limit)].to_vec()
        } else {
            Vec::new()
        };

        Ok(PaginatedResult::new(items, total, pagination))
    }

    async fn exists(&self, id: SessionId) -> Result<bool, RepositoryError> {
        let store = self.store.read().await;
        Ok(store.sessions.contains_key(&id.to_string()))
    }

    async fn count(&self) -> Result<usize, RepositoryError> {
        let store = self.store.read().await;
        Ok(store.sessions.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_save_and_get() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileSessionRepository::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();

        let session = Session::new(Some("Test".to_string()), None);
        let id = session.id();

        repo.save(&session).await.unwrap();
        let retrieved = repo.get(id).await.unwrap();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title(), "Test");
    }

    #[tokio::test]
    async fn test_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        // 创建并保存会话
        let session = Session::new(Some("Persistent".to_string()), None);
        let id = session.id();

        {
            let repo = FileSessionRepository::new(path.clone()).await.unwrap();
            repo.save(&session).await.unwrap();
        }

        // 重新加载仓储，验证数据持久化
        {
            let repo = FileSessionRepository::new(path).await.unwrap();
            let retrieved = repo.get(id).await.unwrap();

            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap().title(), "Persistent");
        }
    }

    #[tokio::test]
    async fn test_delete() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileSessionRepository::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();

        let session = Session::new(Some("ToDelete".to_string()), None);
        let id = session.id();

        repo.save(&session).await.unwrap();
        assert!(repo.exists(id).await.unwrap());

        repo.delete(id).await.unwrap();
        assert!(!repo.exists(id).await.unwrap());
    }
}
