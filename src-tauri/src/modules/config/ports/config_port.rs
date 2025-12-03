// Config Port
//
// 配置服务端口定义

use async_trait::async_trait;
use thiserror::Error;

use crate::modules::config::domain::{AppConfig, PartialAppConfig};

/// 配置错误类型
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration not found: {0}")]
    NotFound(String),

    #[error("Invalid configuration: {0}")]
    Invalid(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Validation error: {errors:?}")]
    ValidationError { errors: Vec<String> },
}

impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        ConfigError::SerializationError(err.to_string())
    }
}

/// 配置端口 - 定义配置的读写操作
#[async_trait]
pub trait ConfigPort: Send + Sync {
    /// 获取完整配置
    async fn get_all(&self) -> Result<AppConfig, ConfigError>;

    /// 获取指定键的配置值
    async fn get<T: serde::de::DeserializeOwned + Send>(
        &self,
        key: &str,
    ) -> Result<Option<T>, ConfigError>;

    /// 设置指定键的配置值
    async fn set<T: serde::Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
    ) -> Result<(), ConfigError>;

    /// 更新部分配置
    async fn update(&self, partial: PartialAppConfig) -> Result<AppConfig, ConfigError>;

    /// 删除指定键的配置
    async fn delete(&self, key: &str) -> Result<(), ConfigError>;

    /// 重置为默认配置
    async fn reset(&self) -> Result<AppConfig, ConfigError>;

    /// 检查配置是否存在
    async fn exists(&self, key: &str) -> Result<bool, ConfigError>;
}

/// 配置观察者 - 用于监听配置变化
pub trait ConfigObserver: Send + Sync {
    /// 配置变化时调用
    fn on_config_changed(&self, key: &str, new_value: &serde_json::Value);
}
