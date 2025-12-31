use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use entangle_core::{AppError, AppResult, DocumentPermissionService};
use entangle_db::{
    models::{
        AddCollaborator, CreateDocument, DocumentListItem, DocumentResponse, UpdateDocument,
    },
    DocumentRepository,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::middleware::AuthUser;

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

/// 创建文档
async fn create_document(
    State(pool): State<PgPool>,
    user: AuthUser,
    Json(doc_data): Json<CreateDocument>,
) -> AppResult<Json<DocumentResponse>> {
    // 检查用户是否有创建文档的权限
    if !DocumentPermissionService::has_document_permission(&pool, user.user_id, "create").await? {
        return Err(AppError::Forbidden(
            "需要 document:create 权限".to_string(),
        ));
    }

    // 创建文档
    let document = DocumentRepository::create(&pool, user.user_id, &doc_data).await?;

    // 获取完整文档信息
    let doc_response = DocumentRepository::get_detail(&pool, document.id)
        .await?
        .ok_or_else(|| AppError::Internal("创建文档后无法获取".to_string()))?;

    Ok(Json(doc_response))
}

/// 获取文档详情
async fn get_document(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(doc_id): Path<Uuid>,
) -> AppResult<Json<DocumentResponse>> {
    // 检查读取权限
    if !DocumentPermissionService::can_read(&pool, user.user_id, doc_id).await? {
        return Err(AppError::Forbidden("无权访问该文档".to_string()));
    }

    let document = DocumentRepository::get_detail(&pool, doc_id)
        .await?
        .ok_or_else(|| AppError::NotFound("文档不存在".to_string()))?;

    Ok(Json(document))
}

/// 更新文档
async fn update_document(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(doc_id): Path<Uuid>,
    Json(update_data): Json<UpdateDocument>,
) -> AppResult<Json<DocumentResponse>> {
    // 检查写入权限
    if !DocumentPermissionService::can_write(&pool, user.user_id, doc_id).await? {
        return Err(AppError::Forbidden("无权编辑该文档".to_string()));
    }

    // 更新文档
    DocumentRepository::update(&pool, doc_id, &update_data).await?;

    // 获取更新后的文档
    let document = DocumentRepository::get_detail(&pool, doc_id)
        .await?
        .ok_or_else(|| AppError::Internal("更新后无法获取文档".to_string()))?;

    Ok(Json(document))
}

/// 删除文档
async fn delete_document(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(doc_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    // 检查删除权限（仅所有者）
    if !DocumentPermissionService::can_delete(&pool, user.user_id, doc_id).await? {
        return Err(AppError::Forbidden("只有文档所有者可以删除".to_string()));
    }

    DocumentRepository::delete(&pool, doc_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// 列出我的文档
async fn list_my_documents(
    State(pool): State<PgPool>,
    user: AuthUser,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<Json<Vec<DocumentListItem>>> {
    let documents =
        DocumentRepository::list_by_owner(&pool, user.user_id, pagination.limit, pagination.offset)
            .await?;

    Ok(Json(documents))
}

/// 列出我可访问的文档（包含协作的）
async fn list_accessible_documents(
    State(pool): State<PgPool>,
    user: AuthUser,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<Json<Vec<DocumentListItem>>> {
    let documents =
        DocumentRepository::list_accessible(&pool, user.user_id, pagination.limit, pagination.offset)
            .await?;

    Ok(Json(documents))
}

/// 列出公开文档
async fn list_public_documents(
    State(pool): State<PgPool>,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<Json<Vec<DocumentListItem>>> {
    let documents =
        DocumentRepository::list_public(&pool, pagination.limit, pagination.offset).await?;

    Ok(Json(documents))
}

/// 添加协作者
async fn add_collaborator(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(doc_id): Path<Uuid>,
    Json(collab_data): Json<AddCollaborator>,
) -> AppResult<StatusCode> {
    // 检查管理权限
    if !DocumentPermissionService::can_manage(&pool, user.user_id, doc_id).await? {
        return Err(AppError::Forbidden("无权管理该文档协作者".to_string()));
    }

    DocumentRepository::add_collaborator(&pool, doc_id, &collab_data).await?;

    Ok(StatusCode::CREATED)
}

/// 移除协作者
async fn remove_collaborator(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path((doc_id, collab_id)): Path<(Uuid, Uuid)>,
) -> AppResult<StatusCode> {
    // 检查管理权限
    if !DocumentPermissionService::can_manage(&pool, user.user_id, doc_id).await? {
        return Err(AppError::Forbidden("无权管理该文档协作者".to_string()));
    }

    DocumentRepository::remove_collaborator(&pool, doc_id, collab_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// 文档路由
pub fn document_routes() -> Router<PgPool> {
    Router::new()
        // 文档 CRUD
        .route("/documents", post(create_document))
        .route("/documents/:id", get(get_document))
        .route("/documents/:id", put(update_document))
        .route("/documents/:id", delete(delete_document))
        // 文档列表
        .route("/documents/my", get(list_my_documents))
        .route("/documents/accessible", get(list_accessible_documents))
        .route("/documents/public", get(list_public_documents))
        // 协作者管理
        .route("/documents/:id/collaborators", post(add_collaborator))
        .route(
            "/documents/:id/collaborators/:collab_id",
            delete(remove_collaborator),
        )
}
