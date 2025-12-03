pub mod commands;
pub mod infrastructure;
pub mod modules;
pub mod shared;

use std::sync::Arc;
use tauri::Manager;
use tokio::sync::RwLock;

use infrastructure::{AppState, EventBus};
use modules::chat::LLMAdapterRegistry;
use modules::{ChatModule, ConfigModule, WindowModule};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(true)
        .init();

    tracing::info!("Kizuna starting...");

    let app_state = AppState::new();
    let event_bus = Arc::new(RwLock::new(EventBus::new()));

    // 初始化 LLM 适配器注册表
    let llm_registry = Arc::new(LLMAdapterRegistry::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_websocket::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .manage(app_state)
        .manage(event_bus.clone())
        .manage(llm_registry.clone())
        .setup(move |app| {
            let handle = app.handle().clone();
            let event_bus_clone = event_bus.clone();
            let llm_registry_clone = llm_registry.clone();

            // 获取应用数据目录
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");

            tracing::info!("App data directory: {:?}", app_data_dir);

            // 初始化 Chat 模块（使用持久化存储）
            let chat_module = tauri::async_runtime::block_on(async {
                match ChatModule::new_with_persistence(app_data_dir.clone(), llm_registry_clone)
                    .await
                {
                    Ok(module) => {
                        tracing::info!("Chat module initialized with persistent storage");
                        Arc::new(RwLock::new(module))
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to initialize persistent storage: {}, falling back to memory",
                            e
                        );
                        Arc::new(RwLock::new(ChatModule::new(llm_registry.clone())))
                    }
                }
            });
            app.manage(chat_module);

            // 初始化 Config 模块（使用文件存储）
            let config_module = Arc::new(RwLock::new(ConfigModule::new_with_store(app_data_dir)));
            app.manage(config_module);

            // 初始化 Window 模块
            let window_module = Arc::new(WindowModule::new(handle.clone()));
            app.manage(window_module);

            // 设置 EventBus 的 AppHandle
            tauri::async_runtime::spawn(async move {
                let mut bus = event_bus_clone.write().await;
                bus.set_app_handle(handle);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Session commands
            commands::session_create,
            commands::session_list,
            commands::session_get,
            commands::session_delete,
            commands::session_rename,
            // Chat commands
            commands::chat_send_message,
            commands::chat_regenerate,
            commands::chat_stop_generation,
            commands::chat_get_messages,
            commands::chat_fetch_models,
            // Window commands
            commands::window_toggle_pet_mode,
            commands::window_set_always_on_top,
            commands::window_start_dragging,
            commands::window_create,
            commands::window_list,
            commands::window_close,
            // Config commands
            commands::config_get_all,
            commands::config_reset,
            commands::preset_list,
            commands::preset_create,
            commands::preset_delete,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
