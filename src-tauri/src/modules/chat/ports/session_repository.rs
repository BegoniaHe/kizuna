use async_trait::async_trait;
use thiserror::Error;

use super::super::domain::{Session, SessionId};

/// 仓储错误类型
#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Conflict: {0}")]
    Conflict(String),
}

/// 分页参数
#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    pub page: u32,
    pub limit: u32,
}

impl Pagination {
    pub fn new(page: u32, limit: u32) -> Self {
        Self { page, limit }
    }

    pub fn offset(&self) -> u32 {
        (self.page.saturating_sub(1)) * self.limit
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self { page: 1, limit: 20 }
    }
}

/// 分页结果
#[derive(Debug, Clone)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub page: u32,
    pub limit: u32,
}

impl<T> PaginatedResult<T> {
    pub fn new(items: Vec<T>, total: usize, pagination: Pagination) -> Self {
        Self {
            items,
            total,
            page: pagination.page,
            limit: pagination.limit,
        }
    }

    pub fn has_next(&self) -> bool {
        (self.page as usize * self.limit as usize) < self.total
    }

    pub fn has_prev(&self) -> bool {
        self.page > 1
    }

    pub fn total_pages(&self) -> u32 {
        ((self.total as f64) / (self.limit as f64)).ceil() as u32
    }
}

/// 会话仓储端口
///
/// 定义会话持久化的抽象接口
#[async_trait]
pub trait SessionRepository: Send + Sync {
    /// 根据 ID 获取会话
    async fn get(&self, id: SessionId) -> Result<Option<Session>, RepositoryError>;

    /// 保存会话（创建或更新）
    async fn save(&self, session: &Session) -> Result<(), RepositoryError>;

    /// 删除会话
    async fn delete(&self, id: SessionId) -> Result<(), RepositoryError>;

    /// 获取所有会话（分页）
    async fn find_all(
        &self,
        pagination: Pagination,
    ) -> Result<PaginatedResult<Session>, RepositoryError>;

    /// 检查会话是否存在
    async fn exists(&self, id: SessionId) -> Result<bool, RepositoryError>;

    /// 获取会话总数
    async fn count(&self) -> Result<usize, RepositoryError>;
}
