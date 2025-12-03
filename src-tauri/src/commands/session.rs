// Session Commands - 完全重构版本
//
// 使用 ChatModule 的 CQRS 命令和查询处理会话操作

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::modules::chat::{
    ChatModule, CreateSessionCommand, DeleteSessionCommand, GetSessionQuery, ListSessionsQuery,
    SessionId, UpdateSessionCommand,
};
use crate::shared::{AppError, AppResult, Session};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionRequest {
    pub preset_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListSessionsRequest {
    pub page: u32,
    pub limit: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListSessionsResponse {
    pub sessions: Vec<Session>,
    pub total: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSessionRequest {
    pub id: Uuid,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSessionRequest {
    pub id: Uuid,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameSessionRequest {
    pub id: Uuid,
    pub title: String,
}

/// 创建会话 - 使用 ChatModule
#[tauri::command]
pub async fn session_create(
    chat_module: State<'_, Arc<RwLock<ChatModule>>>,
    request: CreateSessionRequest,
) -> AppResult<Session> {
    let module = chat_module.read().await;

    let command = CreateSessionCommand::new(
        Some("New Chat".to_string()),
        request.preset_id.map(|id| id.into()),
    );

    let response = module
        .create_session(command)
        .await
        .map_err(|e| AppError::Unknown(e.to_string()))?;

    let domain_session = response.session;

    Ok(Session {
        id: domain_session.id().into(),
        title: domain_session.title().to_string(),
        model_config: None,
        preset_id: domain_session.preset_id().map(|id| id.into()),
        created_at: domain_session.created_at(),
        updated_at: domain_session.updated_at(),
    })
}

/// 列出会话 - 使用 ChatModule
#[tauri::command]
pub async fn session_list(
    chat_module: State<'_, Arc<RwLock<ChatModule>>>,
    request: ListSessionsRequest,
) -> AppResult<ListSessionsResponse> {
    let module = chat_module.read().await;

    let query = ListSessionsQuery::new(request.page, request.limit);

    let response = module
        .list_sessions(query)
        .await
        .map_err(|e| AppError::Unknown(e.to_string()))?;

    let sessions: Vec<Session> = response
        .sessions
        .into_iter()
        .map(|s| Session {
            id: s.id().into(),
            title: s.title().to_string(),
            model_config: None,
            preset_id: s.preset_id().map(|id| id.into()),
            created_at: s.created_at(),
            updated_at: s.updated_at(),
        })
        .collect();

    Ok(ListSessionsResponse {
        sessions,
        total: response.total,
    })
}

/// 获取会话 - 使用 ChatModule
#[tauri::command]
pub async fn session_get(
    chat_module: State<'_, Arc<RwLock<ChatModule>>>,
    request: GetSessionRequest,
) -> AppResult<Session> {
    let module = chat_module.read().await;

    let session_id = SessionId::from(request.id);
    let query = GetSessionQuery::new(session_id);

    let response = module
        .get_session(query)
        .await
        .map_err(|e| AppError::Unknown(e.to_string()))?;

    let domain_session = response
        .session
        .ok_or_else(|| AppError::SessionNotFound(request.id.to_string()))?;

    Ok(Session {
        id: domain_session.id().into(),
        title: domain_session.title().to_string(),
        model_config: None,
        preset_id: domain_session.preset_id().map(|id| id.into()),
        created_at: domain_session.created_at(),
        updated_at: domain_session.updated_at(),
    })
}

/// 删除会话 - 使用 ChatModule
#[tauri::command]
pub async fn session_delete(
    chat_module: State<'_, Arc<RwLock<ChatModule>>>,
    request: DeleteSessionRequest,
) -> AppResult<()> {
    let module = chat_module.read().await;

    let session_id = SessionId::from(request.id);
    let command = DeleteSessionCommand::new(session_id);

    module
        .delete_session(command)
        .await
        .map_err(|e| AppError::Unknown(e.to_string()))?;

    Ok(())
}

/// 重命名会话 - 使用 UpdateSessionCommand
#[tauri::command]
pub async fn session_rename(
    chat_module: State<'_, Arc<RwLock<ChatModule>>>,
    request: RenameSessionRequest,
) -> AppResult<()> {
    let module = chat_module.read().await;
    let session_id = SessionId::from(request.id);

    let command = UpdateSessionCommand::new(session_id, Some(request.title), None);

    module
        .update_session(command)
        .await
        .map_err(|e| AppError::Unknown(e.to_string()))?;

    Ok(())
}
