// Chat Application Layer - 应用层
// 实现 CQRS 模式的命令和查询处理器

pub mod commands;
pub mod queries;

// 导出命令和查询
pub use commands::*;
pub use queries::*;

use async_trait::async_trait;
use thiserror::Error;

use super::ports::{LLMError, RepositoryError};

/// 应用层错误类型
#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Message not found: {0}")]
    MessageNotFound(String),

    #[error("LLM error: {0}")]
    LLMError(#[from] LLMError),

    #[error("Repository error: {0}")]
    RepositoryError(#[from] RepositoryError),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// 命令处理器 trait
///
/// 遵循 CQRS 模式，命令处理器负责执行有副作用的操作
#[async_trait]
pub trait CommandHandler<C, R>: Send + Sync
where
    C: Send + Sync,
{
    /// 执行命令
    async fn handle(&self, command: C) -> Result<R, ApplicationError>;
}

/// 查询处理器 trait
///
/// 遵循 CQRS 模式，查询处理器负责只读操作
#[async_trait]
pub trait QueryHandler<Q, R>: Send + Sync
where
    Q: Send + Sync,
{
    /// 执行查询
    async fn handle(&self, query: Q) -> Result<R, ApplicationError>;
}
