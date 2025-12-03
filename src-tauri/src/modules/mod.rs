// Modules Layer - 业务模块
//
// 按照六边形架构组织的业务模块：
// - chat: 聊天模块，处理消息和会话
// - config: 配置模块，处理应用设置
// - tray: 系统托盘模块
// - window: 窗口管理模块

pub mod chat;
pub mod config;
pub mod tray;
pub mod window;

pub use chat::ChatModule;
pub use config::ConfigModule;
pub use tray::TrayModule;
pub use window::WindowModule;
