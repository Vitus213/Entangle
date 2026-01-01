use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 标签基础模型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 创建标签请求
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTag {
    pub name: String,
    #[serde(default = "default_color")]
    pub color: String,
}

fn default_color() -> String {
    "#3B82F6".to_string()
}

/// 更新标签请求
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTag {
    pub name: Option<String>,
    pub color: Option<String>,
}

/// 标签（带文档数量）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagWithCount {
    #[serde(flatten)]
    pub tag: Tag,
    pub document_count: i64,
}

/// 标签摘要（用于文档标签列表）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TagSummary {
    pub id: Uuid,
    pub name: String,
    pub color: String,
}

/// 为文档添加标签请求
#[derive(Debug, Serialize, Deserialize)]
pub struct AddTagToDocument {
    pub tag_id: Uuid,
}

/// 批量设置文档标签请求
#[derive(Debug, Serialize, Deserialize)]
pub struct SetDocumentTags {
    pub tag_ids: Vec<Uuid>,
}

/// 文档标签关联
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DocumentTag {
    pub document_id: Uuid,
    pub tag_id: Uuid,
    pub created_at: DateTime<Utc>,
}
