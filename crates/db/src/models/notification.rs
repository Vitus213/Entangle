use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 通知类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    Comment,     // 评论通知
    Mention,     // @提及
    Task,        // 任务相关
    Share,       // 分享/邀请
    System,      // 系统通知
}

impl std::fmt::Display for NotificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationType::Comment => write!(f, "comment"),
            NotificationType::Mention => write!(f, "mention"),
            NotificationType::Task => write!(f, "task"),
            NotificationType::Share => write!(f, "share"),
            NotificationType::System => write!(f, "system"),
        }
    }
}

/// 资源类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    Document,
    Comment,
    Task,
}

impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceType::Document => write!(f, "document"),
            ResourceType::Comment => write!(f, "comment"),
            ResourceType::Task => write!(f, "task"),
        }
    }
}

/// 通知数据库模型
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    #[sqlx(rename = "type")]
    pub notification_type: String,
    pub title: String,
    pub content: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub sender_id: Option<Uuid>,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

/// 创建通知请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNotification {
    pub user_id: Uuid,
    pub notification_type: NotificationType,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<ResourceType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_id: Option<Uuid>,
}

/// 发送者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSender {
    pub id: Uuid,
    pub nickname: String,
    pub avatar_url: Option<String>,
}

/// 通知响应（包含发送者信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResponse {
    pub id: Uuid,
    pub notification_type: String,
    pub title: String,
    pub content: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub sender: Option<NotificationSender>,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

/// 未读数量响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnreadCountResponse {
    pub count: i64,
}
