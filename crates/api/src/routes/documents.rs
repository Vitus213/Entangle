use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use entangle_auth::PermissionService;
use entangle_core::{AppError, AppResult, DocumentPermissionService};
use entangle_db::{
    models::{
        AddCollaborator, AddCollaboratorByEmail, CreateDocument, DocumentListItem,
        DocumentResponse, UpdateDocument,
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

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    /// 搜索关键词（标题模糊匹配）
    pub q: Option<String>,
    /// 文件夹 ID 筛选
    pub folder_id: Option<Uuid>,
    /// 标签 ID 筛选
    pub tag_id: Option<Uuid>,
    /// 是否公开
    pub is_public: Option<bool>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

#[derive(Debug, Deserialize)]
pub struct DuplicateRequest {
    /// 新文档标题（可选，默认为 "原标题 (副本)"）
    pub title: Option<String>,
}

/// 创建文档
async fn create_document(
    State(pool): State<PgPool>,
    user: AuthUser,
    Json(doc_data): Json<CreateDocument>,
) -> AppResult<Json<DocumentResponse>> {
    // 检查用户是否有创建文档的权限
    if !DocumentPermissionService::has_document_permission(&pool, user.user_id, "create").await? {
        return Err(AppError::Forbidden("需要 document:create 权限".to_string()));
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
    // 如果是管理员，返回所有文档
    if PermissionService::is_admin(&pool, user.user_id).await? {
        let documents =
            DocumentRepository::list_all(&pool, pagination.limit, pagination.offset).await?;
        return Ok(Json(documents));
    }

    let documents = DocumentRepository::list_accessible(
        &pool,
        user.user_id,
        pagination.limit,
        pagination.offset,
    )
    .await?;

    Ok(Json(documents))
}

/// 列出所有文档（仅管理员）
async fn list_all_documents(
    State(pool): State<PgPool>,
    user: AuthUser,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<Json<Vec<DocumentListItem>>> {
    // 检查管理员权限
    if !PermissionService::is_admin(&pool, user.user_id).await? {
        return Err(AppError::Forbidden("需要管理员权限".to_string()));
    }

    let documents =
        DocumentRepository::list_all(&pool, pagination.limit, pagination.offset).await?;

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

/// 添加协作者（通过邮箱）
async fn add_collaborator(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(doc_id): Path<Uuid>,
    Json(collab_data): Json<AddCollaboratorByEmail>,
) -> AppResult<StatusCode> {
    // 检查管理权限
    if !DocumentPermissionService::can_manage(&pool, user.user_id, doc_id).await? {
        return Err(AppError::Forbidden("无权管理该文档协作者".to_string()));
    }

    DocumentRepository::add_collaborator_by_email(
        &pool,
        doc_id,
        &collab_data.email,
        collab_data.permission,
    )
    .await
    .map_err(|e| {
        if e.to_string().contains("no rows") {
            AppError::NotFound(format!("用户不存在: {}", collab_data.email))
        } else {
            AppError::Database(e)
        }
    })?;

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

/// 搜索文档
async fn search_documents(
    State(pool): State<PgPool>,
    user: AuthUser,
    Query(search): Query<SearchQuery>,
) -> AppResult<Json<Vec<DocumentListItem>>> {
    // 检查用户是否是管理员
    let is_admin = PermissionService::is_admin(&pool, user.user_id).await?;

    let documents = DocumentRepository::search(
        &pool,
        user.user_id,
        search.q.as_deref(),
        search.folder_id,
        search.tag_id,
        search.is_public,
        search.limit,
        search.offset,
        is_admin,  // 传递管理员状态
    )
    .await?;

    Ok(Json(documents))
}

/// 复制文档
async fn duplicate_document(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(doc_id): Path<Uuid>,
    Json(dup_data): Json<DuplicateRequest>,
) -> AppResult<Json<DocumentResponse>> {
    // 检查读取权限（可以复制有读取权限的文档）
    if !DocumentPermissionService::can_read(&pool, user.user_id, doc_id).await? {
        return Err(AppError::Forbidden("无权访问该文档".to_string()));
    }

    // 复制文档
    let document =
        DocumentRepository::duplicate(&pool, doc_id, user.user_id, dup_data.title).await?;

    // 获取完整文档信息
    let doc_response = DocumentRepository::get_detail(&pool, document.id)
        .await?
        .ok_or_else(|| AppError::Internal("复制文档后无法获取".to_string()))?;

    Ok(Json(doc_response))
}

/// 获取文档协作者列表
async fn list_collaborators(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(doc_id): Path<Uuid>,
) -> AppResult<Json<Vec<entangle_db::models::CollaboratorResponse>>> {
    // 检查读取权限
    if !DocumentPermissionService::can_read(&pool, user.user_id, doc_id).await? {
        return Err(AppError::Forbidden("无权访问该文档".to_string()));
    }

    let collaborators = DocumentRepository::list_collaborators_with_users(&pool, doc_id).await?;

    Ok(Json(collaborators))
}

/// 手动保存文档（触发 CRDT 状态持久化）
async fn save_document(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(doc_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    // 检查写入权限
    if !DocumentPermissionService::can_write(&pool, user.user_id, doc_id).await? {
        return Err(AppError::Forbidden("无权保存该文档".to_string()));
    }

    // 从 CRDT 状态获取最新内容
    // 注意：这里需要 WsHub 来获取当前的 CRDT 状态
    // 如果文档当前没有活动会话，则无需手动保存（自动保存会处理）

    Ok(StatusCode::OK)
}

/// 文档路由
pub fn document_routes() -> Router<PgPool> {
    Router::new()
        // 文档 CRUD
        .route("/documents", post(create_document))
        .route("/documents/search", get(search_documents)) // 必须在 :id 之前
        .route("/documents/my", get(list_my_documents))
        .route("/documents/all", get(list_all_documents)) // 管理员：查看所有文档
        .route("/documents/accessible", get(list_accessible_documents))
        .route("/documents/public", get(list_public_documents))
        .route("/documents/:id", get(get_document))
        .route("/documents/:id", put(update_document))
        .route("/documents/:id", delete(delete_document))
        // 文档操作
        .route("/documents/:id/save", post(save_document))
        .route("/documents/:id/duplicate", post(duplicate_document))
        // 协作者管理
        .route("/documents/:id/collaborators", get(list_collaborators))
        .route("/documents/:id/collaborators", post(add_collaborator))
        .route(
            "/documents/:id/collaborators/:collab_id",
            delete(remove_collaborator),
        )
}
