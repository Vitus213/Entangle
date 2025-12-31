use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 文档模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Document {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub owner_id: Uuid,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 创建文档请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDocument {
    pub title: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub is_public: bool,
}

/// 更新文档请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDocument {
    pub title: Option<String>,
    pub content: Option<String>,
    pub is_public: Option<bool>,
}

/// 文档响应（包含作者信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentResponse {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub owner: DocumentOwner,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 文档作者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentOwner {
    pub id: Uuid,
    pub nickname: String,
    pub email: String,
}

/// 文档列表项（不包含 content）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentListItem {
    pub id: Uuid,
    pub title: String,
    pub owner: DocumentOwner,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 文档协作者
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DocumentCollaborator {
    pub document_id: Uuid,
    pub user_id: Uuid,
    pub permission: CollaboratorPermission,
    pub created_at: DateTime<Utc>,
}

/// 协作者权限
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum CollaboratorPermission {
    #[serde(rename = "read")]
    Read,
    #[serde(rename = "write")]
    Write,
    #[serde(rename = "admin")]
    Admin,
}

impl std::fmt::Display for CollaboratorPermission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollaboratorPermission::Read => write!(f, "read"),
            CollaboratorPermission::Write => write!(f, "write"),
            CollaboratorPermission::Admin => write!(f, "admin"),
        }
    }
}

/// 添加协作者请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddCollaborator {
    pub user_id: Uuid,
    pub permission: CollaboratorPermission,
}
