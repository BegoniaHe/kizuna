use async_trait::async_trait;

use super::super::domain::{Message, MessageId, SessionId};
use super::session_repository::{PaginatedResult, Pagination, RepositoryError};

/// 消息仓储端口
///
/// 定义消息持久化的抽象接口
#[async_trait]
pub trait MessageRepository: Send + Sync {
    /// 根据 ID 获取消息
    async fn get(&self, id: MessageId) -> Result<Option<Message>, RepositoryError>;

    /// 保存消息
    async fn save(&self, message: &Message) -> Result<(), RepositoryError>;

    /// 删除消息
    async fn delete(&self, id: MessageId) -> Result<(), RepositoryError>;

    /// 获取会话的所有消息
    async fn find_by_session(
        &self,
        session_id: SessionId,
        pagination: Pagination,
    ) -> Result<PaginatedResult<Message>, RepositoryError>;

    /// 删除会话的所有消息
    async fn delete_by_session(&self, session_id: SessionId) -> Result<usize, RepositoryError>;

    /// 获取会话的最后一条消息
    async fn find_last_by_session(
        &self,
        session_id: SessionId,
    ) -> Result<Option<Message>, RepositoryError>;

    /// 获取会话的消息数量
    async fn count_by_session(&self, session_id: SessionId) -> Result<usize, RepositoryError>;
}
