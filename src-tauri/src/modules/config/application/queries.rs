// Config Queries
//
// 配置相关的查询处理器

use async_trait::async_trait;
use std::sync::Arc;

use crate::modules::config::domain::AppConfig;
use crate::modules::config::ports::{ConfigError, ConfigRepository};

/// 查询处理器 trait
#[async_trait]
pub trait QueryHandler<Q> {
    type Output;
    type Error;

    async fn handle(&self, query: Q) -> Result<Self::Output, Self::Error>;
}

// ============================================================================
// Get All Config Query
// ============================================================================

/// 获取全部配置查询
#[derive(Debug, Clone, Default)]
pub struct GetAllConfigQuery;

/// 获取全部配置响应
#[derive(Debug, Clone)]
pub struct GetAllConfigResponse {
    pub config: AppConfig,
}

/// 获取全部配置查询处理器
pub struct GetAllConfigHandler {
    repository: Arc<dyn ConfigRepository>,
}

impl GetAllConfigHandler {
    pub fn new(repository: Arc<dyn ConfigRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl QueryHandler<GetAllConfigQuery> for GetAllConfigHandler {
    type Output = GetAllConfigResponse;
    type Error = ConfigError;

    async fn handle(&self, _query: GetAllConfigQuery) -> Result<Self::Output, Self::Error> {
        let config = self.repository.load().await?;
        Ok(GetAllConfigResponse { config })
    }
}

// ============================================================================
// Get Config Value Query
// ============================================================================

/// 获取配置值查询
#[derive(Debug, Clone)]
pub struct GetConfigValueQuery {
    pub key: String,
}

impl GetConfigValueQuery {
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into() }
    }
}

/// 获取配置值响应
#[derive(Debug, Clone)]
pub struct GetConfigValueResponse {
    pub value: Option<serde_json::Value>,
}

/// 获取配置值查询处理器
pub struct GetConfigValueHandler {
    repository: Arc<dyn ConfigRepository>,
}

impl GetConfigValueHandler {
    pub fn new(repository: Arc<dyn ConfigRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl QueryHandler<GetConfigValueQuery> for GetConfigValueHandler {
    type Output = GetConfigValueResponse;
    type Error = ConfigError;

    async fn handle(&self, query: GetConfigValueQuery) -> Result<Self::Output, Self::Error> {
        let value = self.repository.get_value(&query.key).await?;
        Ok(GetConfigValueResponse { value })
    }
}

// ============================================================================
// Check Config Exists Query
// ============================================================================

/// 检查配置是否存在查询
#[derive(Debug, Clone)]
pub struct ConfigExistsQuery {
    pub key: String,
}

impl ConfigExistsQuery {
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into() }
    }
}

/// 检查配置是否存在响应
#[derive(Debug, Clone)]
pub struct ConfigExistsResponse {
    pub exists: bool,
}

/// 检查配置是否存在查询处理器
pub struct ConfigExistsHandler {
    repository: Arc<dyn ConfigRepository>,
}

impl ConfigExistsHandler {
    pub fn new(repository: Arc<dyn ConfigRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl QueryHandler<ConfigExistsQuery> for ConfigExistsHandler {
    type Output = ConfigExistsResponse;
    type Error = ConfigError;

    async fn handle(&self, query: ConfigExistsQuery) -> Result<Self::Output, Self::Error> {
        let value = self.repository.get_value(&query.key).await?;
        Ok(ConfigExistsResponse {
            exists: value.is_some(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::config::infrastructure::InMemoryConfigRepository;

    #[tokio::test]
    async fn test_get_all_config() {
        let repo = Arc::new(InMemoryConfigRepository::new());
        let handler = GetAllConfigHandler::new(repo);

        let response = handler.handle(GetAllConfigQuery).await.unwrap();
        assert_eq!(response.config.general.language.code(), "zh-CN");
    }

    #[tokio::test]
    async fn test_get_config_value() {
        let repo = Arc::new(InMemoryConfigRepository::new());
        let handler = GetConfigValueHandler::new(repo);

        let response = handler
            .handle(GetConfigValueQuery::new("general.theme"))
            .await
            .unwrap();

        assert!(response.value.is_some());
    }

    #[tokio::test]
    async fn test_config_exists() {
        let repo = Arc::new(InMemoryConfigRepository::new());
        let handler = ConfigExistsHandler::new(repo);

        let response = handler
            .handle(ConfigExistsQuery::new("general.theme"))
            .await
            .unwrap();

        assert!(response.exists);

        let response = handler
            .handle(ConfigExistsQuery::new("nonexistent.key"))
            .await
            .unwrap();

        assert!(!response.exists);
    }
}
