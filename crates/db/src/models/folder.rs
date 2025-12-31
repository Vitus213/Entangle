use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 文件夹
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Folder {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 创建文件夹请求
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFolder {
    pub name: String,
    pub parent_id: Option<Uuid>,
}

/// 更新文件夹请求
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateFolder {
    pub name: Option<String>,
    pub parent_id: Option<Uuid>,
}

/// 文件夹响应（包含所有者信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderResponse {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub owner: OwnerInfo,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OwnerInfo {
    pub id: Uuid,
    pub nickname: String,
    pub email: String,
}

/// 文件夹树节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderTree {
    #[serde(flatten)]
    pub folder: Folder,
    pub children: Vec<FolderTree>,
    pub document_count: i64,
}

/// 文件夹内容
#[derive(Debug, Serialize, Deserialize)]
pub struct FolderContents {
    pub folder: FolderInfo,
    pub subfolders: Vec<FolderSummary>,
    pub documents: Vec<super::document::DocumentListItem>,
}

/// 文件夹信息（包含路径）
#[derive(Debug, Serialize, Deserialize)]
pub struct FolderInfo {
    pub id: Uuid,
    pub name: String,
    pub path: Vec<String>,
}

/// 文件夹摘要
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct FolderSummary {
    pub id: Uuid,
    pub name: String,
    pub document_count: i64,
}

/// 移动文档请求
#[derive(Debug, Serialize, Deserialize)]
pub struct MoveDocument {
    pub folder_id: Option<Uuid>,
}
