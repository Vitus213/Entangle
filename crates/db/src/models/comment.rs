use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 评论位置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentPosition {
    pub start: i32,
    pub end: i32,
}

/// 评论数据库模型
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Comment {
    pub id: Uuid,
    pub doc_id: Uuid,
    pub user_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub content: String,
    pub position: Option<sqlx::types::Json<CommentPosition>>,
    pub is_resolved: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 创建评论请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateComment {
    pub doc_id: Uuid,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<CommentPosition>,
}

/// 更新评论请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateComment {
    pub content: String,
}

/// 评论用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentUser {
    pub id: Uuid,
    pub nickname: String,
    pub avatar_url: Option<String>,
}

/// 评论响应（包含用户信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentResponse {
    pub id: Uuid,
    pub doc_id: Uuid,
    pub user: CommentUser,
    pub parent_id: Option<Uuid>,
    pub content: String,
    pub position: Option<CommentPosition>,
    pub is_resolved: bool,
    pub replies: Vec<CommentResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 评论列表项（不包含嵌套回复）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentListItem {
    pub id: Uuid,
    pub doc_id: Uuid,
    pub user: CommentUser,
    pub parent_id: Option<Uuid>,
    pub content: String,
    pub position: Option<CommentPosition>,
    pub is_resolved: bool,
    pub reply_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
