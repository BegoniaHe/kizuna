use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::shared::Preset;

/// 窗口配置
#[derive(Debug, Clone)]
pub struct WindowSettings {
    pub pet_mode_width: u32,
    pub pet_mode_height: u32,
    pub normal_mode_width: u32,
    pub normal_mode_height: u32,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            pet_mode_width: 300,
            pet_mode_height: 400,
            normal_mode_width: 1200,
            normal_mode_height: 800,
        }
    }
}

/// 应用全局状态
///
/// 注意:会话(sessions)和消息(messages)现在由 ChatModule 管理
/// 这里只保留应用级别的状态
pub struct AppState {
    /// 预设配置(暂时保留,后续可迁移到 ConfigModule)
    pub presets: Arc<RwLock<HashMap<Uuid, Preset>>>,
    /// 活跃的消息生成状态(用于取消操作)
    pub active_generations: Arc<RwLock<HashMap<Uuid, bool>>>,
    /// 窗口设置
    pub window_settings: Arc<RwLock<WindowSettings>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            presets: Arc::new(RwLock::new(HashMap::new())),
            active_generations: Arc::new(RwLock::new(HashMap::new())),
            window_settings: Arc::new(RwLock::new(WindowSettings::default())),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
