use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::modules::chat::domain::{Session, SessionId};
use crate::modules::chat::ports::{
    PaginatedResult, Pagination, RepositoryError, SessionRepository,
};

/// 内存会话仓储
///
/// 用于开发和测试，后续可替换为 SQLite 实现
pub struct InMemorySessionRepository {
    sessions: RwLock<HashMap<SessionId, Session>>,
}

impl InMemorySessionRepository {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemorySessionRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionRepository for InMemorySessionRepository {
    async fn get(&self, id: SessionId) -> Result<Option<Session>, RepositoryError> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(&id).cloned())
    }

    async fn save(&self, session: &Session) -> Result<(), RepositoryError> {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id(), session.clone());
        Ok(())
    }

    async fn delete(&self, id: SessionId) -> Result<(), RepositoryError> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(&id);
        Ok(())
    }

    async fn find_all(
        &self,
        pagination: Pagination,
    ) -> Result<PaginatedResult<Session>, RepositoryError> {
        let sessions = self.sessions.read().await;

        // 按更新时间排序（最新的在前）
        let mut all_sessions: Vec<Session> = sessions.values().cloned().collect();
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
        let sessions = self.sessions.read().await;
        Ok(sessions.contains_key(&id))
    }

    async fn count(&self) -> Result<usize, RepositoryError> {
        let sessions = self.sessions.read().await;
        Ok(sessions.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_save_and_get() {
        let repo = InMemorySessionRepository::new();
        let session = Session::new(Some("Test".to_string()), None);
        let id = session.id();

        repo.save(&session).await.unwrap();
        let retrieved = repo.get(id).await.unwrap();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title(), "Test");
    }

    #[tokio::test]
    async fn test_delete() {
        let repo = InMemorySessionRepository::new();
        let session = Session::new(None, None);
        let id = session.id();

        repo.save(&session).await.unwrap();
        assert!(repo.exists(id).await.unwrap());

        repo.delete(id).await.unwrap();
        assert!(!repo.exists(id).await.unwrap());
    }

    #[tokio::test]
    async fn test_pagination() {
        let repo = InMemorySessionRepository::new();

        // 创建 25 个会话
        for i in 0..25 {
            let session = Session::new(Some(format!("Session {}", i)), None);
            repo.save(&session).await.unwrap();
        }

        // 第一页
        let page1 = repo.find_all(Pagination::new(1, 10)).await.unwrap();
        assert_eq!(page1.items.len(), 10);
        assert_eq!(page1.total, 25);
        assert!(page1.has_next());

        // 第三页
        let page3 = repo.find_all(Pagination::new(3, 10)).await.unwrap();
        assert_eq!(page3.items.len(), 5);
        assert!(!page3.has_next());
    }
}
