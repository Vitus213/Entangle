use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 文档版本数据库模型
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DocumentVersion {
    pub id: Uuid,
    pub doc_id: Uuid,
    pub version_number: i32,
    pub title: String,
    pub content: String,
    #[serde(skip_serializing)]
    #[sqlx(default)]
    pub crdt_state: Option<Vec<u8>>,
    pub created_by: Uuid,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 创建版本请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVersion {
    pub doc_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// 版本创建者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionCreator {
    pub id: Uuid,
    pub nickname: String,
}

/// 版本响应（包含创建者信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionResponse {
    pub id: Uuid,
    pub doc_id: Uuid,
    pub version_number: i32,
    pub title: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crdt_state: Option<String>, // 十六进制编码
    pub creator: VersionCreator,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 版本列表项（不包含 content 和 crdt_state）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionListItem {
    pub id: Uuid,
    pub doc_id: Uuid,
    pub version_number: i32,
    pub title: String,
    pub creator: VersionCreator,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 版本对比结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDiff {
    pub version_a: VersionListItem,
    pub version_b: VersionListItem,
    pub title_changed: bool,
    pub content_diff: Vec<DiffLine>,
}

/// 差异行
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub line_number: i32,
    pub operation: DiffOperation,
    pub content: String,
}

/// 差异操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiffOperation {
    Add,    // 新增
    Remove, // 删除
    Equal,  // 不变
}

/// 回滚请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackRequest {
    pub version_id: Uuid,
}
