// Chat Module - 聊天模块
//
// 实现六边形架构（Hexagonal Architecture）：
// - domain: 领域层，包含实体、值对象、领域服务和领域事件
// - ports: 端口层，定义与外部世界的抽象接口
// - infrastructure: 基础设施层，实现端口的具体适配器
// - application: 应用层，实现 CQRS 命令和查询处理器

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod ports;

// 重新导出常用类型
pub use application::{
    // Traits
    ApplicationError,
    CommandHandler,
    // Commands
    CreateSessionCommand,
    CreateSessionHandler,
    CreateSessionResponse,
    DeleteSessionCommand,
    DeleteSessionHandler,
    DeleteSessionResponse,
    // Regenerate
    RegenerateCommand,
    RegenerateHandler,
    RegenerateResponse,
    // Queries
    GetSessionHandler,
    GetSessionQuery,
    GetSessionResponse,
    ListMessagesHandler,
    ListMessagesQuery,
    ListMessagesResponse,
    ListSessionsHandler,
    ListSessionsQuery,
    ListSessionsResponse,
    QueryHandler,
    SendMessageCommand,
    SendMessageHandler,
    SendMessageResponse,
    StreamEvent,
    UpdateSessionCommand,
    UpdateSessionHandler,
    UpdateSessionResponse,
};

pub use domain::{
    ContextBuilder, Emotion, EmotionAnalyzer, Message, MessageId, MessageRole, Session, SessionId,
};

pub use infrastructure::{
    DynamicLLMAdapter, DynamicLLMConfig, FileMessageRepository, FileSessionRepository,
    InMemoryMessageRepository, InMemorySessionRepository, LLMAdapterRegistry, MockLLMAdapter,
    OpenAIAdapter,
};

pub use ports::{
    CompletionRequest, CompletionResponse, FinishReason, HealthStatus, LLMChatMessage, LLMError,
    LLMPort, LLMProviderConfig, MessageRepository, ModelInfo, PaginatedResult, Pagination,
    ProviderInfo, ProviderType, RepositoryError, SessionRepository, StreamChunk, TokenUsage,
};

use std::sync::Arc;

/// Chat 模块容器
///
/// 管理模块内的依赖注入
pub struct ChatModule {
    // Repositories
    session_repository: Arc<dyn SessionRepository>,
    message_repository: Arc<dyn MessageRepository>,
    // LLM
    llm_registry: Arc<LLMAdapterRegistry>,
    // Handlers
    create_session_handler: CreateSessionHandler,
    delete_session_handler: DeleteSessionHandler,
    update_session_handler: UpdateSessionHandler,
    get_session_handler: GetSessionHandler,
    list_sessions_handler: ListSessionsHandler,
    list_messages_handler: ListMessagesHandler,
}

impl ChatModule {
    /// 创建新的 ChatModule 实例（内存存储，用于开发测试）
    ///
    /// # Arguments
    /// * `llm_registry` - LLM 适配器注册表
    pub fn new(llm_registry: Arc<LLMAdapterRegistry>) -> Self {
        // 创建仓储（使用内存实现，适用于开发和测试）
        let session_repository: Arc<dyn SessionRepository> =
            Arc::new(InMemorySessionRepository::new());
        let message_repository: Arc<dyn MessageRepository> =
            Arc::new(InMemoryMessageRepository::new());

        Self::with_repositories(session_repository, message_repository, llm_registry)
    }

    /// 创建带持久化存储的 ChatModule 实例（生产环境推荐）
    ///
    /// # Arguments
    /// * `data_dir` - 应用数据目录路径
    /// * `llm_registry` - LLM 适配器注册表
    ///
    /// # Errors
    /// 如果无法初始化文件存储，返回错误
    pub async fn new_with_persistence(
        data_dir: std::path::PathBuf,
        llm_registry: Arc<LLMAdapterRegistry>,
    ) -> Result<Self, RepositoryError> {
        // 创建持久化仓储
        let session_repository: Arc<dyn SessionRepository> =
            Arc::new(FileSessionRepository::new(data_dir.clone()).await?);
        let message_repository: Arc<dyn MessageRepository> =
            Arc::new(FileMessageRepository::new(data_dir).await?);

        Ok(Self::with_repositories(
            session_repository,
            message_repository,
            llm_registry,
        ))
    }

    /// 使用自定义仓储创建 ChatModule
    pub fn with_repositories(
        session_repository: Arc<dyn SessionRepository>,
        message_repository: Arc<dyn MessageRepository>,
        llm_registry: Arc<LLMAdapterRegistry>,
    ) -> Self {
        let create_session_handler = CreateSessionHandler::new(session_repository.clone());
        let delete_session_handler =
            DeleteSessionHandler::new(session_repository.clone(), message_repository.clone());
        let update_session_handler = UpdateSessionHandler::new(session_repository.clone());
        let get_session_handler = GetSessionHandler::new(session_repository.clone());
        let list_sessions_handler = ListSessionsHandler::new(session_repository.clone());
        let list_messages_handler = ListMessagesHandler::new(message_repository.clone());

        Self {
            session_repository,
            message_repository,
            llm_registry,
            create_session_handler,
            delete_session_handler,
            update_session_handler,
            get_session_handler,
            list_sessions_handler,
            list_messages_handler,
        }
    }

    // Command handlers

    /// 创建会话
    pub async fn create_session(
        &self,
        command: CreateSessionCommand,
    ) -> Result<CreateSessionResponse, ApplicationError> {
        self.create_session_handler.handle(command).await
    }

    /// 删除会话
    pub async fn delete_session(
        &self,
        command: DeleteSessionCommand,
    ) -> Result<DeleteSessionResponse, ApplicationError> {
        self.delete_session_handler.handle(command).await
    }

    /// 更新会话
    pub async fn update_session(
        &self,
        command: UpdateSessionCommand,
    ) -> Result<UpdateSessionResponse, ApplicationError> {
        self.update_session_handler.handle(command).await
    }

    /// 发送消息（创建临时处理器）
    pub async fn send_message(
        &self,
        command: SendMessageCommand,
        provider_id: &str,
    ) -> Result<SendMessageResponse, ApplicationError> {
        let llm = self.llm_registry.get(provider_id).ok_or_else(|| {
            ApplicationError::LLMError(LLMError::ProviderNotAvailable(provider_id.to_string()))
        })?;

        let default_model = self
            .llm_registry
            .get_default_model(provider_id)
            .unwrap_or_else(|| "gpt-3.5-turbo".to_string());

        let handler = SendMessageHandler::new(
            self.session_repository.clone(),
            self.message_repository.clone(),
            llm,
            default_model,
        );

        handler.handle(command).await
    }

    /// 发送消息（流式）
    pub async fn send_message_stream(
        &self,
        command: SendMessageCommand,
        provider_id: &str,
    ) -> Result<
        (
            SendMessageResponse,
            tokio::sync::mpsc::Receiver<StreamEvent>,
        ),
        ApplicationError,
    > {
        let llm = self.llm_registry.get(provider_id).ok_or_else(|| {
            ApplicationError::LLMError(LLMError::ProviderNotAvailable(provider_id.to_string()))
        })?;

        let default_model = self
            .llm_registry
            .get_default_model(provider_id)
            .unwrap_or_else(|| "gpt-3.5-turbo".to_string());

        let handler = SendMessageHandler::new(
            self.session_repository.clone(),
            self.message_repository.clone(),
            llm,
            default_model,
        );

        handler.handle_stream(command).await
    }

    /// 重新生成消息（流式，不保存用户消息）
    pub async fn regenerate_stream(
        &self,
        command: RegenerateCommand,
        provider_id: &str,
    ) -> Result<
        (
            RegenerateResponse,
            tokio::sync::mpsc::Receiver<StreamEvent>,
        ),
        ApplicationError,
    > {
        let llm = self.llm_registry.get(provider_id).ok_or_else(|| {
            ApplicationError::LLMError(LLMError::ProviderNotAvailable(provider_id.to_string()))
        })?;

        let default_model = self
            .llm_registry
            .get_default_model(provider_id)
            .unwrap_or_else(|| "gpt-3.5-turbo".to_string());

        let handler = RegenerateHandler::new(
            self.session_repository.clone(),
            self.message_repository.clone(),
            llm,
            default_model,
        );

        handler.handle_stream(command).await
    }

    // Query handlers

    /// 获取会话
    pub async fn get_session(
        &self,
        query: GetSessionQuery,
    ) -> Result<GetSessionResponse, ApplicationError> {
        self.get_session_handler.handle(query).await
    }

    /// 列出所有会话
    pub async fn list_sessions(
        &self,
        query: ListSessionsQuery,
    ) -> Result<ListSessionsResponse, ApplicationError> {
        self.list_sessions_handler.handle(query).await
    }

    /// 列出会话消息
    pub async fn list_messages(
        &self,
        query: ListMessagesQuery,
    ) -> Result<ListMessagesResponse, ApplicationError> {
        self.list_messages_handler.handle(query).await
    }

    // Accessors

    /// 获取 LLM 注册表
    pub fn llm_registry(&self) -> &Arc<LLMAdapterRegistry> {
        &self.llm_registry
    }

    /// 获取会话仓储
    pub fn session_repository(&self) -> &Arc<dyn SessionRepository> {
        &self.session_repository
    }

    /// 获取消息仓储
    pub fn message_repository(&self) -> &Arc<dyn MessageRepository> {
        &self.message_repository
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_chat_module_integration() {
        let registry = Arc::new(LLMAdapterRegistry::new());
        let module = ChatModule::new(registry);

        // 创建会话
        let create_cmd = CreateSessionCommand::new(Some("Integration Test".to_string()), None);
        let create_resp = module.create_session(create_cmd).await.unwrap();

        assert_eq!(create_resp.session.title(), "Integration Test");

        // 获取会话
        let get_query = GetSessionQuery::new(create_resp.session.id());
        let get_resp = module.get_session(get_query).await.unwrap();

        assert!(get_resp.session.is_some());

        // 列出会话
        let list_query = ListSessionsQuery::default();
        let list_resp = module.list_sessions(list_query).await.unwrap();

        assert_eq!(list_resp.total, 1);

        // 删除会话
        let delete_cmd = DeleteSessionCommand::new(create_resp.session.id());
        let delete_resp = module.delete_session(delete_cmd).await.unwrap();

        assert_eq!(delete_resp.deleted_messages, 0);

        // 确认已删除
        let list_query = ListSessionsQuery::default();
        let list_resp = module.list_sessions(list_query).await.unwrap();

        assert_eq!(list_resp.total, 0);
    }
}
