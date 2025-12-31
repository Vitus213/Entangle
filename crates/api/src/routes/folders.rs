use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use entangle_core::{AppError, AppResult};
use entangle_db::{
    models::{
        CreateFolder, FolderContents, FolderResponse, FolderTree, MoveDocument, UpdateFolder,
    },
    FolderRepository,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::middleware::AuthUser;

/// 创建文件夹
async fn create_folder(
    State(pool): State<PgPool>,
    user: AuthUser,
    Json(folder_data): Json<CreateFolder>,
) -> AppResult<Json<FolderResponse>> {
    // 任何登录用户都可以创建文件夹
    let folder = FolderRepository::create(&pool, &folder_data, user.user_id).await?;

    // 获取完整文件夹信息
    let folder_response = FolderRepository::get_detail(&pool, folder.id)
        .await?
        .ok_or_else(|| AppError::Internal("创建文件夹后无法获取".to_string()))?;

    Ok(Json(folder_response))
}

/// 获取文件夹详情
async fn get_folder(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(folder_id): Path<Uuid>,
) -> AppResult<Json<FolderResponse>> {
    // 检查所有权
    if !FolderRepository::is_owner(&pool, folder_id, user.user_id).await? {
        return Err(AppError::Forbidden("无权访问该文件夹".to_string()));
    }

    let folder = FolderRepository::get_detail(&pool, folder_id)
        .await?
        .ok_or_else(|| AppError::NotFound("文件夹不存在".to_string()))?;

    Ok(Json(folder))
}

/// 更新文件夹
async fn update_folder(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(folder_id): Path<Uuid>,
    Json(update_data): Json<UpdateFolder>,
) -> AppResult<Json<FolderResponse>> {
    // 检查所有权
    if !FolderRepository::is_owner(&pool, folder_id, user.user_id).await? {
        return Err(AppError::Forbidden("无权修改该文件夹".to_string()));
    }

    // 更新文件夹
    FolderRepository::update(&pool, folder_id, &update_data).await?;

    // 获取更新后的文件夹
    let folder = FolderRepository::get_detail(&pool, folder_id)
        .await?
        .ok_or_else(|| AppError::Internal("更新后无法获取文件夹".to_string()))?;

    Ok(Json(folder))
}

/// 删除文件夹
async fn delete_folder(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(folder_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    // 检查所有权
    if !FolderRepository::is_owner(&pool, folder_id, user.user_id).await? {
        return Err(AppError::Forbidden("只有文件夹所有者可以删除".to_string()));
    }

    FolderRepository::delete(&pool, folder_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// 获取文件夹树
async fn get_folder_tree(
    State(pool): State<PgPool>,
    user: AuthUser,
) -> AppResult<Json<Vec<FolderTree>>> {
    let tree = FolderRepository::get_tree(&pool, user.user_id).await?;

    Ok(Json(tree))
}

/// 获取文件夹内容
async fn get_folder_contents(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(folder_id): Path<Uuid>,
) -> AppResult<Json<FolderContents>> {
    // 检查所有权
    if !FolderRepository::is_owner(&pool, folder_id, user.user_id).await? {
        return Err(AppError::Forbidden("无权访问该文件夹".to_string()));
    }

    let contents = FolderRepository::get_contents(&pool, folder_id).await?;

    Ok(Json(contents))
}

/// 移动文档到文件夹
async fn move_document(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(doc_id): Path<Uuid>,
    Json(move_data): Json<MoveDocument>,
) -> AppResult<StatusCode> {
    // 检查文档所有权
    let doc = entangle_db::DocumentRepository::find_by_id(&pool, doc_id)
        .await?
        .ok_or_else(|| AppError::NotFound("文档不存在".to_string()))?;

    if doc.owner_id != user.user_id {
        return Err(AppError::Forbidden("只有文档所有者可以移动文档".to_string()));
    }

    // 如果目标文件夹存在，检查文件夹所有权
    if let Some(folder_id) = move_data.folder_id {
        if !FolderRepository::is_owner(&pool, folder_id, user.user_id).await? {
            return Err(AppError::Forbidden("无权将文档移动到该文件夹".to_string()));
        }
    }

    // 移动文档
    sqlx::query("UPDATE documents SET folder_id = $1 WHERE id = $2")
        .bind(move_data.folder_id)
        .bind(doc_id)
        .execute(&pool)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// 文件夹路由
pub fn folder_routes() -> Router<PgPool> {
    Router::new()
        // 文件夹 CRUD
        .route("/folders", post(create_folder))
        .route("/folders/:id", get(get_folder))
        .route("/folders/:id", put(update_folder))
        .route("/folders/:id", delete(delete_folder))
        // 文件夹树和内容
        .route("/folders/tree", get(get_folder_tree))
        .route("/folders/:id/contents", get(get_folder_contents))
        // 移动文档
        .route("/documents/:id/move", put(move_document))
}
