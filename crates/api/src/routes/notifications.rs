use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, put},
    Json, Router,
};
use entangle_core::{AppResult};
use entangle_db::{
    models::{NotificationResponse, UnreadCountResponse},
    NotificationRepository,
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

/// 获取我的通知列表
async fn list_notifications(
    State(pool): State<PgPool>,
    user: AuthUser,
    Query(query): Query<PaginationQuery>,
) -> AppResult<Json<Vec<NotificationResponse>>> {
    let notifications =
        NotificationRepository::find_by_user(&pool, user.user_id, query.limit, query.offset)
            .await?;
    Ok(Json(notifications))
}

/// 获取未读通知数量
async fn get_unread_count(
    State(pool): State<PgPool>,
    user: AuthUser,
) -> AppResult<Json<UnreadCountResponse>> {
    let count = NotificationRepository::count_unread(&pool, user.user_id).await?;
    Ok(Json(UnreadCountResponse { count }))
}

/// 标记单个通知为已读
async fn mark_read(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(notification_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    NotificationRepository::mark_read(&pool, notification_id, user.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 标记所有通知为已读
async fn mark_all_read(
    State(pool): State<PgPool>,
    user: AuthUser,
) -> AppResult<Json<serde_json::Value>> {
    let count = NotificationRepository::mark_all_read(&pool, user.user_id).await?;
    Ok(Json(serde_json::json!({
        "marked_count": count
    })))
}

/// 删除单个通知
async fn delete_notification(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(notification_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    NotificationRepository::delete(&pool, notification_id, user.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 删除所有已读通知
async fn delete_read_notifications(
    State(pool): State<PgPool>,
    user: AuthUser,
) -> AppResult<Json<serde_json::Value>> {
    let count = NotificationRepository::delete_read(&pool, user.user_id).await?;
    Ok(Json(serde_json::json!({
        "deleted_count": count
    })))
}

/// 通知路由
pub fn notification_routes() -> Router<PgPool> {
    Router::new()
        .route("/notifications", get(list_notifications))
        .route("/notifications/unread-count", get(get_unread_count))
        .route("/notifications/read-all", put(mark_all_read))
        .route("/notifications/delete-read", delete(delete_read_notifications))
        .route("/notifications/:id/read", put(mark_read))
        .route("/notifications/:id", delete(delete_notification))
}
