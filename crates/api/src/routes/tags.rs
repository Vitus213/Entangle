use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use entangle_core::{AppError, AppResult, DocumentPermissionService};
use entangle_db::{
    models::{AddTagToDocument, CreateTag, SetDocumentTags, TagSummary, TagWithCount, UpdateTag},
    TagRepository,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::middleware::AuthUser;

/// 创建标签
async fn create_tag(
    State(pool): State<PgPool>,
    user: AuthUser,
    Json(tag_data): Json<CreateTag>,
) -> AppResult<Json<TagWithCount>> {
    // 创建标签
    let tag = TagRepository::create(&pool, &tag_data, user.user_id).await?;

    // 返回带计数的标签（新创建的标签计数为 0）
    let tag_with_count = TagWithCount {
        tag,
        document_count: 0,
    };

    Ok(Json(tag_with_count))
}

/// 获取我的所有标签
async fn list_my_tags(
    State(pool): State<PgPool>,
    user: AuthUser,
) -> AppResult<Json<Vec<TagWithCount>>> {
    let tags = TagRepository::list_by_owner(&pool, user.user_id).await?;

    Ok(Json(tags))
}

/// 更新标签
async fn update_tag(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(tag_id): Path<Uuid>,
    Json(update_data): Json<UpdateTag>,
) -> AppResult<Json<TagWithCount>> {
    // 检查所有权
    if !TagRepository::is_owner(&pool, tag_id, user.user_id).await? {
        return Err(AppError::Forbidden("无权修改该标签".to_string()));
    }

    // 更新标签
    let _tag = TagRepository::update(&pool, tag_id, &update_data).await?;

    // 获取文档计数
    let tags = TagRepository::list_by_owner(&pool, user.user_id).await?;
    let tag_with_count = tags
        .into_iter()
        .find(|t| t.tag.id == tag_id)
        .ok_or_else(|| AppError::Internal("更新后无法获取标签".to_string()))?;

    Ok(Json(tag_with_count))
}

/// 删除标签
async fn delete_tag(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(tag_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    // 检查所有权
    if !TagRepository::is_owner(&pool, tag_id, user.user_id).await? {
        return Err(AppError::Forbidden("只有标签所有者可以删除".to_string()));
    }

    TagRepository::delete(&pool, tag_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// 为文档添加标签
async fn add_tag_to_document(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(doc_id): Path<Uuid>,
    Json(data): Json<AddTagToDocument>,
) -> AppResult<StatusCode> {
    // 检查标签所有权
    if !TagRepository::is_owner(&pool, data.tag_id, user.user_id).await? {
        return Err(AppError::Forbidden("无权使用该标签".to_string()));
    }

    // 检查文档写权限
    if !DocumentPermissionService::can_write(&pool, user.user_id, doc_id).await? {
        return Err(AppError::Forbidden("无权编辑该文档".to_string()));
    }

    TagRepository::add_to_document(&pool, doc_id, data.tag_id).await?;

    Ok(StatusCode::CREATED)
}

/// 从文档移除标签
async fn remove_tag_from_document(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path((doc_id, tag_id)): Path<(Uuid, Uuid)>,
) -> AppResult<StatusCode> {
    // 检查文档写权限
    if !DocumentPermissionService::can_write(&pool, user.user_id, doc_id).await? {
        return Err(AppError::Forbidden("无权编辑该文档".to_string()));
    }

    TagRepository::remove_from_document(&pool, doc_id, tag_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// 获取文档的所有标签
async fn get_document_tags(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(doc_id): Path<Uuid>,
) -> AppResult<Json<Vec<TagSummary>>> {
    // 检查文档读权限
    if !DocumentPermissionService::can_read(&pool, user.user_id, doc_id).await? {
        return Err(AppError::Forbidden("无权访问该文档".to_string()));
    }

    let tags = TagRepository::get_document_tags(&pool, doc_id).await?;

    Ok(Json(tags))
}

/// 批量设置文档标签
async fn set_document_tags(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(doc_id): Path<Uuid>,
    Json(data): Json<SetDocumentTags>,
) -> AppResult<Json<Vec<TagSummary>>> {
    // 检查文档写权限
    if !DocumentPermissionService::can_write(&pool, user.user_id, doc_id).await? {
        return Err(AppError::Forbidden("无权编辑该文档".to_string()));
    }

    // 检查所有标签的所有权
    for tag_id in &data.tag_ids {
        if !TagRepository::is_owner(&pool, *tag_id, user.user_id).await? {
            return Err(AppError::Forbidden(format!("无权使用标签 {}", tag_id)));
        }
    }

    // 批量设置标签
    TagRepository::set_document_tags(&pool, doc_id, &data.tag_ids).await?;

    // 返回更新后的标签列表
    let tags = TagRepository::get_document_tags(&pool, doc_id).await?;

    Ok(Json(tags))
}

#[derive(Debug, Deserialize)]
struct TagFilterQuery {
    tag_ids: String,  // 逗号分隔的 UUID 列表
    #[serde(default = "default_match_mode")]
    match_mode: String,  // "all" 或 "any"
}

fn default_match_mode() -> String {
    "any".to_string()
}

/// 按标签筛选文档
async fn get_documents_by_tags(
    State(pool): State<PgPool>,
    user: AuthUser,
    Query(query): Query<TagFilterQuery>,
) -> AppResult<Json<Vec<entangle_db::models::DocumentListItem>>> {
    // 解析标签 ID 列表
    let tag_ids: Result<Vec<Uuid>, _> = query
        .tag_ids
        .split(',')
        .map(|s| s.trim().parse::<Uuid>())
        .collect();

    let tag_ids = tag_ids.map_err(|_| AppError::BadRequest("无效的标签 ID".to_string()))?;

    if tag_ids.is_empty() {
        return Ok(Json(vec![]));
    }

    // 检查所有标签的所有权
    for tag_id in &tag_ids {
        if !TagRepository::is_owner(&pool, *tag_id, user.user_id).await? {
            return Err(AppError::Forbidden(format!("无权使用标签 {}", tag_id)));
        }
    }

    // 确定匹配模式
    let match_all = query.match_mode == "all";

    // 获取文档
    let documents = TagRepository::get_documents_by_tags(&pool, user.user_id, &tag_ids, match_all).await?;

    Ok(Json(documents))
}

/// 标签路由
pub fn tag_routes() -> Router<PgPool> {
    Router::new()
        // 标签 CRUD
        .route("/tags", post(create_tag))
        .route("/tags", get(list_my_tags))
        .route("/tags/:id", put(update_tag))
        .route("/tags/:id", delete(delete_tag))
        // 文档标签管理
        .route("/documents/:id/tags", post(add_tag_to_document))
        .route("/documents/:id/tags", get(get_document_tags))
        .route("/documents/:id/tags", put(set_document_tags))
        .route(
            "/documents/:id/tags/:tag_id",
            delete(remove_tag_from_document),
        )
        // 按标签筛选
        .route("/documents/by-tags", get(get_documents_by_tags))
}
