// Config Repository Port
//
// 配置存储仓储端口定义

use async_trait::async_trait;

use super::ConfigError;
use crate::modules::config::domain::AppConfig;

/// 配置仓储端口 - 定义配置持久化抽象
#[async_trait]
pub trait ConfigRepository: Send + Sync {
    /// 加载配置
    async fn load(&self) -> Result<AppConfig, ConfigError>;

    /// 保存配置
    async fn save(&self, config: &AppConfig) -> Result<(), ConfigError>;

    /// 清除配置
    async fn clear(&self) -> Result<(), ConfigError>;

    /// 检查配置是否存在
    async fn exists(&self) -> Result<bool, ConfigError>;

    /// 获取单个配置项
    async fn get_value(&self, key: &str) -> Result<Option<serde_json::Value>, ConfigError>;

    /// 设置单个配置项
    async fn set_value(&self, key: &str, value: serde_json::Value) -> Result<(), ConfigError>;

    /// 删除单个配置项
    async fn delete_value(&self, key: &str) -> Result<(), ConfigError>;
}
