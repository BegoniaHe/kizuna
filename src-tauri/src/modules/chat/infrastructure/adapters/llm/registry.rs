use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::modules::chat::ports::{LLMError, LLMPort, LLMProviderConfig, ProviderType};

use super::{ClaudeAdapter, OllamaAdapter, OpenAIAdapter};

/// LLM 适配器注册表
///
/// 管理所有 LLM 提供商适配器的创建和缓存
pub struct LLMAdapterRegistry {
    /// 已创建的适配器实例缓存
    instances: RwLock<HashMap<String, Arc<dyn LLMPort>>>,
    /// 提供商配置
    configs: RwLock<HashMap<String, LLMProviderConfig>>,
}

impl LLMAdapterRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        Self {
            instances: RwLock::new(HashMap::new()),
            configs: RwLock::new(HashMap::new()),
        }
    }

    /// 注册提供商配置
    pub async fn register(&self, config: LLMProviderConfig) -> Result<(), LLMError> {
        let adapter = self.create_adapter(&config)?;
        let adapter: Arc<dyn LLMPort> = Arc::from(adapter);
        let id = config.id.clone();

        {
            let mut instances = self.instances.write().await;
            instances.insert(id.clone(), adapter);
        }
        {
            let mut configs = self.configs.write().await;
            configs.insert(id, config);
        }

        Ok(())
    }

    /// 获取适配器（同步访问缓存）
    pub fn get(&self, provider_id: &str) -> Option<Arc<dyn LLMPort>> {
        // 使用 try_read 进行非阻塞访问
        if let Ok(instances) = self.instances.try_read() {
            instances.get(provider_id).cloned()
        } else {
            None
        }
    }

    /// 获取适配器（异步）
    pub async fn get_async(&self, provider_id: &str) -> Option<Arc<dyn LLMPort>> {
        let instances = self.instances.read().await;
        instances.get(provider_id).cloned()
    }

    /// 获取默认模型
    pub fn get_default_model(&self, provider_id: &str) -> Option<String> {
        if let Ok(configs) = self.configs.try_read() {
            configs.get(provider_id).map(|c| c.default_model.clone())
        } else {
            None
        }
    }

    /// 获取或创建适配器实例
    pub async fn get_or_create(
        &self,
        config: &LLMProviderConfig,
    ) -> Result<Arc<dyn LLMPort>, LLMError> {
        // 检查缓存
        {
            let instances = self.instances.read().await;
            if let Some(instance) = instances.get(&config.id) {
                return Ok(instance.clone());
            }
        }

        // 创建新实例
        let adapter = self.create_adapter(config)?;
        let adapter: Arc<dyn LLMPort> = Arc::from(adapter);

        // 缓存
        {
            let mut instances = self.instances.write().await;
            instances.insert(config.id.clone(), adapter.clone());
        }
        {
            let mut configs = self.configs.write().await;
            configs.insert(config.id.clone(), config.clone());
        }

        Ok(adapter)
    }

    /// 根据配置创建适配器
    fn create_adapter(&self, config: &LLMProviderConfig) -> Result<Box<dyn LLMPort>, LLMError> {
        match config.provider_type {
            ProviderType::OpenAI => Ok(Box::new(OpenAIAdapter::new(config.clone())?)),
            ProviderType::Claude => Ok(Box::new(ClaudeAdapter::new(config.clone())?)),
            ProviderType::Ollama => Ok(Box::new(OllamaAdapter::new(config.clone())?)),
            ProviderType::Custom => {
                // 自定义提供商使用与 OpenAI 兼容的 API
                Ok(Box::new(OpenAIAdapter::new(config.clone())?))
            }
        }
    }

    /// 清除指定提供商的缓存
    pub async fn invalidate(&self, provider_id: &str) {
        {
            let mut instances = self.instances.write().await;
            instances.remove(provider_id);
        }
        {
            let mut configs = self.configs.write().await;
            configs.remove(provider_id);
        }
    }

    /// 清除所有缓存
    pub async fn invalidate_all(&self) {
        {
            let mut instances = self.instances.write().await;
            instances.clear();
        }
        {
            let mut configs = self.configs.write().await;
            configs.clear();
        }
    }

    /// 获取已注册的提供商数量
    pub async fn count(&self) -> usize {
        let instances = self.instances.read().await;
        instances.len()
    }

    /// 列出所有提供商 ID
    pub async fn list_providers(&self) -> Vec<String> {
        let configs = self.configs.read().await;
        configs.keys().cloned().collect()
    }
}

impl Default for LLMAdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_caching() {
        let registry = LLMAdapterRegistry::new();
        let config = LLMProviderConfig {
            id: "test".to_string(),
            name: "Test".to_string(),
            provider_type: ProviderType::OpenAI,
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: "test-key".to_string(),
            default_model: "gpt-3.5-turbo".to_string(),
            timeout_secs: 60,
            max_retries: 3,
        };

        // 第一次获取
        let adapter1 = registry.get_or_create(&config).await.unwrap();
        // 第二次获取（应该返回缓存的实例）
        let adapter2 = registry.get_or_create(&config).await.unwrap();

        // 验证是同一个实例（通过 provider_id）
        assert_eq!(adapter1.provider_id(), adapter2.provider_id());
        assert_eq!(registry.count().await, 1);
    }

    #[tokio::test]
    async fn test_invalidate() {
        let registry = LLMAdapterRegistry::new();
        let config = LLMProviderConfig {
            id: "test".to_string(),
            provider_type: ProviderType::OpenAI,
            ..Default::default()
        };

        let _ = registry.get_or_create(&config).await.unwrap();
        assert_eq!(registry.count().await, 1);

        registry.invalidate("test").await;
        assert_eq!(registry.count().await, 0);
    }
}
