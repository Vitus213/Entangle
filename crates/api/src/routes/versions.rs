use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use entangle_core::{AppError, AppResult, DocumentPermissionService};
use entangle_db::{
    models::{
        CreateVersion, DocumentVersion, VersionDiff, VersionListItem, VersionResponse,
        DiffLine, DiffOperation,
    },
    DocumentRepository, VersionRepository,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::middleware::AuthUser;

/// 创建版本快照
async fn create_version(
    State(pool): State<PgPool>,
    user: AuthUser,
    Json(input): Json<CreateVersion>,
) -> AppResult<Json<VersionResponse>> {
    // 检查文档写权限
    if !DocumentPermissionService::can_write(&pool, user.user_id, input.doc_id).await? {
        return Err(AppError::Forbidden("无权为该文档创建版本".to_string()));
    }

    // 获取当前文档
    let doc = DocumentRepository::find_by_id(&pool, input.doc_id).await?;
    let doc = doc.ok_or_else(|| AppError::NotFound("文档不存在".to_string()))?;

    // 创建版本
    let version = VersionRepository::create(
        &pool,
        input.doc_id,
        doc.title,
        doc.content,
        doc.crdt_state,
        user.user_id,
        input.description,
    )
    .await?;

    // 获取完整响应
    let response = VersionRepository::find_with_creator(&pool, version.id).await?;
    match response {
        Some(r) => Ok(Json(r)),
        None => Err(AppError::Internal("创建版本后无法获取".to_string())),
    }
}

/// 获取文档的所有版本
async fn get_document_versions(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(doc_id): Path<Uuid>,
) -> AppResult<Json<Vec<VersionListItem>>> {
    // 检查文档读权限
    if !DocumentPermissionService::can_read(&pool, user.user_id, doc_id).await? {
        return Err(AppError::Forbidden("无权访问该文档".to_string()));
    }

    let versions = VersionRepository::find_by_document(&pool, doc_id).await?;
    Ok(Json(versions))
}

/// 获取版本详情
async fn get_version(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(version_id): Path<Uuid>,
) -> AppResult<Json<VersionResponse>> {
    // 获取版本
    let version: Option<DocumentVersion> = VersionRepository::find_by_id(&pool, version_id).await?;
    let version = version.ok_or_else(|| AppError::NotFound("版本不存在".to_string()))?;

    // 检查文档读权限
    if !DocumentPermissionService::can_read(&pool, user.user_id, version.doc_id).await? {
        return Err(AppError::Forbidden("无权访问该版本".to_string()));
    }

    let response = VersionRepository::find_with_creator(&pool, version_id).await?;
    match response {
        Some(r) => Ok(Json(r)),
        None => Err(AppError::NotFound("版本不存在".to_string())),
    }
}

/// 对比两个版本
async fn compare_versions(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path((version_a_id, version_b_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<VersionDiff>> {
    // 获取两个版本
    let version_a: Option<VersionResponse> = VersionRepository::find_with_creator(&pool, version_a_id).await?;
    let version_a = version_a.ok_or_else(|| AppError::NotFound("版本 A 不存在".to_string()))?;

    let version_b: Option<VersionResponse> = VersionRepository::find_with_creator(&pool, version_b_id).await?;
    let version_b = version_b.ok_or_else(|| AppError::NotFound("版本 B 不存在".to_string()))?;

    // 检查两个版本是否属于同一文档
    if version_a.doc_id != version_b.doc_id {
        return Err(AppError::BadRequest("两个版本必须属于同一文档".to_string()));
    }

    // 检查文档读权限
    if !DocumentPermissionService::can_read(&pool, user.user_id, version_a.doc_id).await? {
        return Err(AppError::Forbidden("无权访问该文档".to_string()));
    }

    // 计算差异
    let content_diff = compute_diff(&version_a.content, &version_b.content);

    Ok(Json(VersionDiff {
        version_a: VersionListItem {
            id: version_a.id,
            doc_id: version_a.doc_id,
            version_number: version_a.version_number,
            title: version_a.title.clone(),
            creator: version_a.creator.clone(),
            description: version_a.description.clone(),
            created_at: version_a.created_at,
        },
        version_b: VersionListItem {
            id: version_b.id,
            doc_id: version_b.doc_id,
            version_number: version_b.version_number,
            title: version_b.title.clone(),
            creator: version_b.creator.clone(),
            description: version_b.description.clone(),
            created_at: version_b.created_at,
        },
        title_changed: version_a.title != version_b.title,
        content_diff,
    }))
}

/// 回滚到指定版本
async fn rollback_to_version(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(version_id): Path<Uuid>,
) -> AppResult<Json<VersionResponse>> {
    // 获取版本
    let version: Option<DocumentVersion> = VersionRepository::find_by_id(&pool, version_id).await?;
    let version = version.ok_or_else(|| AppError::NotFound("版本不存在".to_string()))?;

    // 检查文档写权限
    if !DocumentPermissionService::can_write(&pool, user.user_id, version.doc_id).await? {
        return Err(AppError::Forbidden("无权修改该文档".to_string()));
    }

    // 先为当前版本创建一个快照（作为回滚前的备份）
    let current_doc = DocumentRepository::find_by_id(&pool, version.doc_id).await?;
    let current_doc = current_doc.ok_or_else(|| AppError::NotFound("文档不存在".to_string()))?;

    let _ = VersionRepository::create(
        &pool,
        version.doc_id,
        current_doc.title,
        current_doc.content,
        current_doc.crdt_state,
        user.user_id,
        Some(format!("回滚前自动备份 (回滚到版本 {})", version.version_number)),
    )
    .await?;

    // 更新文档为指定版本的内容
    DocumentRepository::update_content(
        &pool,
        version.doc_id,
        version.title.clone(),
        version.content.clone(),
        version.crdt_state.clone(),
    )
    .await?;

    // 创建新版本记录回滚操作
    let new_version = VersionRepository::create(
        &pool,
        version.doc_id,
        version.title,
        version.content,
        version.crdt_state,
        user.user_id,
        Some(format!("从版本 {} 回滚", version.version_number)),
    )
    .await?;

    // 获取完整响应
    let response = VersionRepository::find_with_creator(&pool, new_version.id).await?;
    match response {
        Some(r) => Ok(Json(r)),
        None => Err(AppError::Internal("回滚后无法获取版本".to_string())),
    }
}

/// 删除版本
async fn delete_version(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(version_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    // 获取版本
    let version: Option<DocumentVersion> = VersionRepository::find_by_id(&pool, version_id).await?;
    let version = version.ok_or_else(|| AppError::NotFound("版本不存在".to_string()))?;

    // 检查是否是文档所有者
    if !DocumentPermissionService::is_owner(&pool, user.user_id, version.doc_id).await? {
        return Err(AppError::Forbidden("只有文档所有者可以删除版本".to_string()));
    }

    VersionRepository::delete(&pool, version_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 计算两个文本的差异
fn compute_diff(old_text: &str, new_text: &str) -> Vec<DiffLine> {
    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();

    let mut diff_lines = Vec::new();
    let mut old_idx = 0;
    let mut new_idx = 0;
    let mut line_number = 1;

    // 简单的逐行对比算法
    while old_idx < old_lines.len() || new_idx < new_lines.len() {
        match (old_lines.get(old_idx), new_lines.get(new_idx)) {
            (Some(old), Some(new)) if old == new => {
                // 行相同
                diff_lines.push(DiffLine {
                    line_number,
                    operation: DiffOperation::Equal,
                    content: old.to_string(),
                });
                old_idx += 1;
                new_idx += 1;
            }
            (Some(old), Some(new)) => {
                // 行不同，标记为删除旧行、添加新行
                diff_lines.push(DiffLine {
                    line_number,
                    operation: DiffOperation::Remove,
                    content: old.to_string(),
                });
                diff_lines.push(DiffLine {
                    line_number,
                    operation: DiffOperation::Add,
                    content: new.to_string(),
                });
                old_idx += 1;
                new_idx += 1;
            }
            (Some(old), None) => {
                // 旧文本有多余行
                diff_lines.push(DiffLine {
                    line_number,
                    operation: DiffOperation::Remove,
                    content: old.to_string(),
                });
                old_idx += 1;
            }
            (None, Some(new)) => {
                // 新文本有多余行
                diff_lines.push(DiffLine {
                    line_number,
                    operation: DiffOperation::Add,
                    content: new.to_string(),
                });
                new_idx += 1;
            }
            (None, None) => break,
        }
        line_number += 1;
    }

    diff_lines
}

/// 版本控制路由
pub fn version_routes() -> Router<PgPool> {
    Router::new()
        // 版本 CRUD
        .route("/versions", post(create_version))
        .route("/versions/:id", get(get_version))
        .route("/versions/:id", delete(delete_version))
        .route("/versions/:id/rollback", post(rollback_to_version))
        // 版本对比
        .route("/versions/:a/compare/:b", get(compare_versions))
        // 文档版本列表
        .route("/documents/:id/versions", get(get_document_versions))
}
