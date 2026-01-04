use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use entangle_core::{AppError, AppResult, DocumentPermissionService};
use entangle_db::{
    models::{
        CreateNotification, CreateTask, NotificationType, ResourceType,
        TaskListItem, TaskResponse, UpdateTask, UpdateTaskStatus,
    },
    NotificationRepository, TaskRepository, UserRepository,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::middleware::AuthUser;

#[derive(Debug, Deserialize)]
struct PaginationQuery {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_limit() -> i64 {
    20
}

#[derive(Debug, Deserialize)]
struct TaskFilterQuery {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
    /// 过滤类型: "all", "created", "assigned"
    #[serde(default = "default_filter")]
    filter: String,
}

fn default_filter() -> String {
    "all".to_string()
}

/// 创建任务
async fn create_task(
    State(pool): State<PgPool>,
    user: AuthUser,
    Json(input): Json<CreateTask>,
) -> AppResult<Json<TaskResponse>> {
    // 如果关联文档，检查权限
    if let Some(doc_id) = input.doc_id {
        if !DocumentPermissionService::can_read(&pool, user.user_id, doc_id).await? {
            return Err(AppError::Forbidden("无权访问该文档".to_string()));
        }
    }

    // 如果指定了被分配者，验证用户存在
    if let Some(assignee_id) = input.assignee_id {
        let assignee = UserRepository::find_by_id(&pool, assignee_id).await?;
        if assignee.is_none() {
            return Err(AppError::NotFound("被分配用户不存在".to_string()));
        }
    }

    // 创建任务
    let task = TaskRepository::create(&pool, user.user_id, input.clone()).await?;

    // 发送通知给被分配者（如果不是自己）
    if let Some(assignee_id) = input.assignee_id {
        if assignee_id != user.user_id {
            let _ = NotificationRepository::create(
                &pool,
                CreateNotification {
                    user_id: assignee_id,
                    notification_type: NotificationType::Task,
                    title: "新任务分配".to_string(),
                    content: Some(format!("您被分配了一个新任务: {}", input.title)),
                    resource_type: Some(ResourceType::Task),
                    resource_id: Some(task.id),
                    sender_id: Some(user.user_id),
                },
            )
            .await;
        }
    }

    // 获取完整的任务响应
    let response = TaskRepository::find_with_details(&pool, task.id).await?;
    match response {
        Some(r) => Ok(Json(r)),
        None => Err(AppError::Internal("创建任务后无法获取".to_string())),
    }
}

/// 获取任务列表
async fn list_tasks(
    State(pool): State<PgPool>,
    user: AuthUser,
    Query(query): Query<TaskFilterQuery>,
) -> AppResult<Json<Vec<TaskListItem>>> {
    let tasks = match query.filter.as_str() {
        "created" => TaskRepository::find_created_by(&pool, user.user_id, query.limit, query.offset).await?,
        "assigned" => TaskRepository::find_assigned_to(&pool, user.user_id, query.limit, query.offset).await?,
        _ => TaskRepository::find_for_user(&pool, user.user_id, query.limit, query.offset).await?,
    };
    Ok(Json(tasks))
}

/// 获取任务详情
async fn get_task(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(task_id): Path<Uuid>,
) -> AppResult<Json<TaskResponse>> {
    // 检查权限
    if !TaskRepository::can_view(&pool, task_id, user.user_id).await? {
        return Err(AppError::Forbidden("无权查看该任务".to_string()));
    }

    let response = TaskRepository::find_with_details(&pool, task_id).await?;
    match response {
        Some(r) => Ok(Json(r)),
        None => Err(AppError::NotFound("任务不存在".to_string())),
    }
}

/// 更新任务
async fn update_task(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(task_id): Path<Uuid>,
    Json(input): Json<UpdateTask>,
) -> AppResult<Json<TaskResponse>> {
    // 只有创建者可以更新任务
    if !TaskRepository::is_creator(&pool, task_id, user.user_id).await? {
        return Err(AppError::Forbidden("只有任务创建者可以编辑".to_string()));
    }

    // 如果更新被分配者，验证用户存在
    if let Some(assignee_id) = input.assignee_id {
        let assignee = UserRepository::find_by_id(&pool, assignee_id).await?;
        if assignee.is_none() {
            return Err(AppError::NotFound("被分配用户不存在".to_string()));
        }
    }

    TaskRepository::update(&pool, task_id, input).await?;

    let response = TaskRepository::find_with_details(&pool, task_id).await?;
    match response {
        Some(r) => Ok(Json(r)),
        None => Err(AppError::Internal("更新任务后无法获取".to_string())),
    }
}

/// 更新任务状态
async fn update_task_status(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(task_id): Path<Uuid>,
    Json(input): Json<UpdateTaskStatus>,
) -> AppResult<Json<TaskResponse>> {
    // 创建者和被分配者都可以更新状态
    if !TaskRepository::can_view(&pool, task_id, user.user_id).await? {
        return Err(AppError::Forbidden("无权更新该任务状态".to_string()));
    }

    let old_task = TaskRepository::find_by_id(&pool, task_id).await?;
    let old_task = old_task.ok_or_else(|| AppError::NotFound("任务不存在".to_string()))?;

    TaskRepository::update_status(&pool, task_id, input.clone()).await?;

    // 如果状态变为已完成，通知创建者
    if input.status.to_string() == "completed" && old_task.created_by != user.user_id {
        let _ = NotificationRepository::create(
            &pool,
            CreateNotification {
                user_id: old_task.created_by,
                notification_type: NotificationType::Task,
                title: "任务已完成".to_string(),
                content: Some(format!("任务 \"{}\" 已被标记为完成", old_task.title)),
                resource_type: Some(ResourceType::Task),
                resource_id: Some(task_id),
                sender_id: Some(user.user_id),
            },
        )
        .await;
    }

    let response = TaskRepository::find_with_details(&pool, task_id).await?;
    match response {
        Some(r) => Ok(Json(r)),
        None => Err(AppError::Internal("更新任务后无法获取".to_string())),
    }
}

/// 分配任务
#[derive(Debug, Deserialize)]
struct AssignTask {
    assignee_id: Option<Uuid>,
}

async fn assign_task(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(task_id): Path<Uuid>,
    Json(input): Json<AssignTask>,
) -> AppResult<Json<TaskResponse>> {
    // 只有创建者可以分配任务
    if !TaskRepository::is_creator(&pool, task_id, user.user_id).await? {
        return Err(AppError::Forbidden("只有任务创建者可以分配".to_string()));
    }

    let task = TaskRepository::find_by_id(&pool, task_id).await?;
    let task = task.ok_or_else(|| AppError::NotFound("任务不存在".to_string()))?;

    // 验证被分配者存在
    if let Some(assignee_id) = input.assignee_id {
        let assignee = UserRepository::find_by_id(&pool, assignee_id).await?;
        if assignee.is_none() {
            return Err(AppError::NotFound("被分配用户不存在".to_string()));
        }

        // 发送通知给新的被分配者
        if assignee_id != user.user_id {
            let _ = NotificationRepository::create(
                &pool,
                CreateNotification {
                    user_id: assignee_id,
                    notification_type: NotificationType::Task,
                    title: "任务分配".to_string(),
                    content: Some(format!("您被分配了任务: {}", task.title)),
                    resource_type: Some(ResourceType::Task),
                    resource_id: Some(task_id),
                    sender_id: Some(user.user_id),
                },
            )
            .await;
        }
    }

    TaskRepository::assign(&pool, task_id, input.assignee_id).await?;

    let response = TaskRepository::find_with_details(&pool, task_id).await?;
    match response {
        Some(r) => Ok(Json(r)),
        None => Err(AppError::Internal("分配任务后无法获取".to_string())),
    }
}

/// 删除任务
async fn delete_task(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(task_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    // 只有创建者可以删除任务
    if !TaskRepository::is_creator(&pool, task_id, user.user_id).await? {
        return Err(AppError::Forbidden("只有任务创建者可以删除".to_string()));
    }

    TaskRepository::delete(&pool, task_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 获取文档相关的任务
async fn get_document_tasks(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(doc_id): Path<Uuid>,
    Query(query): Query<PaginationQuery>,
) -> AppResult<Json<Vec<TaskListItem>>> {
    // 检查文档权限
    if !DocumentPermissionService::can_read(&pool, user.user_id, doc_id).await? {
        return Err(AppError::Forbidden("无权访问该文档".to_string()));
    }

    let tasks = TaskRepository::find_by_document(&pool, doc_id, query.limit, query.offset).await?;
    Ok(Json(tasks))
}

/// 任务路由
pub fn task_routes() -> Router<PgPool> {
    Router::new()
        // 任务 CRUD
        .route("/tasks", post(create_task))
        .route("/tasks", get(list_tasks))
        .route("/tasks/:id", get(get_task))
        .route("/tasks/:id", put(update_task))
        .route("/tasks/:id", delete(delete_task))
        .route("/tasks/:id/status", put(update_task_status))
        .route("/tasks/:id/assign", put(assign_task))
        // 文档任务
        .route("/documents/:id/tasks", get(get_document_tasks))
}
