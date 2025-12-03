// In-Memory Config Repository
//
// 基于内存的配置仓储实现（用于测试和开发）

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::modules::config::domain::AppConfig;
use crate::modules::config::ports::{ConfigError, ConfigRepository};

/// 内存配置仓储
pub struct InMemoryConfigRepository {
    config: Arc<RwLock<AppConfig>>,
    values: Arc<RwLock<HashMap<String, serde_json::Value>>>,
}

impl InMemoryConfigRepository {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(AppConfig::default())),
            values: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_config(config: AppConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            values: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryConfigRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConfigRepository for InMemoryConfigRepository {
    async fn load(&self) -> Result<AppConfig, ConfigError> {
        let config = self.config.read().await;
        Ok(config.clone())
    }

    async fn save(&self, config: &AppConfig) -> Result<(), ConfigError> {
        let mut current = self.config.write().await;
        *current = config.clone();
        Ok(())
    }

    async fn clear(&self) -> Result<(), ConfigError> {
        let mut config = self.config.write().await;
        *config = AppConfig::default();

        let mut values = self.values.write().await;
        values.clear();

        Ok(())
    }

    async fn exists(&self) -> Result<bool, ConfigError> {
        // 内存仓储总是存在
        Ok(true)
    }

    async fn get_value(&self, key: &str) -> Result<Option<serde_json::Value>, ConfigError> {
        // 先检查独立存储的值
        {
            let values = self.values.read().await;
            if let Some(value) = values.get(key) {
                return Ok(Some(value.clone()));
            }
        }

        // 从配置中提取值
        let config = self.config.read().await;
        let config_json = serde_json::to_value(&*config)?;

        // 支持点分隔的路径，如 "general.theme"
        let parts: Vec<&str> = key.split('.').collect();
        let mut current = &config_json;

        for part in parts {
            match current.get(part) {
                Some(v) => current = v,
                None => return Ok(None),
            }
        }

        Ok(Some(current.clone()))
    }

    async fn set_value(&self, key: &str, value: serde_json::Value) -> Result<(), ConfigError> {
        let mut values = self.values.write().await;
        values.insert(key.to_string(), value);
        Ok(())
    }

    async fn delete_value(&self, key: &str) -> Result<(), ConfigError> {
        let mut values = self.values.write().await;
        values.remove(key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_default_config() {
        let repo = InMemoryConfigRepository::new();
        let config = repo.load().await.unwrap();
        assert_eq!(config.general.language.code(), "zh-CN");
    }

    #[tokio::test]
    async fn test_save_and_load_config() {
        let repo = InMemoryConfigRepository::new();
        let mut config = AppConfig::default();
        config.general.auto_start = true;

        repo.save(&config).await.unwrap();
        let loaded = repo.load().await.unwrap();

        assert!(loaded.general.auto_start);
    }

    #[tokio::test]
    async fn test_get_nested_value() {
        let repo = InMemoryConfigRepository::new();
        let value = repo.get_value("general.theme").await.unwrap();
        assert!(value.is_some());
    }

    #[tokio::test]
    async fn test_clear_config() {
        let repo = InMemoryConfigRepository::new();
        let mut config = AppConfig::default();
        config.general.auto_start = true;
        repo.save(&config).await.unwrap();

        repo.clear().await.unwrap();
        let loaded = repo.load().await.unwrap();

        assert!(!loaded.general.auto_start);
    }
}
