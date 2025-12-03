// Store-based Config Repository
//
// 基于 Tauri Store 插件的配置仓储实现

use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::modules::config::domain::AppConfig;
use crate::modules::config::ports::{ConfigError, ConfigRepository};

const CONFIG_FILE_NAME: &str = "config.json";
#[allow(dead_code)]
const CONFIG_KEY: &str = "app_config";

/// Tauri Store 配置仓储
///
/// 使用 tauri-plugin-store 持久化配置
pub struct StoreConfigRepository {
    /// 配置文件路径
    config_path: PathBuf,
    /// 内存缓存
    cache: Arc<RwLock<Option<AppConfig>>>,
}

impl StoreConfigRepository {
    /// 创建新的 Store 配置仓储
    ///
    /// # Arguments
    /// * `app_data_dir` - 应用数据目录
    pub fn new(app_data_dir: PathBuf) -> Self {
        Self {
            config_path: app_data_dir.join(CONFIG_FILE_NAME),
            cache: Arc::new(RwLock::new(None)),
        }
    }

    /// 从文件加载配置
    async fn load_from_file(&self) -> Result<Option<AppConfig>, ConfigError> {
        if !self.config_path.exists() {
            return Ok(None);
        }

        let content = tokio::fs::read_to_string(&self.config_path)
            .await
            .map_err(|e| ConfigError::StorageError(e.to_string()))?;

        let config: AppConfig = serde_json::from_str(&content)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))?;

        Ok(Some(config))
    }

    /// 保存配置到文件
    async fn save_to_file(&self, config: &AppConfig) -> Result<(), ConfigError> {
        // 确保目录存在
        if let Some(parent) = self.config_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| ConfigError::StorageError(e.to_string()))?;
        }

        let content = serde_json::to_string_pretty(config)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))?;

        tokio::fs::write(&self.config_path, content)
            .await
            .map_err(|e| ConfigError::StorageError(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl ConfigRepository for StoreConfigRepository {
    async fn load(&self) -> Result<AppConfig, ConfigError> {
        // 先检查缓存
        {
            let cache = self.cache.read().await;
            if let Some(ref config) = *cache {
                return Ok(config.clone());
            }
        }

        // 从文件加载
        let config = self.load_from_file().await?.unwrap_or_default();

        // 更新缓存
        {
            let mut cache = self.cache.write().await;
            *cache = Some(config.clone());
        }

        Ok(config)
    }

    async fn save(&self, config: &AppConfig) -> Result<(), ConfigError> {
        // 保存到文件
        self.save_to_file(config).await?;

        // 更新缓存
        {
            let mut cache = self.cache.write().await;
            *cache = Some(config.clone());
        }

        Ok(())
    }

    async fn clear(&self) -> Result<(), ConfigError> {
        // 删除文件
        if self.config_path.exists() {
            tokio::fs::remove_file(&self.config_path)
                .await
                .map_err(|e| ConfigError::StorageError(e.to_string()))?;
        }

        // 清除缓存
        {
            let mut cache = self.cache.write().await;
            *cache = None;
        }

        Ok(())
    }

    async fn exists(&self) -> Result<bool, ConfigError> {
        Ok(self.config_path.exists())
    }

    async fn get_value(&self, key: &str) -> Result<Option<serde_json::Value>, ConfigError> {
        let config = self.load().await?;
        let config_json = serde_json::to_value(&config)?;

        // 支持点分隔的路径
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
        let mut config = self.load().await?;
        let mut config_json = serde_json::to_value(&config)?;

        // 支持点分隔的路径
        let parts: Vec<&str> = key.split('.').collect();
        set_nested_value(&mut config_json, &parts, value)?;

        // 转换回 AppConfig
        config = serde_json::from_value(config_json)?;
        self.save(&config).await?;

        Ok(())
    }

    async fn delete_value(&self, key: &str) -> Result<(), ConfigError> {
        let mut config = self.load().await?;
        let mut config_json = serde_json::to_value(&config)?;

        // 支持点分隔的路径
        let parts: Vec<&str> = key.split('.').collect();
        delete_nested_value(&mut config_json, &parts)?;

        // 转换回 AppConfig
        config = serde_json::from_value(config_json)?;
        self.save(&config).await?;

        Ok(())
    }
}

/// 设置嵌套的 JSON 值
fn set_nested_value(
    json: &mut serde_json::Value,
    parts: &[&str],
    value: serde_json::Value,
) -> Result<(), ConfigError> {
    if parts.is_empty() {
        return Err(ConfigError::Invalid("Empty key path".to_string()));
    }

    let mut current = json;

    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // 最后一个部分，设置值
            if let Some(obj) = current.as_object_mut() {
                obj.insert((*part).to_string(), value);
                return Ok(());
            } else {
                return Err(ConfigError::Invalid(format!(
                    "Cannot set value at path: {}",
                    parts.join(".")
                )));
            }
        } else {
            // 中间部分，导航
            current = current
                .get_mut(*part)
                .ok_or_else(|| ConfigError::NotFound(parts.join(".")))?;
        }
    }

    Ok(())
}

/// 删除嵌套的 JSON 值
fn delete_nested_value(json: &mut serde_json::Value, parts: &[&str]) -> Result<(), ConfigError> {
    if parts.is_empty() {
        return Err(ConfigError::Invalid("Empty key path".to_string()));
    }

    let mut current = json;

    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // 最后一个部分，删除值
            if let Some(obj) = current.as_object_mut() {
                obj.remove(*part);
                return Ok(());
            } else {
                return Err(ConfigError::Invalid(format!(
                    "Cannot delete value at path: {}",
                    parts.join(".")
                )));
            }
        } else {
            // 中间部分，导航
            current = current
                .get_mut(*part)
                .ok_or_else(|| ConfigError::NotFound(parts.join(".")))?;
        }
    }

    Ok(())
}
