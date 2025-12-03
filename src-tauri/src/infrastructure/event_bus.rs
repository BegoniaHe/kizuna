use tauri::{AppHandle, Emitter};
use tokio::sync::broadcast;

use crate::shared::{Emotion, MessageChunk, WindowMode};

#[derive(Clone, Debug)]
pub enum AppEvent {
    MessageChunk(MessageChunk),
    MessageComplete {
        session_id: uuid::Uuid,
        message_id: uuid::Uuid,
        emotion: Option<Emotion>,
    },
    MessageError {
        session_id: uuid::Uuid,
        error: String,
    },
    WindowModeChanged {
        mode: WindowMode,
    },
}

pub struct EventBus {
    sender: broadcast::Sender<AppEvent>,
    app_handle: Option<AppHandle>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);
        Self {
            sender,
            app_handle: None,
        }
    }

    pub fn set_app_handle(&mut self, handle: AppHandle) {
        self.app_handle = Some(handle);
    }

    pub fn publish(&self, event: AppEvent) {
        tracing::debug!("[EventBus] Publishing event: {:?}", event);
        let _ = self.sender.send(event.clone());

        if let Some(handle) = &self.app_handle {
            match &event {
                AppEvent::MessageChunk(chunk) => {
                    tracing::debug!("[EventBus] Emitting llm:chunk to frontend");
                    let _ = handle.emit("llm:chunk", chunk);
                }
                AppEvent::MessageComplete {
                    session_id,
                    message_id,
                    emotion,
                } => {
                    tracing::info!("[EventBus] Emitting llm:complete to frontend");
                    let _ = handle.emit(
                        "llm:complete",
                        serde_json::json!({
                            "sessionId": session_id,
                            "messageId": message_id,
                            "emotion": emotion,
                        }),
                    );
                }
                AppEvent::MessageError { session_id, error } => {
                    tracing::error!("[EventBus] Emitting llm:error to frontend: {}", error);
                    let _ = handle.emit(
                        "llm:error",
                        serde_json::json!({
                            "sessionId": session_id,
                            "error": error,
                        }),
                    );
                }
                AppEvent::WindowModeChanged { mode } => {
                    tracing::info!("[EventBus] Emitting window:mode_changed");
                    let _ = handle.emit(
                        "window:mode_changed",
                        serde_json::json!({
                            "mode": mode,
                        }),
                    );
                }
            }
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AppEvent> {
        self.sender.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
