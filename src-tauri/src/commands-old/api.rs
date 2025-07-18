use crate::entities::{Task, TaskFilter, TaskStats, TaskStatus};
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

/// Generic API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub metadata: Option<ResponseMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub request_id: String,
    pub execution_time_ms: u64,
    pub cache_hit: bool,
}

/// Generic list response
#[derive(Debug, Serialize, Deserialize)]
pub struct ListResponse<T> {
    pub items: Vec<T>,
    pub total: u32,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            metadata: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            metadata: None,
        }
    }
}

/// Helper to create service context from Tauri state
fn create_service_context(
    app_state: &crate::state::AppState,
    workspace_id: Option<Uuid>,
    user_id: Option<Uuid>,
) -> ServiceContext {
    let auth_context = user_id.map(|uid| crate::services::AuthContext {
        user_id: Some(uid),
        agent_id: None,
        participant_id: uid, // Simplified for demo
        workspace_id: workspace_id.unwrap_or_else(Uuid::new_v4),
        roles: vec!["user".to_string()],
        permissions: vec!["read".to_string(), "write".to_string()],
    });

    // TODO: Need to properly access the pool from DatabaseActor
    // For now, create a placeholder - this needs to be fixed in a proper implementation
    let pool = sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap();

    ServiceContext::build(
        pool,
        Arc::new(crate::services::traits::ActorSystem::new()),
        auth_context,
        workspace_id,
    )
}

// ===== TASK COMMANDS =====

#[tauri::command]
pub async fn create_task(
    input: CreateTaskInput,
    workspace_id: Option<String>,
    user_id: Option<String>,
    app_state: State<'_, crate::state::AppState>,
) -> Result<ApiResponse<Task>> {
    let workspace_uuid = workspace_id.and_then(|id| Uuid::parse_str(&id).ok());
    let user_uuid = user_id.and_then(|id| Uuid::parse_str(&id).ok());

    let ctx = create_service_context(&app_state, workspace_uuid, user_uuid);

    match app_state.services.task_service.create(&ctx, input).await {
        Ok(result) => Ok(ApiResponse::success(result.data)),
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

#[tauri::command]
pub async fn get_task(
    id: String,
    workspace_id: Option<String>,
    user_id: Option<String>,
    app_state: State<'_, crate::state::AppState>,
) -> Result<ApiResponse<Option<Task>>> {
    let task_id = Uuid::parse_str(&id)
        .map_err(|_| crate::error::AppError::ValidationError("Invalid task ID".to_string()))?;

    let workspace_uuid = workspace_id.and_then(|id| Uuid::parse_str(&id).ok());
    let user_uuid = user_id.and_then(|id| Uuid::parse_str(&id).ok());

    let ctx = create_service_context(&app_state, workspace_uuid, user_uuid);

    match app_state.services.task_service.get(&ctx, task_id).await {
        Ok(result) => Ok(ApiResponse::success(result.data)),
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

#[tauri::command]
pub async fn list_tasks(
    filter: TaskFilter,
    workspace_id: Option<String>,
    user_id: Option<String>,
    app_state: State<'_, crate::state::AppState>,
) -> Result<ApiResponse<ListResponse<Task>>> {
    let workspace_uuid = workspace_id.and_then(|id| Uuid::parse_str(&id).ok());
    let user_uuid = user_id.and_then(|id| Uuid::parse_str(&id).ok());

    let ctx = create_service_context(&app_state, workspace_uuid, user_uuid);

    match app_state.services.task_service.list(&ctx, filter).await {
        Ok(result) => {
            let total = result.data.len() as u32;
            let response = ListResponse {
                items: result.data,
                total,
                limit: None,
                offset: None,
            };
            Ok(ApiResponse::success(response))
        }
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

#[tauri::command]
pub async fn update_task(
    input: UpdateTaskInput,
    workspace_id: Option<String>,
    user_id: Option<String>,
    app_state: State<'_, crate::state::AppState>,
) -> Result<ApiResponse<Task>> {
    let workspace_uuid = workspace_id.and_then(|id| Uuid::parse_str(&id).ok());
    let user_uuid = user_id.and_then(|id| Uuid::parse_str(&id).ok());

    let ctx = create_service_context(&app_state, workspace_uuid, user_uuid);

    match app_state.services.task_service.update(&ctx, input).await {
        Ok(result) => Ok(ApiResponse::success(result.data)),
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

#[tauri::command]
pub async fn delete_task(
    id: String,
    workspace_id: Option<String>,
    user_id: Option<String>,
    app_state: State<'_, crate::state::AppState>,
) -> Result<ApiResponse<()>> {
    let task_id = Uuid::parse_str(&id)
        .map_err(|_| crate::error::AppError::ValidationError("Invalid task ID".to_string()))?;

    let workspace_uuid = workspace_id.and_then(|id| Uuid::parse_str(&id).ok());
    let user_uuid = user_id.and_then(|id| Uuid::parse_str(&id).ok());

    let ctx = create_service_context(&app_state, workspace_uuid, user_uuid);

    match app_state.services.task_service.delete(&ctx, task_id).await {
        Ok(_) => Ok(ApiResponse::success(())),
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

#[tauri::command]
pub async fn update_task_status(
    id: String,
    status: TaskStatus,
    workspace_id: Option<String>,
    user_id: Option<String>,
    app_state: State<'_, crate::state::AppState>,
) -> Result<ApiResponse<()>> {
    let task_id = Uuid::parse_str(&id)
        .map_err(|_| crate::error::AppError::ValidationError("Invalid task ID".to_string()))?;

    let workspace_uuid = workspace_id.and_then(|id| Uuid::parse_str(&id).ok());
    let user_uuid = user_id.and_then(|id| Uuid::parse_str(&id).ok());

    let ctx = create_service_context(&app_state, workspace_uuid, user_uuid);

    let input = crate::services::task::UpdateTaskInput {
        id: task_id,
        title: None,
        description: None,
        end_time: None,
        due_date: None,
        priority: None,
        urgency: None,
        importance: None,
        status: Some(status),
        primary_assignee_id: None,
        metadata: None,
    };

    match app_state.services.task_service.update(&ctx, input).await {
        Ok(_) => Ok(ApiResponse::success(())),
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

#[tauri::command]
pub async fn start_task(
    id: String,
    workspace_id: Option<String>,
    user_id: Option<String>,
    app_state: State<'_, crate::state::AppState>,
) -> Result<ApiResponse<()>> {
    let task_id = Uuid::parse_str(&id)
        .map_err(|_| crate::error::AppError::ValidationError("Invalid task ID".to_string()))?;

    let workspace_uuid = workspace_id.and_then(|id| Uuid::parse_str(&id).ok());
    let user_uuid = user_id.and_then(|id| Uuid::parse_str(&id).ok());

    let ctx = create_service_context(&app_state, workspace_uuid, user_uuid);

    match app_state
        .services
        .task_service
        .start_task(&ctx, task_id)
        .await
    {
        Ok(_) => Ok(ApiResponse::success(())),
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

#[tauri::command]
pub async fn complete_task(
    id: String,
    workspace_id: Option<String>,
    user_id: Option<String>,
    app_state: State<'_, crate::state::AppState>,
) -> Result<ApiResponse<()>> {
    let task_id = Uuid::parse_str(&id)
        .map_err(|_| crate::error::AppError::ValidationError("Invalid task ID".to_string()))?;

    let workspace_uuid = workspace_id.and_then(|id| Uuid::parse_str(&id).ok());
    let user_uuid = user_id.and_then(|id| Uuid::parse_str(&id).ok());

    let ctx = create_service_context(&app_state, workspace_uuid, user_uuid);

    match app_state
        .services
        .task_service
        .complete_task(&ctx, task_id)
        .await
    {
        Ok(_) => Ok(ApiResponse::success(())),
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

#[tauri::command]
pub async fn get_task_stats(
    workspace_id: Option<String>,
    user_id: Option<String>,
    app_state: State<'_, crate::state::AppState>,
) -> Result<ApiResponse<TaskStats>> {
    let workspace_uuid = workspace_id.and_then(|id| Uuid::parse_str(&id).ok());
    let user_uuid = user_id.and_then(|id| Uuid::parse_str(&id).ok());

    let ctx = create_service_context(&app_state, workspace_uuid, user_uuid);

    match app_state
        .services
        .task_service
        .get_stats(&ctx, workspace_uuid)
        .await
    {
        Ok(result) => Ok(ApiResponse::success(result.data)),
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

#[tauri::command]
pub async fn get_overdue_tasks(
    workspace_id: Option<String>,
    user_id: Option<String>,
    app_state: State<'_, crate::state::AppState>,
) -> Result<ApiResponse<Vec<Task>>> {
    let workspace_uuid = workspace_id.and_then(|id| Uuid::parse_str(&id).ok());
    let user_uuid = user_id.and_then(|id| Uuid::parse_str(&id).ok());

    let ctx = create_service_context(&app_state, workspace_uuid, user_uuid);

    match app_state
        .services
        .task_service
        .get_overdue_tasks(&ctx)
        .await
    {
        Ok(result) => Ok(ApiResponse::success(result.data)),
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

#[tauri::command]
pub async fn get_high_priority_tasks(
    workspace_id: Option<String>,
    user_id: Option<String>,
    app_state: State<'_, crate::state::AppState>,
) -> Result<ApiResponse<Vec<Task>>> {
    let workspace_uuid = workspace_id.and_then(|id| Uuid::parse_str(&id).ok());
    let user_uuid = user_id.and_then(|id| Uuid::parse_str(&id).ok());

    let ctx = create_service_context(&app_state, workspace_uuid, user_uuid);

    match app_state
        .services
        .task_service
        .get_high_priority_tasks(&ctx)
        .await
    {
        Ok(result) => Ok(ApiResponse::success(result.data)),
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

// ===== UTILITY COMMANDS =====

#[tauri::command]
pub async fn health_check() -> Result<ApiResponse<String>> {
    Ok(ApiResponse::success("Service is healthy".to_string()))
}

#[tauri::command]
pub async fn get_version() -> Result<ApiResponse<String>> {
    Ok(ApiResponse::success(env!("CARGO_PKG_VERSION").to_string()))
}
