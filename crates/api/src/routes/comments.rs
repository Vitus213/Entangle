use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use entangle_core::{AppError, AppResult, DocumentPermissionService};
use entangle_db::{
    models::{
        CommentListItem, CommentResponse, CreateComment, UpdateComment,
        CreateNotification, NotificationType, ResourceType,
    },
    CommentRepository, DocumentRepository, NotificationRepository,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::middleware::AuthUser;

/// 创建评论
async fn create_comment(
    State(pool): State<PgPool>,
    user: AuthUser,
    Json(input): Json<CreateComment>,
) -> AppResult<Json<CommentResponse>> {
    // 检查文档访问权限
    if !DocumentPermissionService::can_read(&pool, user.user_id, input.doc_id).await? {
        return Err(AppError::Forbidden("无权访问该文档".to_string()));
    }

    // 如果是回复，检查父评论是否存在
    if let Some(parent_id) = input.parent_id {
        let parent = CommentRepository::find_by_id(&pool, parent_id).await?;
        if parent.is_none() {
            return Err(AppError::NotFound("父评论不存在".to_string()));
        }
    }

    // 创建评论
    let comment = CommentRepository::create(&pool, user.user_id, input.clone()).await?;

    // 发送通知给文档所有者（如果不是自己）
    let doc = DocumentRepository::find_by_id(&pool, input.doc_id).await?;
    if let Some(doc) = doc {
        if doc.owner_id != user.user_id {
            let _ = NotificationRepository::create(
                &pool,
                CreateNotification {
                    user_id: doc.owner_id,
                    notification_type: NotificationType::Comment,
                    title: "新评论".to_string(),
                    content: Some(format!("有人在您的文档 \"{}\" 中发表了评论", doc.title)),
                    resource_type: Some(ResourceType::Comment),
                    resource_id: Some(comment.id),
                    sender_id: Some(user.user_id),
                },
            )
            .await;
        }
    }

    // 获取完整的评论响应
    let response = CommentRepository::find_with_replies(&pool, comment.id).await?;
    match response {
        Some(r) => Ok(Json(r)),
        None => Err(AppError::Internal("创建评论后无法获取".to_string())),
    }
}

/// 获取文档的所有评论
async fn get_document_comments(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(doc_id): Path<Uuid>,
) -> AppResult<Json<Vec<CommentListItem>>> {
    // 检查文档访问权限
    if !DocumentPermissionService::can_read(&pool, user.user_id, doc_id).await? {
        return Err(AppError::Forbidden("无权访问该文档".to_string()));
    }

    let comments = CommentRepository::find_by_document(&pool, doc_id).await?;
    Ok(Json(comments))
}

/// 获取评论详情（包含回复）
async fn get_comment(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(comment_id): Path<Uuid>,
) -> AppResult<Json<CommentResponse>> {
    // 获取评论
    let comment = CommentRepository::find_by_id(&pool, comment_id).await?;
    let comment = comment.ok_or_else(|| AppError::NotFound("评论不存在".to_string()))?;

    // 检查文档访问权限
    if !DocumentPermissionService::can_read(&pool, user.user_id, comment.doc_id).await? {
        return Err(AppError::Forbidden("无权访问该评论".to_string()));
    }

    let response = CommentRepository::find_with_replies(&pool, comment_id).await?;
    match response {
        Some(r) => Ok(Json(r)),
        None => Err(AppError::NotFound("评论不存在".to_string())),
    }
}

/// 获取评论的回复
async fn get_comment_replies(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(comment_id): Path<Uuid>,
) -> AppResult<Json<Vec<CommentResponse>>> {
    // 获取评论
    let comment = CommentRepository::find_by_id(&pool, comment_id).await?;
    let comment = comment.ok_or_else(|| AppError::NotFound("评论不存在".to_string()))?;

    // 检查文档访问权限
    if !DocumentPermissionService::can_read(&pool, user.user_id, comment.doc_id).await? {
        return Err(AppError::Forbidden("无权访问该评论".to_string()));
    }

    let replies = CommentRepository::find_replies(&pool, comment_id).await?;
    Ok(Json(replies))
}

/// 更新评论
async fn update_comment(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(comment_id): Path<Uuid>,
    Json(input): Json<UpdateComment>,
) -> AppResult<Json<CommentResponse>> {
    // 检查是否是评论作者
    if !CommentRepository::is_author(&pool, comment_id, user.user_id).await? {
        return Err(AppError::Forbidden("只有评论作者可以编辑".to_string()));
    }

    // 更新评论
    CommentRepository::update(&pool, comment_id, input).await?;

    // 获取完整的评论响应
    let response = CommentRepository::find_with_replies(&pool, comment_id).await?;
    match response {
        Some(r) => Ok(Json(r)),
        None => Err(AppError::Internal("更新评论后无法获取".to_string())),
    }
}

/// 删除评论
async fn delete_comment(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(comment_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    // 获取评论
    let comment = CommentRepository::find_by_id(&pool, comment_id).await?;
    let comment = comment.ok_or_else(|| AppError::NotFound("评论不存在".to_string()))?;

    // 检查权限：评论作者或文档所有者可以删除
    let is_author = CommentRepository::is_author(&pool, comment_id, user.user_id).await?;
    let is_doc_owner = DocumentPermissionService::is_owner(&pool, user.user_id, comment.doc_id).await?;

    if !is_author && !is_doc_owner {
        return Err(AppError::Forbidden("无权删除该评论".to_string()));
    }

    CommentRepository::delete(&pool, comment_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 标记评论为已解决
async fn resolve_comment(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(comment_id): Path<Uuid>,
) -> AppResult<Json<CommentResponse>> {
    // 获取评论
    let comment = CommentRepository::find_by_id(&pool, comment_id).await?;
    let comment = comment.ok_or_else(|| AppError::NotFound("评论不存在".to_string()))?;

    // 检查权限：只有文档编辑者可以标记解决
    if !DocumentPermissionService::can_write(&pool, user.user_id, comment.doc_id).await? {
        return Err(AppError::Forbidden("无权操作该评论".to_string()));
    }

    CommentRepository::resolve(&pool, comment_id).await?;

    let response = CommentRepository::find_with_replies(&pool, comment_id).await?;
    match response {
        Some(r) => Ok(Json(r)),
        None => Err(AppError::Internal("操作后无法获取评论".to_string())),
    }
}

/// 取消评论的解决状态
async fn unresolve_comment(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(comment_id): Path<Uuid>,
) -> AppResult<Json<CommentResponse>> {
    // 获取评论
    let comment = CommentRepository::find_by_id(&pool, comment_id).await?;
    let comment = comment.ok_or_else(|| AppError::NotFound("评论不存在".to_string()))?;

    // 检查权限：只有文档编辑者可以操作
    if !DocumentPermissionService::can_write(&pool, user.user_id, comment.doc_id).await? {
        return Err(AppError::Forbidden("无权操作该评论".to_string()));
    }

    CommentRepository::unresolve(&pool, comment_id).await?;

    let response = CommentRepository::find_with_replies(&pool, comment_id).await?;
    match response {
        Some(r) => Ok(Json(r)),
        None => Err(AppError::Internal("操作后无法获取评论".to_string())),
    }
}

/// 评论路由
pub fn comment_routes() -> Router<PgPool> {
    Router::new()
        // 评论 CRUD
        .route("/comments", post(create_comment))
        .route("/comments/:id", get(get_comment))
        .route("/comments/:id", put(update_comment))
        .route("/comments/:id", delete(delete_comment))
        .route("/comments/:id/replies", get(get_comment_replies))
        .route("/comments/:id/resolve", put(resolve_comment))
        .route("/comments/:id/unresolve", put(unresolve_comment))
        // 文档评论
        .route("/documents/:id/comments", get(get_document_comments))
}
