use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use gloo_net::http::Request;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use std::rc::Rc;
use std::cell::RefCell;

mod crdt;
use crdt::{CrdtManager, bytes_to_hex, hex_to_bytes};

// ===== 共享类型 =====

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RegisterRequest {
    email: String,
    password: String,
    nickname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    phone: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AuthResponse {
    token: String,
    user: UserResponse,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct UserResponse {
    id: String,
    email: String,
    nickname: String,
    avatar_url: Option<String>,
    role: Option<String>,
    email_verified: bool,
    created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Document {
    id: String,
    title: String,
    content: String,
    owner: DocumentOwner,
    is_public: bool,
    created_at: String,
    updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    crdt_state: Option<String>, // 十六进制编码的 CRDT 状态
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DocumentOwner {
    id: String,
    nickname: String,
    email: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DocumentListItem {
    id: String,
    title: String,
    owner: DocumentOwner,
    is_public: bool,
    created_at: String,
    updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CreateDocumentRequest {
    title: String,
    content: String,
    is_public: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    folder_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct UpdateDocumentRequest {
    title: String,
    content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Folder {
    id: String,
    name: String,
    parent_id: Option<String>,
    owner_id: String,
    created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct FolderTree {
    id: String,
    name: String,
    parent_id: Option<String>,
    children: Vec<FolderTree>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CreateFolderRequest {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Tag {
    id: String,
    name: String,
    color: String,
    owner_id: String,
    created_at: String,
    updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TagWithCount {
    // 后端使用 #[serde(flatten)]，所以字段是扁平化的
    id: String,
    name: String,
    color: String,
    owner_id: String,
    document_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    updated_at: Option<String>,
}

impl TagWithCount {
    /// 转换为 Tag
    fn to_tag(&self) -> Tag {
        Tag {
            id: self.id.clone(),
            name: self.name.clone(),
            color: self.color.clone(),
            owner_id: self.owner_id.clone(),
            created_at: self.created_at.clone().unwrap_or_default(),
            updated_at: self.updated_at.clone().unwrap_or_default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CreateTagRequest {
    name: String,
    color: String,
}

// 协作者相关类型
#[derive(Clone, Debug, Serialize, Deserialize)]
struct CollaboratorResponse {
    user_id: String,
    nickname: String,
    email: String,
    permission: CollaboratorPermission,
    created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum CollaboratorPermission {
    Read,
    Write,
    Admin,
}

impl std::fmt::Display for CollaboratorPermission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollaboratorPermission::Read => write!(f, "只读"),
            CollaboratorPermission::Write => write!(f, "编辑"),
            CollaboratorPermission::Admin => write!(f, "管理"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AddCollaboratorRequest {
    email: String,
    permission: CollaboratorPermission,
}

// ===== 评论系统类型 =====

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CommentPosition {
    start: i32,
    end: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CommentUser {
    id: String,
    nickname: String,
    avatar_url: Option<String>,
}

/// 评论列表项（用于获取文档评论列表）
#[derive(Clone, Debug, Serialize, Deserialize)]
struct CommentListItem {
    id: String,
    doc_id: String,
    user: CommentUser,
    parent_id: Option<String>,
    content: String,
    position: Option<CommentPosition>,
    is_resolved: bool,
    reply_count: i64,
    created_at: String,
    updated_at: String,
}

/// 评论响应（用于创建/更新评论的响应，包含嵌套回复）
#[derive(Clone, Debug, Serialize, Deserialize)]
struct CommentResponse {
    id: String,
    doc_id: String,
    user: CommentUser,
    parent_id: Option<String>,
    content: String,
    position: Option<CommentPosition>,
    is_resolved: bool,
    #[serde(default)]
    replies: Vec<CommentResponse>,
    created_at: String,
    updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CreateCommentRequest {
    doc_id: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    position: Option<CommentPosition>,
}

// ===== 通知系统类型 =====

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum NotificationType {
    Comment,
    Mention,
    Task,
    Share,
    System,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct NotificationSender {
    id: String,
    nickname: String,
    avatar_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Notification {
    id: String,
    notification_type: String,
    title: String,
    content: Option<String>,
    resource_type: Option<String>,
    resource_id: Option<String>,
    sender: Option<NotificationSender>,
    is_read: bool,
    created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct UnreadCount {
    count: i32,
}

// ===== 用户管理类型 =====

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Role {
    id: String,
    name: String,
    description: Option<String>,
    is_system: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ManageableUser {
    id: String,
    email: String,
    nickname: String,
    avatar_url: Option<String>,
    role: Option<String>,
    email_verified: bool,
    status: String,
    created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct UpdateUserRoleRequest {
    role_id: String,
}

// ===== 用户更新类型 =====

#[derive(Clone, Debug, Serialize, Deserialize)]
struct UpdateUserRequest {
    nickname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    phone: Option<String>,
}

// ===== 搜索结果类型 =====

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SearchResult {
    id: String,
    title: String,
    content: String,
    owner: DocumentOwner,
    is_public: bool,
    updated_at: String,
    highlights: Option<Vec<SearchHighlight>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SearchHighlight {
    text: String,
    position: usize,
}

// ===== 任务系统类型 =====

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

impl TaskStatus {
    fn display(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "待处理",
            TaskStatus::InProgress => "进行中",
            TaskStatus::Completed => "已完成",
            TaskStatus::Cancelled => "已取消",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TaskPriority {
    Low,
    Medium,
    High,
    Urgent,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Medium
    }
}

impl TaskPriority {
    fn display(&self) -> &'static str {
        match self {
            TaskPriority::Low => "低",
            TaskPriority::Medium => "中",
            TaskPriority::High => "高",
            TaskPriority::Urgent => "紧急",
        }
    }

    fn color(&self) -> &'static str {
        match self {
            TaskPriority::Low => "#6B7280",
            TaskPriority::Medium => "#3B82F6",
            TaskPriority::High => "#F59E0B",
            TaskPriority::Urgent => "#EF4444",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TaskUser {
    id: String,
    nickname: String,
    avatar_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Task {
    id: String,
    doc_id: Option<String>,
    doc_title: Option<String>,
    title: String,
    description: Option<String>,
    assignee: Option<TaskUser>,
    created_by: TaskUser,
    status: String,
    priority: String,
    due_date: Option<String>,
    completed_at: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TaskListItem {
    id: String,
    doc_id: Option<String>,
    doc_title: Option<String>,
    title: String,
    assignee: Option<TaskUser>,
    status: String,
    priority: String,
    due_date: Option<String>,
    created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CreateTaskRequest {
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    doc_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    assignee_id: Option<String>,
    #[serde(default)]
    priority: TaskPriority,
    #[serde(skip_serializing_if = "Option::is_none")]
    due_date: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct UpdateTaskStatusRequest {
    status: TaskStatus,
}

// WebSocket 消息类型
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum WsMessage {
    /// 同步文档更新（CRDT 更新）
    Sync { update: String },
    /// 用户感知状态（光标位置等）
    Awareness { state: AwarenessState },
    /// 用户加入
    UserJoined { user_id: String, nickname: String },
    /// 用户离开
    UserLeft { user_id: String },
    /// 错误消息
    Error { message: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AwarenessState {
    user_id: String,
    nickname: Option<String>,
    cursor: Option<CursorPosition>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CursorPosition {
    line: usize,
    column: usize,
}

// ===== LocalStorage 辅助函数 =====

const API_BASE: &str = "http://127.0.0.1:3000";
const WS_BASE: &str = "ws://127.0.0.1:3000";

fn get_token() -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    storage.get_item("token").ok()?
}

fn save_token(token: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("token", token);
        }
    }
}

fn clear_token() {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item("token");
        }
    }
}

// ===== API 客户端 =====

async fn register_api(email: String, password: String, nickname: String) -> Result<AuthResponse, String> {
    let response = Request::post(&format!("{}/api/auth/register", API_BASE))
        .json(&RegisterRequest { email, password, nickname, phone: None })
        .map_err(|e| format!("请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(format!("注册失败 ({}): {}", status, body))
    }
}

async fn login_api(email: String, password: String) -> Result<AuthResponse, String> {
    let response = Request::post(&format!("{}/api/auth/login", API_BASE))
        .json(&LoginRequest { email, password })
        .map_err(|e| format!("请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("登录失败: {}", response.status()))
    }
}

async fn fetch_documents(token: &str) -> Result<Vec<DocumentListItem>, String> {
    let response = Request::get(&format!("{}/api/documents/accessible", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("获取文档失败: {}", response.status()))
    }
}

async fn create_document_api(token: &str, title: String) -> Result<Document, String> {
    let response = Request::post(&format!("{}/api/documents", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .json(&CreateDocumentRequest {
            title,
            content: String::new(),
            is_public: false,
            folder_id: None,
        })
        .map_err(|e| format!("请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("创建失败: {}", response.status()))
    }
}

async fn fetch_document(token: &str, id: &str) -> Result<Document, String> {
    let response = Request::get(&format!("{}/api/documents/{}", API_BASE, id))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("获取文档失败: {}", response.status()))
    }
}

async fn update_document_api(token: &str, id: &str, title: String, content: String) -> Result<Document, String> {
    let response = Request::put(&format!("{}/api/documents/{}", API_BASE, id))
        .header("Authorization", &format!("Bearer {}", token))
        .json(&UpdateDocumentRequest { title, content })
        .map_err(|e| format!("请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("更新失败: {}", response.status()))
    }
}

// 文件夹相关 API
async fn fetch_folder_tree(token: &str) -> Result<Vec<FolderTree>, String> {
    let response = Request::get(&format!("{}/api/folders/tree", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("获取文件夹失败: {}", response.status()))
    }
}

async fn create_folder_api(token: &str, name: String, parent_id: Option<String>) -> Result<Folder, String> {
    let response = Request::post(&format!("{}/api/folders", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .json(&CreateFolderRequest { name, parent_id })
        .map_err(|e| format!("请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("创建文件夹失败: {}", response.status()))
    }
}

// 标签相关 API
async fn fetch_tags(token: &str) -> Result<Vec<TagWithCount>, String> {
    let response = Request::get(&format!("{}/api/tags", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("获取标签失败: {}", response.status()))
    }
}

async fn create_tag_api(token: &str, name: String, color: String) -> Result<TagWithCount, String> {
    let response = Request::post(&format!("{}/api/tags", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .json(&CreateTagRequest { name, color })
        .map_err(|e| format!("请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("创建标签失败: {}", response.status()))
    }
}

// 协作者相关 API
async fn fetch_collaborators(token: &str, doc_id: &str) -> Result<Vec<CollaboratorResponse>, String> {
    let response = Request::get(&format!("{}/api/documents/{}/collaborators", API_BASE, doc_id))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("获取协作者失败: {}", response.status()))
    }
}

async fn add_collaborator_api(
    token: &str,
    doc_id: &str,
    email: String,
    permission: CollaboratorPermission,
) -> Result<(), String> {
    let response = Request::post(&format!("{}/api/documents/{}/collaborators", API_BASE, doc_id))
        .header("Authorization", &format!("Bearer {}", token))
        .json(&AddCollaboratorRequest { email, permission })
        .map_err(|e| format!("请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        Ok(())
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(format!("添加协作者失败 ({}): {}", status, body))
    }
}

async fn remove_collaborator_api(token: &str, doc_id: &str, user_id: &str) -> Result<(), String> {
    let response = Request::delete(&format!(
        "{}/api/documents/{}/collaborators/{}",
        API_BASE, doc_id, user_id
    ))
    .header("Authorization", &format!("Bearer {}", token))
    .send()
    .await
    .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        Ok(())
    } else {
        Err(format!("删除协作者失败: {}", response.status()))
    }
}

// ===== 评论 API =====

async fn fetch_comments(token: &str, doc_id: &str) -> Result<Vec<CommentListItem>, String> {
    let response = Request::get(&format!("{}/api/documents/{}/comments", API_BASE, doc_id))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("获取评论失败: {}", response.status()))
    }
}

async fn create_comment_api(token: &str, request: CreateCommentRequest) -> Result<CommentResponse, String> {
    let response = Request::post(&format!("{}/api/comments", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .json(&request)
        .map_err(|e| format!("请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(format!("创建评论失败 ({}): {}", status, body))
    }
}

async fn resolve_comment_api(token: &str, comment_id: &str) -> Result<CommentResponse, String> {
    let response = Request::put(&format!("{}/api/comments/{}/resolve", API_BASE, comment_id))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("操作失败: {}", response.status()))
    }
}

async fn unresolve_comment_api(token: &str, comment_id: &str) -> Result<CommentResponse, String> {
    let response = Request::put(&format!("{}/api/comments/{}/unresolve", API_BASE, comment_id))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("操作失败: {}", response.status()))
    }
}

async fn delete_comment_api(token: &str, comment_id: &str) -> Result<(), String> {
    let response = Request::delete(&format!("{}/api/comments/{}", API_BASE, comment_id))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        Ok(())
    } else {
        Err(format!("删除评论失败: {}", response.status()))
    }
}

// ===== 通知 API =====

async fn fetch_notifications(token: &str) -> Result<Vec<Notification>, String> {
    let response = Request::get(&format!("{}/api/notifications?limit=50", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("获取通知失败: {}", response.status()))
    }
}

async fn fetch_unread_count(token: &str) -> Result<UnreadCount, String> {
    let response = Request::get(&format!("{}/api/notifications/unread-count", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("获取未读数失败: {}", response.status()))
    }
}

async fn mark_notification_read(token: &str, notification_id: &str) -> Result<(), String> {
    let response = Request::put(&format!("{}/api/notifications/{}/read", API_BASE, notification_id))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        Ok(())
    } else {
        Err(format!("标记已读失败: {}", response.status()))
    }
}

async fn mark_all_notifications_read(token: &str) -> Result<i32, String> {
    let response = Request::put(&format!("{}/api/notifications/read-all", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        #[derive(Deserialize)]
        struct MarkAllResponse {
            marked_count: i32,
        }
        response.json().await.map_err(|e| format!("解析失败: {}", e)).map(|r: MarkAllResponse| r.marked_count)
    } else {
        Err(format!("标记全部已读失败: {}", response.status()))
    }
}

async fn delete_notification(token: &str, notification_id: &str) -> Result<(), String> {
    let response = Request::delete(&format!("{}/api/notifications/{}", API_BASE, notification_id))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        Ok(())
    } else {
        Err(format!("删除通知失败: {}", response.status()))
    }
}

// ===== 任务 API =====

async fn fetch_tasks(token: &str, filter: &str) -> Result<Vec<TaskListItem>, String> {
    let response = Request::get(&format!("{}/api/tasks?filter={}&limit=50", API_BASE, filter))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("获取任务失败: {}", response.status()))
    }
}

async fn create_task_api(token: &str, request: CreateTaskRequest) -> Result<Task, String> {
    let response = Request::post(&format!("{}/api/tasks", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .json(&request)
        .map_err(|e| format!("请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(format!("创建任务失败 ({}): {}", status, body))
    }
}

async fn update_task_status_api(token: &str, task_id: &str, status: TaskStatus) -> Result<Task, String> {
    let response = Request::put(&format!("{}/api/tasks/{}/status", API_BASE, task_id))
        .header("Authorization", &format!("Bearer {}", token))
        .json(&UpdateTaskStatusRequest { status })
        .map_err(|e| format!("请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("更新任务状态失败: {}", response.status()))
    }
}

async fn delete_task_api(token: &str, task_id: &str) -> Result<(), String> {
    let response = Request::delete(&format!("{}/api/tasks/{}", API_BASE, task_id))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        Ok(())
    } else {
        Err(format!("删除任务失败: {}", response.status()))
    }
}

// ===== 用户更新 API =====

async fn fetch_current_user(token: &str) -> Result<UserResponse, String> {
    let response = Request::get(&format!("{}/api/users/me", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("获取用户信息失败: {}", response.status()))
    }
}

async fn update_user_api(token: &str, request: UpdateUserRequest) -> Result<UserResponse, String> {
    let response = Request::put(&format!("{}/api/users/me", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .json(&request)
        .map_err(|e| format!("请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(format!("更新用户信息失败 ({}): {}", status, body))
    }
}

async fn upload_avatar_api(token: &str, file_data: Vec<u8>, filename: &str) -> Result<String, String> {
    let boundary = "----formdata-boundary-multipart-upload";

    let mut body = Vec::new();
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(format!("Content-Disposition: form-data; name=\"avatar\"; filename=\"{}\"\r\n", filename).as_bytes());
    body.extend_from_slice(b"Content-Type: image/jpeg\r\n\r\n");
    body.extend_from_slice(&file_data);
    body.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());

    let response = Request::post(&format!("{}/api/users/me/avatar", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Type", &format!("multipart/form-data; boundary={}", boundary))
        .body(body)
        .map_err(|e| format!("请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        #[derive(Deserialize)]
        struct AvatarResponse {
            avatar_url: String,
        }
        response.json().await.map_err(|e| format!("解析失败: {}", e)).map(|r: AvatarResponse| r.avatar_url)
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(format!("上传头像失败 ({}): {}", status, body))
    }
}

// ===== 用户管理 API =====

async fn fetch_all_users(token: &str) -> Result<Vec<ManageableUser>, String> {
    let response = Request::get(&format!("{}/api/users?limit=100", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("获取用户列表失败: {}", response.status()))
    }
}

async fn fetch_all_roles(token: &str) -> Result<Vec<Role>, String> {
    let response = Request::get(&format!("{}/api/roles", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("获取角色列表失败: {}", response.status()))
    }
}

async fn update_user_role_api(token: &str, user_id: &str, role_id: &str) -> Result<(), String> {
    let request = UpdateUserRoleRequest {
        role_id: role_id.to_string(),
    };

    let response = Request::post(&format!("{}/api/users/{}/role", API_BASE, user_id))
        .header("Authorization", &format!("Bearer {}", token))
        .json(&request)
        .map_err(|e| format!("请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        Ok(())
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(format!("更新用户角色失败 ({}): {}", status, body))
    }
}

// ===== 搜索 API =====

async fn search_documents_api(token: &str, query: &str) -> Result<Vec<DocumentListItem>, String> {
    let response = Request::get(&format!("{}/api/documents/search?q={}&limit=20", API_BASE, urlencoding::encode(query)))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("搜索失败: {}", response.status()))
    }
}

// ===== 注册页面 =====

#[component]
fn RegisterPage() -> impl IntoView {
    let (email, set_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (nickname, set_nickname) = create_signal(String::new());
    let (error, set_error) = create_signal(None::<String>);
    let (loading, set_loading) = create_signal(false);

    let navigate = use_navigate();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error.set(None);

        let email_val = email.get();
        let password_val = password.get();
        let nickname_val = nickname.get();
        let nav = navigate.clone();

        spawn_local(async move {
            match register_api(email_val, password_val, nickname_val).await {
                Ok(response) => {
                    save_token(&response.token);
                    nav("/documents", Default::default());
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="auth-container">
            <h1>"注册 Entangle"</h1>
            <form on:submit=on_submit>
                <div class="form-group">
                    <label>"昵称"</label>
                    <input
                        type="text"
                        placeholder="你的昵称"
                        prop:value=move || nickname.get()
                        on:input=move |ev| set_nickname.set(event_target_value(&ev))
                        required
                    />
                </div>
                <div class="form-group">
                    <label>"邮箱"</label>
                    <input
                        type="email"
                        placeholder="your@email.com"
                        prop:value=move || email.get()
                        on:input=move |ev| set_email.set(event_target_value(&ev))
                        required
                    />
                </div>
                <div class="form-group">
                    <label>"密码"</label>
                    <input
                        type="password"
                        placeholder="至少6位"
                        prop:value=move || password.get()
                        on:input=move |ev| set_password.set(event_target_value(&ev))
                        required
                        minlength="6"
                    />
                </div>
                {move || error.get().map(|e| view! {
                    <div class="error">{e}</div>
                })}
                <button type="submit" class="btn" disabled=move || loading.get()>
                    {move || if loading.get() { "注册中..." } else { "注册" }}
                </button>
                <p class="auth-link">
                    "已有账号？"
                    <a href="/">"去登录"</a>
                </p>
            </form>
        </div>
    }
}

// ===== 登录页面 =====

#[component]
fn LoginPage() -> impl IntoView {
    let (email, set_email) = create_signal(String::from("demo@example.com"));
    let (password, set_password) = create_signal(String::from("demo123"));
    let (error, set_error) = create_signal(None::<String>);
    let (loading, set_loading) = create_signal(false);

    let navigate = use_navigate();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error.set(None);

        let email_val = email.get();
        let password_val = password.get();
        let nav = navigate.clone();

        spawn_local(async move {
            match login_api(email_val, password_val).await {
                Ok(response) => {
                    save_token(&response.token);
                    nav("/documents", Default::default());
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="auth-container">
            <h1>"Entangle 登录"</h1>
            <form on:submit=on_submit>
                <div class="form-group">
                    <label>"邮箱"</label>
                    <input
                        type="email"
                        placeholder="your@email.com"
                        prop:value=move || email.get()
                        on:input=move |ev| set_email.set(event_target_value(&ev))
                        required
                    />
                </div>
                <div class="form-group">
                    <label>"密码"</label>
                    <input
                        type="password"
                        placeholder="••••••••"
                        prop:value=move || password.get()
                        on:input=move |ev| set_password.set(event_target_value(&ev))
                        required
                    />
                </div>
                {move || error.get().map(|e| view! {
                    <div class="error">{e}</div>
                })}
                <button type="submit" class="btn" disabled=move || loading.get()>
                    {move || if loading.get() { "登录中..." } else { "登录" }}
                </button>
                <p class="auth-link">
                    "还没有账号？"
                    <a href="/register">"注册"</a>
                </p>
            </form>
        </div>
    }
}

// ===== 文档列表页面 =====

#[component]
fn DocumentsPage() -> impl IntoView {
    let (documents, set_documents) = create_signal(Vec::<DocumentListItem>::new());
    let (folders, set_folders) = create_signal(Vec::<FolderTree>::new());
    let (tags, set_tags) = create_signal(Vec::<TagWithCount>::new());
    let (loading, set_loading) = create_signal(true);
    let (error, set_error) = create_signal(None::<String>);

    // 当前用户信息
    let (current_user, set_current_user) = create_signal(None::<UserResponse>);

    // 用户管理功能
    let (show_user_management, set_show_user_management) = create_signal(false);
    let (all_users, set_all_users) = create_signal(Vec::<ManageableUser>::new());
    let (all_roles, set_all_roles) = create_signal(Vec::<Role>::new());

    // 搜索功能
    let (search_query, set_search_query) = create_signal(String::new());
    let (search_results, set_search_results) = create_signal(Vec::<DocumentListItem>::new());
    let (is_searching, set_is_searching) = create_signal(false);
    let (show_search_dropdown, set_show_search_dropdown) = create_signal(false);

    // 创建文档
    let (show_create_doc, set_show_create_doc) = create_signal(false);
    let (new_doc_title, set_new_doc_title) = create_signal(String::from("新文档"));

    // 创建文件夹
    let (show_create_folder, set_show_create_folder) = create_signal(false);
    let (new_folder_name, set_new_folder_name) = create_signal(String::new());

    // 创建标签
    let (show_create_tag, set_show_create_tag) = create_signal(false);
    let (new_tag_name, set_new_tag_name) = create_signal(String::new());
    let (new_tag_color, set_new_tag_color) = create_signal(String::from("#3B82F6"));

    // 用户资料编辑
    let (show_profile_modal, set_show_profile_modal) = create_signal(false);
    let (edit_nickname, set_edit_nickname) = create_signal(String::new());
    let (edit_avatar_url, set_edit_avatar_url) = create_signal(String::new());

    let (sidebar_collapsed, _set_sidebar_collapsed) = create_signal(false);

    // 通知系统
    let (notifications, set_notifications) = create_signal(Vec::<Notification>::new());
    let (unread_count, set_unread_count) = create_signal(0);
    let (show_notifications_panel, set_show_notifications_panel) = create_signal(false);

    let navigate = use_navigate();
    let nav_clone1 = navigate.clone();

    // 加载所有数据
    create_effect(move |_| {
        if let Some(token) = get_token() {
            set_loading.set(true);

            // 加载当前用户信息
            let token_clone = token.clone();
            let set_current_user_clone = set_current_user.clone();
            let set_edit_nickname_clone = set_edit_nickname.clone();
            let set_edit_avatar_url_clone = set_edit_avatar_url.clone();
            spawn_local(async move {
                match fetch_current_user(&token_clone).await {
                    Ok(user) => {
                        set_edit_nickname_clone.set(user.nickname.clone());
                        set_edit_avatar_url_clone.set(user.avatar_url.clone().unwrap_or_default());
                        set_current_user_clone.set(Some(user));
                    }
                    Err(e) => leptos::logging::error!("加载用户信息失败: {}", e),
                }
            });

            // 加载文档
            let token_clone = token.clone();
            spawn_local(async move {
                match fetch_documents(&token_clone).await {
                    Ok(docs) => set_documents.set(docs),
                    Err(e) => set_error.set(Some(e)),
                }
            });

            // 加载文件夹
            let token_clone = token.clone();
            spawn_local(async move {
                match fetch_folder_tree(&token_clone).await {
                    Ok(tree) => set_folders.set(tree),
                    Err(e) => leptos::logging::error!("加载文件夹失败: {}", e),
                }
            });

            // 加载标签
            let token_clone = token.clone();
            spawn_local(async move {
                match fetch_tags(&token_clone).await {
                    Ok(tag_list) => set_tags.set(tag_list),
                    Err(e) => leptos::logging::error!("加载标签失败: {}", e),
                }
                set_loading.set(false);
            });

            // 加载通知
            let set_notifications_clone = set_notifications.clone();
            let set_unread_count_clone = set_unread_count.clone();
            spawn_local(async move {
                match fetch_notifications(&token).await {
                    Ok(notifs) => {
                        // 计算未读数
                        let unread = notifs.iter().filter(|n| !n.is_read).count() as i32;
                        set_unread_count_clone.set(unread);
                        set_notifications_clone.set(notifs);
                    }
                    Err(e) => leptos::logging::error!("加载通知失败: {}", e),
                }
            });
        } else {
            nav_clone1("/", Default::default());
        }
    });

    // ===== 搜索功能 =====

    let perform_search = move |query: String| {
        let query_trimmed = query.trim().to_string();
        if query_trimmed.is_empty() {
            set_search_results.set(Vec::new());
            set_show_search_dropdown.set(false);
            return;
        }

        if let Some(token) = get_token() {
            set_is_searching.set(true);
            let set_search_results_clone = set_search_results.clone();
            spawn_local(async move {
                match search_documents_api(&token, &query_trimmed).await {
                    Ok(results) => {
                        set_search_results_clone.set(results);
                        set_show_search_dropdown.set(true);
                    }
                    Err(e) => leptos::logging::error!("搜索失败: {}", e),
                }
                set_is_searching.set(false);
            });
        }
    };

    // 防抖搜索
    let search_debounce_timer = create_signal(None::<i32>);
    let on_search_input = move |ev| {
        let query = event_target_value(&ev);
        set_search_query.set(query.clone());

        // 清除之前的定时器
        if let Some(timer_id) = search_debounce_timer.0.get() {
            web_sys::window()
                .unwrap()
                .clear_timeout_with_handle(timer_id);
        }

        // 设置新的防抖定时器（300ms）
        let perform_search_clone = perform_search.clone();
        let callback = Closure::wrap(Box::new(move || {
            perform_search_clone(query.clone());
        }) as Box<dyn Fn()>);

        if let Some(window) = web_sys::window() {
            let timer_id = window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    callback.as_ref().unchecked_ref(),
                    300,
                )
                .unwrap();
            search_debounce_timer.1.set(Some(timer_id));
            callback.forget();
        }
    };

    // ===== 用户资料更新 =====

    let open_profile_modal = move |_: web_sys::MouseEvent| {
        if let Some(user) = current_user.get() {
            set_edit_nickname.set(user.nickname.clone());
            set_edit_avatar_url.set(user.avatar_url.clone().unwrap_or_default());
        }
        set_show_profile_modal.set(true);
    };

    let update_profile = move |_: web_sys::MouseEvent| {
        if let Some(token) = get_token() {
            let nickname = edit_nickname.get();
            let avatar_url = edit_avatar_url.get();
            let avatar_url_opt = if avatar_url.is_empty() { None } else { Some(avatar_url) };

            let request = UpdateUserRequest {
                nickname,
                avatar_url: avatar_url_opt,
                phone: None,
            };

            let set_current_user_clone = set_current_user.clone();
            spawn_local(async move {
                match update_user_api(&token, request).await {
                    Ok(updated_user) => {
                        set_current_user_clone.set(Some(updated_user));
                        set_show_profile_modal.set(false);
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    };

    // 创建文档
    let nav_for_create = navigate.clone();
    let create_document = move |_: web_sys::MouseEvent| {
        if let Some(token) = get_token() {
            let title = new_doc_title.get();
            let nav = nav_for_create.clone();
            spawn_local(async move {
                match create_document_api(&token, title).await {
                    Ok(doc) => nav(&format!("/editor/{}", doc.id), Default::default()),
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    };

    // 创建文件夹
    let create_folder = move |_: web_sys::MouseEvent| {
        if let Some(token) = get_token() {
            let name = new_folder_name.get();
            spawn_local(async move {
                match create_folder_api(&token, name, None).await {
                    Ok(_) => {
                        set_show_create_folder.set(false);
                        set_new_folder_name.set(String::new());
                        // 重新加载文件夹树
                        if let Ok(tree) = fetch_folder_tree(&token).await {
                            set_folders.set(tree);
                        }
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    };

    // 创建标签
    let create_tag = move |_: web_sys::MouseEvent| {
        if let Some(token) = get_token() {
            let name = new_tag_name.get();
            let color = new_tag_color.get();
            spawn_local(async move {
                match create_tag_api(&token, name, color).await {
                    Ok(tag_with_count) => {
                        set_show_create_tag.set(false);
                        set_new_tag_name.set(String::new());
                        // 添加到标签列表
                        let mut current_tags = tags.get();
                        current_tags.push(tag_with_count);
                        set_tags.set(current_tags);
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    };

    // ===== 通知操作 =====

    // 标记通知为已读
    let mark_notification_read_fn = move |notification_id: String| {
        if let Some(token) = get_token() {
            let notifications_rc = notifications.clone();
            let set_notifications_clone = set_notifications.clone();
            let set_unread_count_clone = set_unread_count.clone();
            spawn_local(async move {
                match mark_notification_read(&token, &notification_id).await {
                    Ok(_) => {
                        // 更新通知状态
                        let current = notifications_rc.get();
                        let updated: Vec<_> = current.into_iter().map(|mut n| {
                            if n.id == notification_id {
                                n.is_read = true;
                            }
                            n
                        }).collect();
                        // 更新未读数
                        let unread = updated.iter().filter(|n| !n.is_read).count() as i32;
                        set_notifications_clone.set(updated);
                        set_unread_count_clone.set(unread);
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    };

    // 标记所有通知为已读
    let mark_all_read = move |_: web_sys::MouseEvent| {
        if let Some(token) = get_token() {
            let notifications_rc = notifications.clone();
            let set_notifications_clone = set_notifications.clone();
            let set_unread_count_clone = set_unread_count.clone();
            spawn_local(async move {
                match mark_all_notifications_read(&token).await {
                    Ok(_) => {
                        let mut current = notifications_rc.get();
                        for n in &mut current {
                            n.is_read = true;
                        }
                        set_notifications_clone.set(current);
                        set_unread_count_clone.set(0);
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    };

    // 删除通知
    let delete_notification_fn = move |notification_id: String| {
        if let Some(token) = get_token() {
            let notifications_rc = notifications.clone();
            let set_notifications_clone = set_notifications.clone();
            let set_unread_count_clone = set_unread_count.clone();
            spawn_local(async move {
                match delete_notification(&token, &notification_id).await {
                    Ok(_) => {
                        let current = notifications_rc.get();
                        let updated: Vec<_> = current.into_iter().filter(|n| n.id != notification_id).collect();
                        let unread = updated.iter().filter(|n| !n.is_read).count() as i32;
                        set_notifications_clone.set(updated);
                        set_unread_count_clone.set(unread);
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    };

    // 登出
    let logout = move |_: web_sys::MouseEvent| {
        clear_token();
        // 使用 window.location 而不是 navigate 避免 FnOnce 问题
        if let Some(window) = web_sys::window() {
            let _ = window.location().assign("/");
        }
    };

    view! {
        <div class="app-container">
            // 顶部导航栏
            <div class="navbar" style="position: relative;">
                <div class="navbar-brand">
                    <h1>"Entangle"</h1>
                </div>

                // 搜索框（居中）
                <div style="position: absolute; left: 50%; transform: translateX(-50%); width: 400px; max-width: 50%;">
                    <div style="position: relative;">
                        <input
                            type="text"
                            placeholder="搜索文档..."
                            prop:value=move || search_query.get()
                            on:input=on_search_input
                            on:focus=move |_| set_show_search_dropdown.set(true)
                            style="width: 100%; padding: 8px 36px 8px 12px; border: 1px solid #E5E6EB; border-radius: 20px; font-size: 14px; background: #F5F6F7;"
                        />
                        <span style="position: absolute; right: 12px; top: 50%; transform: translateY(-50%); color: #86909C;">"🔍"</span>

                        // 搜索结果下拉框
                        <div style="position: relative;">
                            {move || {
                                let show = show_search_dropdown.get();
                                let query = search_query.get();
                                let searching = is_searching.get();
                                let results = search_results.get();

                                if !show || query.is_empty() {
                                    view! { }.into_view()
                                } else {
                                    let set_show = set_show_search_dropdown.clone();
                                    let set_q = set_search_query.clone();
                                    let set_r = set_search_results.clone();

                                    // 点击文档后导航
                                    let handle_click = move |doc_id: String| {
                                        let ss = set_show.clone();
                                        let sq = set_q.clone();
                                        let sr = set_r.clone();
                                        move |_| {
                                            // 使用 window.location.href 导航
                                            if let Some(window) = web_sys::window() {
                                                let url = format!("/editor/{}", doc_id);
                                                let _ = window.location().assign(&url);
                                            }
                                            ss.set(false);
                                            sq.set(String::new());
                                            sr.set(Vec::new());
                                        }
                                    };

                                    if searching {
                                        view! {
                                            <div style="position: absolute; top: 100%; left: 0; right: 0; margin-top: 4px; background: white; border: 1px solid #E5E6EB; border-radius: 8px; box-shadow: 0 4px 12px rgba(0,0,0,0.1); max-height: 400px; overflow-y: auto; z-index: 1000; padding: 16px; text-align: center; color: #86909C;">
                                                "搜索中..."
                                            </div>
                                        }.into_view()
                                    } else if results.is_empty() {
                                        view! {
                                            <div style="position: absolute; top: 100%; left: 0; right: 0; margin-top: 4px; background: white; border: 1px solid #E5E6EB; border-radius: 8px; box-shadow: 0 4px 12px rgba(0,0,0,0.1); max-height: 400px; overflow-y: auto; z-index: 1000; padding: 16px; text-align: center; color: #86909C;">
                                                "未找到相关文档"
                                            </div>
                                        }.into_view()
                                    } else {
                                        view! {
                                            <div style="position: absolute; top: 100%; left: 0; right: 0; margin-top: 4px; background: white; border: 1px solid #E5E6EB; border-radius: 8px; box-shadow: 0 4px 12px rgba(0,0,0,0.1); max-height: 400px; overflow-y: auto; z-index: 1000;"
                                                on:click=move |ev| ev.stop_propagation()
                                                on:mousedown=move |ev| ev.stop_propagation()
                                            >
                                                {results.into_iter().map(|doc| {
                                                    let click_handler = handle_click(doc.id.clone());
                                                    view! {
                                                        <div
                                                            style="padding: 12px 16px; cursor: pointer; border-bottom: 1px solid #F2F3F5;"
                                                            on:click=click_handler
                                                        >
                                                            <div style="font-weight: 500; color: #1D2129; margin-bottom: 4px;">{doc.title}</div>
                                                            <div style="font-size: 12px; color: #86909C;">"作者: "{doc.owner.nickname}</div>
                                                        </div>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        }.into_view()
                                    }
                                }
                            }}
                        </div>
                    </div>
                </div>

                <div class="navbar-actions">
                    // 新建按钮
                    <button class="btn btn-primary" on:click=move |_| set_show_create_doc.set(!show_create_doc.get())>
                        "+ 新建"
                    </button>

                    // 通知按钮
                    <button
                        class="btn btn-secondary"
                        style="position: relative;"
                        on:click=move |_| set_show_notifications_panel.set(!show_notifications_panel.get())
                    >
                        "🔔"
                        {move || if unread_count.get() > 0 {
                            view! {
                                <span style="position: absolute; top: -5px; right: -5px; background: #ef4444; color: white; border-radius: 50%; width: 18px; height: 18px; display: flex; align-items: center; justify-content: center; font-size: 11px;">
                                    {unread_count.get()}
                                </span>
                            }
                        } else {
                            view! { <span style="display: none;"></span> }
                        }}
                    </button>

                    // 用户资料下拉菜单（最右边）
                    <div style="position: relative;">
                        <button
                            class="btn btn-secondary"
                            style="display: flex; align-items: center; gap: 8px;"
                            on:click=move |_| set_show_profile_modal.set(!show_profile_modal.get())
                        >
                            {move || {
                                let user_opt = current_user.get();
                                let avatar_html = if let Some(user) = user_opt {
                                    if let Some(ref avatar_url) = user.avatar_url {
                                        if !avatar_url.is_empty() {
                                            format!(r#"<img src="{}" style="width: 24px; height: 24px; border-radius: 50%;">"#, avatar_url)
                                        } else {
                                            r#"<span style="width: 24px; height: 24px; display: flex; align-items: center; justify-content: center;">👤</span>"#.to_string()
                                        }
                                    } else {
                                        r#"<span style="width: 24px; height: 24px; display: flex; align-items: center; justify-content: center;">👤</span>"#.to_string()
                                    }
                                } else {
                                    r#"<span style="width: 24px; height: 24px; display: flex; align-items: center; justify-content: center;">👤</span>"#.to_string()
                                };
                                view! { <div inner_html={avatar_html} style="display: contents;"></div> }
                            }}
                            <span>{move || current_user.get().map(|u| u.nickname).unwrap_or_default()}</span>
                            <span style="font-size: 10px;">"▼"</span>
                        </button>
                    </div>
                </div>
            </div>

            // 用户资料模态框
            {move || show_profile_modal.get().then(|| view! {
                <div
                    style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 2000;"
                    on:click=move |_| set_show_profile_modal.set(false)
                >
                    <div
                        style="background: white; padding: 24px; border-radius: 12px; width: 400px; max-width: 90%;"
                        on:click=move |ev| ev.stop_propagation()
                    >
                        <h2 style="margin: 0 0 20px 0; font-size: 18px;">"编辑个人资料"</h2>

                        // 管理员入口
                        {move || if current_user.get().as_ref().and_then(|u| u.role.as_deref()) == Some("admin") {
                            view! {
                                <div style="margin-bottom: 16px; padding: 12px; background: #F0F9FF; border: 1px solid #3B82F6; border-radius: 6px;">
                                    <div style="display: flex; align-items: center; justify-content: space-between;">
                                        <div>
                                            <strong style="color: #1E40AF;">"管理员权限"</strong>
                                            <div style="font-size: 12px; color: #6B7280; margin-top: 4px;">"您可以管理所有用户和文档"</div>
                                        </div>
                                        <button
                                            class="btn-sm"
                                            style="background: #3B82F6; color: white;"
                                            on:click=move |_| {
                                                set_show_profile_modal.set(false);
                                                // 加载用户列表
                                                let token = get_token().unwrap();
                                                let set_users = set_all_users.clone();
                                                let set_roles = set_all_roles.clone();
                                                spawn_local(async move {
                                                    if let Ok(users) = fetch_all_users(&token).await {
                                                        set_users.set(users);
                                                    }
                                                    if let Ok(roles) = fetch_all_roles(&token).await {
                                                        set_roles.set(roles);
                                                    }
                                                });
                                                set_show_user_management.set(true);
                                            }
                                        >
                                            "用户管理"
                                        </button>
                                    </div>
                                </div>
                            }
                        } else {
                            view! { <div style="display: none;"></div> }
                        }}

                        <div style="margin-bottom: 16px;">
                            <label style="display: block; margin-bottom: 6px; font-size: 14px; font-weight: 500;">"昵称"</label>
                            <input
                                type="text"
                                prop:value=move || edit_nickname.get()
                                on:input=move |ev| set_edit_nickname.set(event_target_value(&ev))
                                style="width: 100%; padding: 10px; border: 1px solid #E5E6EB; border-radius: 6px;"
                            />
                        </div>

                        <div style="margin-bottom: 20px;">
                            <label style="display: block; margin-bottom: 6px; font-size: 14px; font-weight: 500;">"头像 URL"</label>
                            <input
                                type="text"
                                placeholder="https://example.com/avatar.jpg"
                                prop:value=move || edit_avatar_url.get()
                                on:input=move |ev| set_edit_avatar_url.set(event_target_value(&ev))
                                style="width: 100%; padding: 10px; border: 1px solid #E5E6EB; border-radius: 6px;"
                            />
                            <div style="margin-top: 8px; display: flex; align-items: center; gap: 12px;">
                                {move || {
                                    let url = edit_avatar_url.get();
                                    if url.is_empty() {
                                        view! {
                                            <div style="width: 48px; height: 48px; background: #F2F3F5; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-size: 24px;">"👤"</div>
                                        }
                                    } else {
                                        view! {
                                            <div style="width: 48px; height: 48px; border-radius: 50%; overflow: hidden;">
                                                <img src={url} alt="预览" style="width: 100%; height: 100%; object-fit: cover;" />
                                            </div>
                                        }
                                    }
                                }}
                                <span style="font-size: 12px; color: #86909C;">"头像预览"</span>
                            </div>
                        </div>

                        <div style="border-top: 1px solid #E5E6EB; margin-top: 16px; padding-top: 16px;">
                            <button
                                class="btn"
                                style="width: 100%; text-align: left; background: none; color: #ef4444; padding: 10px; border: 1px solid #fee2e2;"
                                on:click=logout
                            >
                                "🚪 登出"
                            </button>
                        </div>
                    </div>
                </div>
            })}

            // 通知面板（浮动）
            {move || show_notifications_panel.get().then(|| view! {
                <div style="position: fixed; top: 70px; right: 20px; width: 350px; max-height: 500px; background: white; border: 1px solid #ddd; border-radius: 8px; box-shadow: 0 4px 12px rgba(0,0,0,0.1); z-index: 1000;">
                    <div style="display: flex; justify-content: space-between; align-items: center; padding: 12px 16px; border-bottom: 1px solid #eee;">
                        <h3 style="margin: 0;">"通知"</h3>
                        <div style="display: flex; gap: 8px;">
                            {move || if unread_count.get() > 0 {
                                view! {
                                    <button
                                        class="btn-sm"
                                        on:click=mark_all_read
                                    >
                                        "全部已读"
                                    </button>
                                }.into_view()
                            } else {
                                view! {}.into_view()
                            }}
                            <button
                                class="btn-close"
                                on:click=move |_| set_show_notifications_panel.set(false)
                            >
                                "×"
                            </button>
                        </div>
                    </div>
                    <div style="max-height: 400px; overflow-y: auto;">
                        {move || if notifications.get().is_empty() {
                            view! { <div style="padding: 20px; text-align: center; color: #999;">"暂无通知"</div> }.into_view()
                        } else {
                            view! {
                                <For
                                    each=move || notifications.get()
                                    key=|notif| notif.id.clone()
                                    children=move |notif: Notification| {
                                        let notif_id = notif.id.clone();
                                        let notif_id_for_delete = notif_id.clone();
                                        let is_read = notif.is_read;
                                        view! {
                                            <div
                                                style=format!("padding: 12px 16px; border-bottom: 1px solid #eee; cursor: pointer; {}",
                                                    if is_read { "background: #f9f9f9; opacity: 0.7;" } else { "background: white;" }
                                                )
                                                on:click=move |_| mark_notification_read_fn(notif_id.clone())
                                            >
                                                <div style="display: flex; justify-content: space-between; align-items: start;">
                                                    <div style="flex: 1;">
                                                        <div style=format!("font-size: 14px; margin-bottom: 4px; font-weight: {};",
                                                            if is_read { "normal" } else { "bold" }
                                                        )>
                                                            {&notif.title}
                                                        </div>
                                                        {notif.content.as_ref().map(|content| view! {
                                                            <div style="font-size: 13px; color: #666; margin-bottom: 4px;">{content}</div>
                                                        })}
                                                        <div style="font-size: 12px; color: #999;">
                                                            {notif.created_at.chars().take(16).collect::<String>()}
                                                        </div>
                                                    </div>
                                                    <button
                                                        class="btn-icon"
                                                        style="padding: 4px;"
                                                        on:click=move |ev| {
                                                            ev.stop_propagation();
                                                            delete_notification_fn(notif_id_for_delete.clone());
                                                        }
                                                    >
                                                        "×"
                                                    </button>
                                                </div>
                                            </div>
                                        }
                                    }
                                />
                            }.into_view()
                        }}
                    </div>
                </div>
            })}

            // 用户管理面板（仅管理员）- 使用简化方式
            {move || show_user_management.get().then(|| {
                let users = all_users.get();
                let roles = all_roles.get();
                view! {
                    <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 3000;">
                        <div
                            style="background: white; padding: 24px; border-radius: 12px; width: 800px; max-width: 90%; max-height: 80vh; overflow-y: auto;"
                            on:click=move |ev| ev.stop_propagation()
                        >
                            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
                                <h2 style="margin: 0;">"用户管理"</h2>
                                <button
                                    style="background: none; border: none; font-size: 24px; cursor: pointer;"
                                    on:click=move |_| set_show_user_management.set(false)
                                >
                                    "×"
                                </button>
                            </div>

                            <div style="overflow-x: auto;">
                                <table style="width: 100%; border-collapse: collapse;">
                                    <thead>
                                        <tr style="background: #F5F6F7;">
                                            <th style="padding: 12px; text-align: left; border-bottom: 1px solid #E5E6EB;">"用户"</th>
                                            <th style="padding: 12px; text-align: left; border-bottom: 1px solid #E5E6EB;">"邮箱"</th>
                                            <th style="padding: 12px; text-align: left; border-bottom: 1px solid #E5E6EB;">"角色"</th>
                                            <th style="padding: 12px; text-align: left; border-bottom: 1px solid #E5E6EB;">"状态"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {users.into_iter().map(|user| {
                                            let user_id = user.id.clone();
                                            let current_role = user.role.clone();
                                            let roles = roles.clone();
                                            view! {
                                                <tr style="border-bottom: 1px solid #F2F3F5;">
                                                    <td style="padding: 12px;">{&user.nickname}</td>
                                                    <td style="padding: 12px; color: #666;">{&user.email}</td>
                                                    <td style="padding: 12px;">
                                                        <select
                                                            style="padding: 6px 10px; border: 1px solid #E5E6EB; border-radius: 4px;"
                                                            on:change=move |ev| {
                                                                let new_role_id = event_target_value(&ev);
                                                                let user_id = user_id.clone();
                                                                let set_users = set_all_users.clone();
                                                                spawn_local(async move {
                                                                    if let Some(token) = get_token() {
                                                                        match update_user_role_api(&token, &user_id, &new_role_id).await {
                                                                            Ok(_) => {
                                                                                if let Ok(updated_users) = fetch_all_users(&token).await {
                                                                                    set_users.set(updated_users);
                                                                                }
                                                                            }
                                                                            Err(e) => leptos::logging::error!("更新角色失败: {}", e),
                                                                        }
                                                                    }
                                                                });
                                                            }
                                                        >
                                                            {roles.iter().map(|role| {
                                                                let is_selected = current_role.as_ref().map_or(false, |r| r == &role.name);
                                                                view! {
                                                                    <option value={role.id.clone()} selected=is_selected>
                                                                        {&role.name}
                                                                    </option>
                                                                }
                                                            }).collect_view()}
                                                        </select>
                                                    </td>
                                                    <td style="padding: 12px;">
                                                        {if user.status == "active" { "活跃" } else { "禁用" }}
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    </div>
                }
            })}

            <div class="main-layout">
                // 侧边栏
                <div class="sidebar" class:collapsed=move || sidebar_collapsed.get()>
                    <div class="sidebar-section">
                        <div class="section-header">
                            <h3>"文件夹"</h3>
                            <button class="btn-icon" on:click=move |_| set_show_create_folder.set(!show_create_folder.get())>
                                "+"
                            </button>
                        </div>

                        {move || show_create_folder.get().then(|| view! {
                            <div class="create-form-inline">
                                <input
                                    type="text"
                                    placeholder="文件夹名称"
                                    prop:value=move || new_folder_name.get()
                                    on:input=move |ev| set_new_folder_name.set(event_target_value(&ev))
                                    class="input-sm"
                                />
                                <button class="btn-sm" on:click=create_folder>"创建"</button>
                                <button class="btn-sm btn-cancel" on:click=move |_| set_show_create_folder.set(false)>"取消"</button>
                            </div>
                        })}

                        <div class="folder-list">
                            <For
                                each=move || folders.get()
                                key=|folder| folder.id.clone()
                                children=|folder: FolderTree| {
                                    view! {
                                        <div class="folder-item">
                                            <span class="folder-icon">"📁"</span>
                                            <span class="folder-name">{&folder.name}</span>
                                        </div>
                                    }
                                }
                            />
                        </div>
                    </div>

                    <div class="sidebar-section">
                        <div class="section-header">
                            <h3>"标签"</h3>
                            <button class="btn-icon" on:click=move |_| set_show_create_tag.set(!show_create_tag.get())>
                                "+"
                            </button>
                        </div>

                        {move || show_create_tag.get().then(|| view! {
                            <div class="create-form-inline">
                                <input
                                    type="text"
                                    placeholder="标签名称"
                                    prop:value=move || new_tag_name.get()
                                    on:input=move |ev| set_new_tag_name.set(event_target_value(&ev))
                                    class="input-sm"
                                />
                                <input
                                    type="color"
                                    prop:value=move || new_tag_color.get()
                                    on:input=move |ev| set_new_tag_color.set(event_target_value(&ev))
                                    class="input-color"
                                />
                                <button class="btn-sm" on:click=create_tag>"创建"</button>
                                <button class="btn-sm btn-cancel" on:click=move |_| set_show_create_tag.set(false)>"取消"</button>
                            </div>
                        })}

                        <div class="tag-list">
                            <For
                                each=move || tags.get()
                                key=|tag| tag.id.clone()
                                children=|tag_with_count: TagWithCount| {
                                    let color = tag_with_count.color.clone();
                                    view! {
                                        <div class="tag-item">
                                            <span class="tag-badge" style=format!("background-color: {}", color)>
                                                {&tag_with_count.name}
                                            </span>
                                            <span class="tag-count">{tag_with_count.document_count}</span>
                                        </div>
                                    }
                                }
                            />
                        </div>
                    </div>
                </div>

                // 主内容区
                <div class="content-area">
                    {move || show_create_doc.get().then(|| {
                        let create_document = create_document.clone();
                        view! {
                            <div class="create-doc-form">
                                <h2>"创建新文档"</h2>
                                <input
                                    type="text"
                                    placeholder="文档标题"
                                    prop:value=move || new_doc_title.get()
                                    on:input=move |ev| set_new_doc_title.set(event_target_value(&ev))
                                    class="input-lg"
                                />
                                <div class="form-actions">
                                    <button class="btn btn-primary" on:click=create_document>"创建"</button>
                                    <button class="btn btn-secondary" on:click=move |_| set_show_create_doc.set(false)>"取消"</button>
                                </div>
                            </div>
                        }
                    })}

                    {
                        let nav_for_docs = navigate.clone();
                        move || if loading.get() {
                            view! { <div class="loading">"加载中..."</div> }.into_view()
                        } else if let Some(err) = error.get() {
                            view! { <div class="error">{err}</div> }.into_view()
                        } else {
                            let docs = documents.get();
                            if docs.is_empty() {
                                view! { <p class="empty">"还没有文档，点击右上角创建一个吧！"</p> }.into_view()
                            } else {
                                let nav = nav_for_docs.clone();
                                view! {
                                <div class="documents-grid">
                                    <For
                                        each=move || documents.get()
                                        key=|doc| doc.id.clone()
                                        children=move |doc: DocumentListItem| {
                                            let doc_id = doc.id.clone();
                                            let owner_name = doc.owner.nickname.clone();
                                            let nav_clone = nav.clone();
                                            view! {
                                                <div class="document-card" on:click=move |_| {
                                                    nav_clone(&format!("/editor/{}", doc_id), Default::default());
                                                }>
                                                    <div class="card-header">
                                                        <h3>{&doc.title}</h3>
                                                        {doc.is_public.then(|| view! {
                                                            <span class="badge badge-public">"公开"</span>
                                                        })}
                                                    </div>
                                                    <p class="card-preview" style="color: #666; font-size: 14px;">
                                                        "作者: "{&owner_name}
                                                    </p>
                                                    <p class="card-meta">
                                                        <span>"创建于: "{doc.created_at.chars().take(10).collect::<String>()}</span>
                                                        <span>"更新于: "{doc.updated_at.chars().take(10).collect::<String>()}</span>
                                                    </p>
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                            }.into_view()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}


// ===== 编辑器页面 =====

#[component]
fn EditorPage() -> impl IntoView {
    let params = use_params_map();
    let doc_id = move || params.with(|p| p.get("id").cloned().unwrap_or_default());

    let (title, set_title) = create_signal(String::new());
    let (content, set_content) = create_signal(String::new());
    let (loading, set_loading) = create_signal(true);
    let (saving, set_saving) = create_signal(false);
    let (error, set_error) = create_signal(None::<String>);
    let (syncing, set_syncing) = create_signal(false); // 同步状态
    let (last_sync_time, set_last_sync_time) = create_signal(None::<f64>); // 上次同步时间

    // CRDT 管理器
    let (crdt_manager, set_crdt_manager) = create_signal(None::<Rc<RefCell<CrdtManager>>>);

    // 协作者管理
    let (collaborators, set_collaborators) = create_signal(Vec::<CollaboratorResponse>::new());
    let (show_collab_panel, set_show_collab_panel) = create_signal(false);
    let (show_add_collab, set_show_add_collab) = create_signal(false);
    let (new_collab_email, set_new_collab_email) = create_signal(String::new());
    let (new_collab_permission, set_new_collab_permission) = create_signal(CollaboratorPermission::Write);

    // WebSocket 和实时协作
    let (ws, set_ws) = create_signal(None::<web_sys::WebSocket>);
    let (online_users, set_online_users) = create_signal(Vec::<(String, String, usize, usize)>::new()); // (user_id, nickname, line, column)
    let (ws_connected, set_ws_connected) = create_signal(false);

    // 当前用户的光标位置（用于发送给其他用户）
    let (cursor_line, set_cursor_line) = create_signal(0usize);
    let (cursor_column, set_cursor_column) = create_signal(0usize);

    // 评论系统
    let (comments, set_comments) = create_signal(Vec::<CommentListItem>::new());
    let (show_comments_panel, set_show_comments_panel) = create_signal(false);
    let (new_comment_content, set_new_comment_content) = create_signal(String::new());

    // 任务系统
    let (tasks, set_tasks) = create_signal(Vec::<TaskListItem>::new());
    let (show_tasks_panel, set_show_tasks_panel) = create_signal(false);
    let (show_create_task, set_show_create_task) = create_signal(false);
    let (new_task_title, set_new_task_title) = create_signal(String::new());
    let (new_task_description, set_new_task_description) = create_signal(String::new());
    let (new_task_priority, set_new_task_priority) = create_signal(TaskPriority::Medium);

    let navigate = use_navigate();
    let navigate_clone = navigate.clone();

    // 加载文档和协作者
    create_effect(move |_| {
        let id = doc_id();
        if let Some(token) = get_token() {
            set_loading.set(true);

            // 加载文档
            let token_clone = token.clone();
            let id_clone = id.clone();
            spawn_local(async move {
                match fetch_document(&token_clone, &id_clone).await {
                    Ok(doc) => {
                        set_title.set(doc.title);
                        set_content.set(doc.content.clone());

                        // 初始化 CRDT 管理器
                        let mut manager = CrdtManager::new();

                        // 如果有 CRDT 状态，从状态初始化；否则从文本内容初始化
                        if let Some(crdt_hex) = doc.crdt_state {
                            if !crdt_hex.is_empty() {
                                leptos::logging::log!("从 CRDT 状态初始化文档");
                                match hex_to_bytes(&crdt_hex) {
                                    Ok(state) => {
                                        if let Err(e) = manager.init_from_state(&state) {
                                            leptos::logging::error!("初始化 CRDT 失败: {}", e);
                                            // 回退到文本初始化
                                            manager.set_text(&doc.content);
                                        } else {
                                            // 从 CRDT 获取文本并更新 UI
                                            let text = manager.get_text();
                                            set_content.set(text);
                                        }
                                    }
                                    Err(e) => {
                                        leptos::logging::error!("解码 CRDT 状态失败: {}", e);
                                        manager.set_text(&doc.content);
                                    }
                                }
                            } else {
                                // 空状态，从文本初始化
                                manager.set_text(&doc.content);
                            }
                        } else {
                            // 没有 CRDT 状态，从文本初始化
                            manager.set_text(&doc.content);
                        }

                        set_crdt_manager.set(Some(Rc::new(RefCell::new(manager))));
                        set_loading.set(false);
                    }
                    Err(e) => {
                        set_error.set(Some(e));
                        set_loading.set(false);
                    }
                }
            });

            // 加载协作者列表
            let token_clone = token.clone();
            let id_clone = id.clone();
            spawn_local(async move {
                match fetch_collaborators(&token_clone, &id_clone).await {
                    Ok(collabs) => set_collaborators.set(collabs),
                    Err(e) => leptos::logging::error!("加载协作者失败: {}", e),
                }
            });

            // 加载评论列表
            let token_clone = token.clone();
            let id_clone = id.clone();
            let set_comments_clone = set_comments.clone();
            spawn_local(async move {
                match fetch_comments(&token_clone, &id_clone).await {
                    Ok(comms) => set_comments_clone.set(comms),
                    Err(e) => leptos::logging::error!("加载评论失败: {}", e),
                }
            });

            // 加载任务列表
            let token_clone = token.clone();
            spawn_local(async move {
                match fetch_tasks(&token_clone, "all").await {
                    Ok(task_list) => set_tasks.set(task_list),
                    Err(e) => leptos::logging::error!("加载任务失败: {}", e),
                }
            });

            // 建立 WebSocket 连接
            let ws_url = format!("{}/ws/documents/{}?token={}", WS_BASE, id, token);
            match web_sys::WebSocket::new(&ws_url) {
                Ok(websocket) => {
                    use wasm_bindgen::prelude::*;
                    use wasm_bindgen::JsCast;

                    // 连接打开
                    let onopen_callback = Closure::wrap(Box::new(move |_| {
                        set_ws_connected.set(true);
                        leptos::logging::log!("WebSocket 已连接");
                    }) as Box<dyn FnMut(JsValue)>);
                    websocket.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
                    onopen_callback.forget();

                    // 接收消息
                    let onmessage_callback = Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
                        if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                            let msg_str: String = txt.into();
                            leptos::logging::log!("收到消息: {}", msg_str);

                            if let Ok(msg) = serde_json::from_str::<WsMessage>(&msg_str) {
                                match msg {
                                    WsMessage::Sync { update } => {
                                        leptos::logging::log!("收到 CRDT 更新: {} 字符", update.len());

                                        // 使用 CRDT 应用更新
                                        if let Some(manager_rc) = crdt_manager.get() {
                                            match hex_to_bytes(&update) {
                                                Ok(update_bytes) => {
                                                    let mut manager = manager_rc.borrow_mut();
                                                    if let Err(e) = manager.apply_update(&update_bytes) {
                                                        leptos::logging::error!("应用 CRDT 更新失败: {}", e);
                                                    } else {
                                                        // 更新成功，同步到 UI
                                                        let text = manager.get_text();
                                                        set_content.set(text);
                                                        set_syncing.set(false);
                                                        set_last_sync_time.set(Some(1.0));
                                                        leptos::logging::log!("CRDT 更新已应用");
                                                    }
                                                }
                                                Err(e) => {
                                                    leptos::logging::error!("解码 CRDT 更新失败: {}", e);
                                                }
                                            }
                                        }
                                    }
                                    WsMessage::UserJoined { user_id, nickname } => {
                                        leptos::logging::log!("用户加入: {} ({})", nickname, user_id);
                                        let mut users = online_users.get();
                                        users.push((user_id.clone(), nickname.clone(), 0, 0));
                                        set_online_users.set(users);
                                    }
                                    WsMessage::UserLeft { user_id } => {
                                        leptos::logging::log!("用户离开: {}", user_id);
                                        let users: Vec<_> = online_users.get()
                                            .into_iter()
                                            .filter(|(id, _, _, _)| id != &user_id)
                                            .collect();
                                        set_online_users.set(users);
                                    }
                                    WsMessage::Awareness { state } => {
                                        // 更新远程用户的光标位置
                                        let mut users = online_users.get();
                                        let mut found = false;
                                        for user in &mut users {
                                            if user.0 == state.user_id {
                                                user.2 = state.cursor.as_ref().map(|c| c.line).unwrap_or(0);
                                                user.3 = state.cursor.as_ref().map(|c| c.column).unwrap_or(0);
                                                found = true;
                                                break;
                                            }
                                        }
                                        if !found {
                                            // 新用户通过 awareness 加入
                                            users.push((
                                                state.user_id.clone(),
                                                state.nickname.unwrap_or_default(),
                                                state.cursor.as_ref().map(|c| c.line).unwrap_or(0),
                                                state.cursor.as_ref().map(|c| c.column).unwrap_or(0),
                                            ));
                                        }
                                        set_online_users.set(users);
                                    }
                                    WsMessage::Error { message } => {
                                        set_error.set(Some(format!("WebSocket 错误: {}", message)));
                                    }
                                }
                            }
                        }
                    }) as Box<dyn FnMut(web_sys::MessageEvent)>);
                    websocket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
                    onmessage_callback.forget();

                    // 连接关闭
                    let onclose_callback = Closure::wrap(Box::new(move |_| {
                        set_ws_connected.set(false);
                        set_online_users.set(Vec::new());
                        leptos::logging::log!("WebSocket 已断开");
                    }) as Box<dyn FnMut(JsValue)>);
                    websocket.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
                    onclose_callback.forget();

                    // 错误处理
                    let onerror_callback = Closure::wrap(Box::new(move |e: web_sys::ErrorEvent| {
                        leptos::logging::error!("WebSocket 错误: {:?}", e);
                        set_ws_connected.set(false);
                    }) as Box<dyn FnMut(web_sys::ErrorEvent)>);
                    websocket.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
                    onerror_callback.forget();

                    set_ws.set(Some(websocket.clone()));
                }
                Err(e) => {
                    leptos::logging::error!("创建 WebSocket 失败: {:?}", e);
                }
            }
        } else {
            navigate_clone("/", Default::default());
        }
    });

    let save_doc = move |_: web_sys::MouseEvent| {
        let id = doc_id();
        if let Some(token) = get_token() {
            set_saving.set(true);
            let title_val = title.get();
            let content_val = content.get();

            spawn_local(async move {
                match update_document_api(&token, &id, title_val, content_val).await {
                    Ok(_) => {
                        set_error.set(Some("保存成功 ✓".to_string()));
                        set_timeout(move || set_error.set(None), std::time::Duration::from_secs(2));
                        set_saving.set(false);
                    }
                    Err(e) => {
                        set_error.set(Some(e));
                        set_saving.set(false);
                    }
                }
            });
        }
    };

    // 添加协作者
    let add_collaborator = move |_: web_sys::MouseEvent| {
        let id = doc_id();
        if let Some(token) = get_token() {
            let email = new_collab_email.get();
            let permission = new_collab_permission.get();

            spawn_local(async move {
                match add_collaborator_api(&token, &id, email.clone(), permission).await {
                    Ok(_) => {
                        set_show_add_collab.set(false);
                        set_new_collab_email.set(String::new());
                        // 重新加载协作者列表
                        if let Ok(collabs) = fetch_collaborators(&token, &id).await {
                            set_collaborators.set(collabs);
                        }
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    };

    // 删除协作者
    let remove_collaborator = move |user_id: String| {
        let id = doc_id();
        if let Some(token) = get_token() {
            spawn_local(async move {
                match remove_collaborator_api(&token, &id, &user_id).await {
                    Ok(_) => {
                        // 从列表中移除
                        let current_collabs = collaborators.get();
                        let updated = current_collabs
                            .into_iter()
                            .filter(|c| c.user_id != user_id)
                            .collect();
                        set_collaborators.set(updated);
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    };

    // ===== 评论操作 =====

    // 创建评论
    let create_comment = move |_: web_sys::MouseEvent| {
        let id = doc_id();
        if let Some(token) = get_token() {
            let content = new_comment_content.get();
            if content.trim().is_empty() {
                return;
            }

            let request = CreateCommentRequest {
                doc_id: id.clone(),
                content,
                parent_id: None,
                position: None,
            };

            let set_comments_clone = set_comments.clone();
            spawn_local(async move {
                match create_comment_api(&token, request).await {
                    Ok(_) => {
                        set_new_comment_content.set(String::new());
                        // 重新加载评论列表
                        if let Ok(comms) = fetch_comments(&token, &id).await {
                            set_comments_clone.set(comms);
                        }
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    };

    // 标记评论为已解决
    let resolve_comment = move |comment_id: String| {
        let id = doc_id();
        if let Some(token) = get_token() {
            let set_comments_clone = set_comments.clone();
            spawn_local(async move {
                match resolve_comment_api(&token, &comment_id).await {
                    Ok(_) => {
                        if let Ok(comms) = fetch_comments(&token, &id).await {
                            set_comments_clone.set(comms);
                        }
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    };

    // 取消评论解决状态
    let unresolve_comment = move |comment_id: String| {
        let id = doc_id();
        if let Some(token) = get_token() {
            let set_comments_clone = set_comments.clone();
            spawn_local(async move {
                match unresolve_comment_api(&token, &comment_id).await {
                    Ok(_) => {
                        if let Ok(comms) = fetch_comments(&token, &id).await {
                            set_comments_clone.set(comms);
                        }
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    };

    // 删除评论
    let delete_comment = move |comment_id: String| {
        let id = doc_id();
        if let Some(token) = get_token() {
            let set_comments_clone = set_comments.clone();
            spawn_local(async move {
                match delete_comment_api(&token, &comment_id).await {
                    Ok(_) => {
                        // 从列表中移除
                        let current_comments = comments.get();
                        let updated = current_comments
                            .into_iter()
                            .filter(|c| c.id != comment_id)
                            .collect();
                        set_comments_clone.set(updated);
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    };

    // ===== 任务操作 =====

    // 创建任务
    let create_task = move |_: web_sys::MouseEvent| {
        let id = doc_id();
        if let Some(token) = get_token() {
            let title = new_task_title.get();
            if title.trim().is_empty() {
                return;
            }

            let request = CreateTaskRequest {
                title,
                description: {
                    let desc = new_task_description.get();
                    if desc.trim().is_empty() { None } else { Some(desc) }
                },
                doc_id: Some(id.clone()),
                assignee_id: None,
                priority: new_task_priority.get(),
                due_date: None,
            };

            let set_tasks_clone = set_tasks.clone();
            spawn_local(async move {
                match create_task_api(&token, request).await {
                    Ok(_) => {
                        set_show_create_task.set(false);
                        set_new_task_title.set(String::new());
                        set_new_task_description.set(String::new());
                        // 重新加载任务列表
                        if let Ok(task_list) = fetch_tasks(&token, "all").await {
                            set_tasks_clone.set(task_list);
                        }
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    };

    // 更新任务状态
    let update_task_status = move |task_id: String, status: TaskStatus| {
        if let Some(token) = get_token() {
            let set_tasks_clone = set_tasks.clone();
            spawn_local(async move {
                match update_task_status_api(&token, &task_id, status).await {
                    Ok(_) => {
                        if let Ok(task_list) = fetch_tasks(&token, "all").await {
                            set_tasks_clone.set(task_list);
                        }
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    };

    // 删除任务
    let delete_task_fn = move |task_id: String| {
        if let Some(token) = get_token() {
            let set_tasks_clone = set_tasks.clone();
            spawn_local(async move {
                match delete_task_api(&token, &task_id).await {
                    Ok(_) => {
                        // 从列表中移除
                        let current_tasks = tasks.get();
                        let updated = current_tasks
                            .into_iter()
                            .filter(|t| t.id != task_id)
                            .collect();
                        set_tasks_clone.set(updated);
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    };

    // 通过 WebSocket 发送 CRDT 更新
    let sync_content = move |new_content: String| {
        if let Some(websocket) = ws.get() {
            if let Some(manager_rc) = crdt_manager.get() {
                set_syncing.set(true);

                let mut manager = manager_rc.borrow_mut();

                // 更新 CRDT 文档
                manager.set_text(&new_content);

                // 获取完整状态（简化版，实际应该是增量更新）
                let state = manager.get_state();
                let hex_update = bytes_to_hex(&state);

                // 发送 CRDT 更新
                let msg = WsMessage::Sync {
                    update: hex_update,
                };

                if let Ok(msg_json) = serde_json::to_string(&msg) {
                    if websocket.send_with_str(&msg_json).is_ok() {
                        leptos::logging::log!("CRDT 更新已发送");
                    } else {
                        leptos::logging::error!("发送 CRDT 更新失败");
                        set_syncing.set(false);
                    }
                }
            }
        }
    };

    // 发送 Awareness 状态（光标位置）
    let send_awareness = move |line: usize, column: usize| {
        if let Some(websocket) = ws.get() {
            let awareness_state = AwarenessState {
                user_id: "".to_string(), // 后端会从 token 中提取
                nickname: None,
                cursor: Some(CursorPosition { line, column }),
            };

            let msg = WsMessage::Awareness {
                state: awareness_state,
            };

            if let Ok(msg_json) = serde_json::to_string(&msg) {
                let _ = websocket.send_with_str(&msg_json);
            }
        }
    };

    // 监听内容变化，防抖后同步
    let debounce_timer = create_signal(None::<i32>);
    let on_content_change = move |ev| {
        let new_content = event_target_value(&ev);
        set_content.set(new_content.clone());

        // 清除之前的定时器
        if let Some(timer_id) = debounce_timer.0.get() {
            web_sys::window()
                .unwrap()
                .clear_timeout_with_handle(timer_id);
        }

        // 设置新的防抖定时器（500ms）- 同时触发自动保存
        let id = doc_id();
        let title_val = title.get();
        let sync_fn = sync_content.clone();
        let set_error_clone = set_error.clone();
        let callback = Closure::wrap(Box::new(move || {
            // 同步 CRDT
            sync_fn(new_content.clone());

            // 自动保存到后端
            if let Some(token) = get_token() {
                let title = title_val.clone();
                let content = new_content.clone();
                let id_clone = id.clone();
                let set_error = set_error_clone.clone();
                spawn_local(async move {
                    match update_document_api(&token, &id_clone, title, content).await {
                        Ok(_) => {
                            leptos::logging::log!("自动保存成功");
                        }
                        Err(e) => {
                            leptos::logging::error!("自动保存失败: {}", e);
                        }
                    }
                });
            }
        }) as Box<dyn Fn()>);

        if let Some(window) = web_sys::window() {
            let timer_id = window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    callback.as_ref().unchecked_ref(),
                    500,
                )
                .unwrap();
            debounce_timer.1.set(Some(timer_id));
            callback.forget();
        }
    };

    // ===== 导出功能 =====

    // 导出为 Markdown - 使用简单的 data URI 方法
    let export_markdown = move |_: web_sys::MouseEvent| {
        let title_val = title.get();
        let content_val = content.get();

        // 创建 Markdown 内容
        let markdown = format!("# {}\n\n{}", title_val, content_val);

        // 使用 data URI 下载
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                use wasm_bindgen::JsCast;
                if let Ok(anchor) = document.create_element("a") {
                    // URL encode the content
                    let encoded = urlencoding::encode(&markdown);
                    let data_url = format!("data:text/markdown;charset=utf-8,{}", encoded);

                    anchor.set_attribute("href", &data_url).unwrap();
                    anchor.set_attribute("download", &format!("{}.md", title_val)).unwrap();
                    if let Some(html_anchor) = anchor.dyn_into::<web_sys::HtmlAnchorElement>().ok() {
                        html_anchor.click();
                    }
                }
            }
        }
    };

    // 导出为 PDF (通过打印)
    let export_pdf = move |_: web_sys::MouseEvent| {
        let title_val = title.get();
        let content_val = content.get();

        // 创建一个新的窗口用于打印
        if let Some(window) = web_sys::window() {
            if let Ok(Some(print_window)) = window.open_with_url_and_target("", "_blank") {
                // 使用 innerHTML 或 outerHTML 直接设置内容
                let html = format!(
                    r#"<!DOCTYPE html>
                    <html>
                    <head>
                        <title>{}</title>
                        <style>
                            body {{ font-family: sans-serif; max-width: 800px; margin: 40px auto; padding: 20px; }}
                            h1 {{ border-bottom: 2px solid #333; }}
                        </style>
                    </head>
                    <body>
                        <h1>{}</h1>
                        <div style="white-space: pre-wrap;">{}</div>
                    </body>
                    </html>"#,
                    title_val, title_val, content_val
                );

                if let Some(print_doc) = print_window.document() {
                    use wasm_bindgen::JsCast;
                    // 直接设置 body 的 innerHTML
                    if let Some(body) = print_doc.body() {
                        let _ = body.set_outer_html(&html);
                    }

                    let callback = wasm_bindgen::closure::Closure::once(Box::new(move || {
                        let _ = print_window.print();
                    }) as Box<dyn Fn()>);
                    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                        callback.as_ref().unchecked_ref(),
                        500,
                    );
                    callback.forget();
                }
            }
        }
    };

    view! {
        <div class="editor-page">
            <div class="editor-toolbar" style="display: flex; justify-content: space-between; align-items: center; padding: 8px 16px;">
                <div style="display: flex; align-items: center; gap: 12px;">
                    <button class="btn-back" on:click=move |_| navigate("/documents", Default::default())>
                        "← 返回"
                    </button>

                    // 在线用户显示
                    <div style="display: flex; align-items: center; gap: 4px; padding: 4px 8px; background: #F5F6F7; border-radius: 16px;">
                        {move || if online_users.get().is_empty() {
                            view! {
                                <span style="font-size: 12px; color: #86909C;">
                                    {move || if ws_connected.get() { "● 仅我" } else { "○ 离线" }}
                                </span>
                            }.into_view()
                        } else {
                            view! {
                                <>
                                    <span style="font-size: 12px; color: #10b981;">"●"</span>
                                    <For
                                        each=move || online_users.get()
                                        key=|(id, _, _, _)| id.clone()
                                        children=|(_, nickname, _, _): (String, String, usize, usize)| {
                                            view! {
                                                <span
                                                    style="font-size: 12px; color: #4E5969; padding: 2px 6px; background: white; border-radius: 10px; margin-left: 2px;"
                                                    title={nickname.clone()}
                                                >
                                                    {&nickname}
                                                </span>
                                            }
                                        }
                                    />
                                </>
                            }.into_view()
                        }}
                    </div>

                    // 导出按钮
                    <div style="display: flex; gap: 4px;">
                        <button
                            class="btn btn-secondary"
                            style="padding: 6px 10px; font-size: 12px;"
                            on:click=export_markdown
                            title="导出为 Markdown"
                        >
                            "📄 MD"
                        </button>
                        <button
                            class="btn btn-secondary"
                            style="padding: 6px 10px; font-size: 12px;"
                            on:click=export_pdf
                            title="导出为 PDF"
                        >
                            "📕 PDF"
                        </button>
                    </div>
                </div>

                // 右侧按钮区域
                <div style="display: flex; gap: 8px; align-items: center;">
                    <button
                        class="btn btn-secondary"
                        style="padding: 6px 10px; font-size: 13px;"
                        on:click=move |_| set_show_collab_panel.set(!show_collab_panel.get())
                    >
                        "👥 "{move || collaborators.get().len()}
                    </button>
                    <button
                        class="btn btn-secondary"
                        style="padding: 6px 10px; font-size: 13px;"
                        on:click=move |_| set_show_comments_panel.set(!show_comments_panel.get())
                    >
                        "💬 "{move || comments.get().len()}
                    </button>
                    <button
                        class="btn btn-secondary"
                        style="padding: 6px 10px; font-size: 13px;"
                        on:click=move |_| set_show_tasks_panel.set(!show_tasks_panel.get())
                    >
                        "📋 "{move || tasks.get().len()}
                    </button>

                    // 自动保存状态指示器
                    <div style="display: flex; align-items: center; gap: 4px; font-size: 12px; color: #86909C;">
                        {move || if syncing.get() {
                            view! { <span style="color: #3b82f6;">"⟳ 保存中..."</span> }.into_view()
                        } else if last_sync_time.get().is_some() {
                            view! { <span style="color: #10b981;">"✓ 已保存"</span> }.into_view()
                        } else {
                            view! { <span>"等待编辑..."</span> }.into_view()
                        }}
                    </div>
                </div>
            </div>

            <div class="editor-layout">
                {move || if loading.get() {
                    view! { <div class="loading">"加载中..."</div> }.into_view()
                } else {
                    view! {
                        <div class="editor-content">
                            <input
                                class="title-input"
                                type="text"
                                placeholder="文档标题"
                                prop:value=move || title.get()
                                on:input=move |ev| set_title.set(event_target_value(&ev))
                            />
                            <textarea
                                class="content-textarea"
                                id="editor-textarea"
                                placeholder="开始输入..."
                                prop:value=move || content.get()
                                on:input=on_content_change
                                on:keyup=move |ev| {
                                    // 计算光标位置（简化版，基于换行符计数）
                                    use wasm_bindgen::JsCast;
                                    if let Some(textarea) = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlTextAreaElement>().ok()) {
                                        let text = textarea.value();
                                        // 简化：使用文本长度作为光标位置的近似值
                                        let position = text.chars().count();
                                        let line = text.chars().filter(|&c| c == '\n').count() + 1;
                                        set_cursor_line.set(line);
                                        set_cursor_column.set(position);

                                        // 发送 awareness 状态
                                        send_awareness(line, position);
                                    }
                                }
                                on:click=move |ev| {
                                    // 点击时也更新光标位置
                                    use wasm_bindgen::JsCast;
                                    if let Some(textarea) = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlTextAreaElement>().ok()) {
                                        let text = textarea.value();
                                        let position = text.chars().count();
                                        let line = text.chars().filter(|&c| c == '\n').count() + 1;
                                        set_cursor_line.set(line);
                                        set_cursor_column.set(position);
                                        send_awareness(line, position);
                                    }
                                }
                            />

                            {move || error.get().map(|e| view! {
                                <div class="message">{e}</div>
                            })}
                        </div>
                    }.into_view()
                }}

                // 协作者侧边栏
                {move || show_collab_panel.get().then(|| view! {
                    <div class="collaborators-panel">
                        <div class="panel-header">
                            <h3>"协作者"</h3>
                            <button
                                class="btn-close"
                                on:click=move |_| set_show_collab_panel.set(false)
                            >
                                "×"
                            </button>
                        </div>

                        <div class="panel-content">
                            // 在线用户显示
                            <div class="online-users" style="margin-bottom: 16px; padding: 12px; background: #f5f5f5; border-radius: 4px;">
                                <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 8px;">
                                    <strong>"在线用户"</strong>
                                    <span style=move || if ws_connected.get() {
                                        "color: #10b981;"
                                    } else {
                                        "color: #ef4444;"
                                    }>
                                        {move || if ws_connected.get() { "● 已连接" } else { "○ 未连接" }}
                                    </span>
                                </div>
                                <div>
                                    {move || if online_users.get().is_empty() {
                                        view! { <div style="color: #999; font-size: 14px;">"暂无在线用户"</div> }.into_view()
                                    } else {
                                        view! {
                                            <For
                                                each=move || online_users.get()
                                                key=|(id, _, _, _)| id.clone()
                                                children=|(id, nickname, line, column): (String, String, usize, usize)| {
                                                    view! {
                                                        <div style="font-size: 14px; padding: 4px 0; color: #10b981;">
                                                            "● "{nickname}
                                                            <span style="color: #999; font-size: 12px;">
                                                                "(行: "{line}", 列: "{column}")"
                                                            </span>
                                                        </div>
                                                    }
                                                }
                                            />
                                        }.into_view()
                                    }}
                                </div>
                            </div>

                            <button
                                class="btn btn-primary"
                                style="width: 100%; margin-bottom: 16px;"
                                on:click=move |_| set_show_add_collab.set(!show_add_collab.get())
                            >
                                "+ 添加协作者"
                            </button>

                            {move || show_add_collab.get().then(|| view! {
                                <div class="add-collab-form">
                                    <input
                                        type="text"
                                        placeholder="用户邮箱"
                                        prop:value=move || new_collab_email.get()
                                        on:input=move |ev| set_new_collab_email.set(event_target_value(&ev))
                                        class="input-sm"
                                    />
                                    <select
                                        class="select-sm"
                                        on:change=move |ev| {
                                            let value = event_target_value(&ev);
                                            let perm = match value.as_str() {
                                                "read" => CollaboratorPermission::Read,
                                                "write" => CollaboratorPermission::Write,
                                                "admin" => CollaboratorPermission::Admin,
                                                _ => CollaboratorPermission::Write,
                                            };
                                            set_new_collab_permission.set(perm);
                                        }
                                    >
                                        <option value="read">"只读"</option>
                                        <option value="write" selected>"编辑"</option>
                                        <option value="admin">"管理"</option>
                                    </select>
                                    <div style="display: flex; gap: 8px; margin-top: 8px;">
                                        <button class="btn-sm" on:click=add_collaborator>"添加"</button>
                                        <button
                                            class="btn-sm btn-cancel"
                                            on:click=move |_| set_show_add_collab.set(false)
                                        >
                                            "取消"
                                        </button>
                                    </div>
                                </div>
                            })}

                            <div class="collaborators-list">
                                <For
                                    each=move || collaborators.get()
                                    key=|collab| collab.user_id.clone()
                                    children=move |collab: CollaboratorResponse| {
                                        let user_id = collab.user_id.clone();
                                        view! {
                                            <div class="collaborator-item">
                                                <div class="collab-info">
                                                    <div class="collab-name">{&collab.nickname}</div>
                                                    <div class="collab-email">{&collab.email}</div>
                                                </div>
                                                <div class="collab-actions">
                                                    <span class="collab-permission">{format!("{}", collab.permission)}</span>
                                                    <button
                                                        class="btn-icon btn-danger"
                                                        on:click=move |_| remove_collaborator(user_id.clone())
                                                        title="移除"
                                                    >
                                                        "×"
                                                    </button>
                                                </div>
                                            </div>
                                        }
                                    }
                                />
                            </div>
                        </div>
                    </div>
                })}

                // 评论侧边栏
                {move || show_comments_panel.get().then(|| view! {
                    <div class="collaborators-panel">
                        <div class="panel-header">
                            <h3>"评论"</h3>
                            <button
                                class="btn-close"
                                on:click=move |_| set_show_comments_panel.set(false)
                            >
                                "×"
                            </button>
                        </div>

                        <div class="panel-content">
                            // 创建评论表单
                            <div style="margin-bottom: 16px;">
                                <textarea
                                    placeholder="写下你的评论..."
                                    prop:value=move || new_comment_content.get()
                                    on:input=move |ev| set_new_comment_content.set(event_target_value(&ev))
                                    style="width: 100%; min-height: 80px; padding: 8px; border: 1px solid #ddd; border-radius: 4px; resize: vertical;"
                                />
                                <button
                                    class="btn btn-primary"
                                    style="width: 100%; margin-top: 8px;"
                                    on:click=create_comment
                                >
                                    "发布评论"
                                </button>
                            </div>

                            // 评论列表
                            <div class="comments-list" style="max-height: 400px; overflow-y: auto;">
                                {move || if comments.get().is_empty() {
                                    view! { <div style="color: #999; text-align: center; padding: 20px;">"暂无评论"</div> }.into_view()
                                } else {
                                    view! {
                                        <For
                                            each=move || comments.get()
                                            key=|comment| comment.id.clone()
                                            children=move |comment: CommentListItem| {
                                                let comment_id = comment.id.clone();
                                                let comment_id_for_resolve = comment_id.clone();
                                                let comment_id_for_delete = comment_id.clone();
                                                let is_resolved = comment.is_resolved;
                                                view! {
                                                    <div class="comment-item" style=format!("padding: 12px; border-bottom: 1px solid #eee; {}", if is_resolved { "opacity: 0.6;" } else { "" })>
                                                        <div style="display: flex; justify-content: space-between; align-items: start; margin-bottom: 8px;">
                                                            <div style="display: flex; align-items: center; gap: 8px;">
                                                                <strong style="font-size: 14px;">{&comment.user.nickname}</strong>
                                                                {is_resolved.then(|| view! {
                                                                    <span class="badge" style="background: #10b981; color: white; padding: 2px 6px; border-radius: 4px; font-size: 11px;">"已解决"</span>
                                                                })}
                                                            </div>
                                                            <span style="font-size: 12px; color: #999;">
                                                                {comment.created_at.chars().take(16).collect::<String>()}
                                                            </span>
                                                        </div>
                                                        <div style="font-size: 14px; line-height: 1.5; margin-bottom: 8px;">
                                                            {&comment.content}
                                                        </div>
                                                        <div style="display: flex; gap: 8px;">
                                                            {if is_resolved {
                                                                view! {
                                                                    <button
                                                                        class="btn-sm btn-secondary"
                                                                        on:click=move |_| unresolve_comment(comment_id_for_resolve.clone())
                                                                    >
                                                                        "重开"
                                                                    </button>
                                                                }.into_view()
                                                            } else {
                                                                view! {
                                                                    <button
                                                                        class="btn-sm btn-secondary"
                                                                        on:click=move |_| resolve_comment(comment_id_for_resolve.clone())
                                                                    >
                                                                        "解决"
                                                                    </button>
                                                                }.into_view()
                                                            }};
                                                            <button
                                                                class="btn-sm btn-danger"
                                                                on:click=move |_| delete_comment(comment_id_for_delete.clone())
                                                            >
                                                                "删除"
                                                            </button>
                                                        </div>
                                                    </div>
                                                }
                                            }
                                        />
                                    }.into_view()
                                }}
                            </div>
                        </div>
                    </div>
                })}

                // 任务侧边栏
                {move || show_tasks_panel.get().then(|| view! {
                    <div class="collaborators-panel">
                        <div class="panel-header">
                            <h3>"任务"</h3>
                            <button
                                class="btn-close"
                                on:click=move |_| set_show_tasks_panel.set(false)
                            >
                                "×"
                            </button>
                        </div>

                        <div class="panel-content">
                            // 创建任务按钮
                            <button
                                class="btn btn-primary"
                                style="width: 100%; margin-bottom: 16px;"
                                on:click=move |_| set_show_create_task.set(!show_create_task.get())
                            >
                                "+ 新建任务"
                            </button>

                            {move || show_create_task.get().then(|| view! {
                                <div style="background: #f5f5f5; padding: 12px; border-radius: 4px; margin-bottom: 16px;">
                                    <input
                                        type="text"
                                        placeholder="任务标题"
                                        prop:value=move || new_task_title.get()
                                        on:input=move |ev| set_new_task_title.set(event_target_value(&ev))
                                        style="width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px; margin-bottom: 8px;"
                                    />
                                    <textarea
                                        placeholder="任务描述（可选）"
                                        prop:value=move || new_task_description.get()
                                        on:input=move |ev| set_new_task_description.set(event_target_value(&ev))
                                        style="width: 100%; min-height: 60px; padding: 8px; border: 1px solid #ddd; border-radius: 4px; margin-bottom: 8px; resize: vertical;"
                                    />
                                    <div style="display: flex; gap: 8px; margin-bottom: 8px;">
                                        <select
                                            style="flex: 1; padding: 8px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:change=move |ev| {
                                                let value = event_target_value(&ev);
                                                let priority = match value.as_str() {
                                                    "low" => TaskPriority::Low,
                                                    "medium" => TaskPriority::Medium,
                                                    "high" => TaskPriority::High,
                                                    "urgent" => TaskPriority::Urgent,
                                                    _ => TaskPriority::Medium,
                                                };
                                                set_new_task_priority.set(priority);
                                            }
                                        >
                                            <option value="low">"低优先级"</option>
                                            <option value="medium" selected>"中优先级"</option>
                                            <option value="high">"高优先级"</option>
                                            <option value="urgent">"紧急"</option>
                                        </select>
                                    </div>
                                    <div style="display: flex; gap: 8px;">
                                        <button
                                            class="btn-sm"
                                            style="flex: 1;"
                                            on:click=create_task
                                        >
                                            "创建"
                                        </button>
                                        <button
                                            class="btn-sm btn-cancel"
                                            style="flex: 1;"
                                            on:click=move |_| set_show_create_task.set(false)
                                        >
                                            "取消"
                                        </button>
                                    </div>
                                </div>
                            })}

                            // 任务列表
                            <div class="tasks-list" style="max-height: 400px; overflow-y: auto;">
                                {move || if tasks.get().is_empty() {
                                    view! { <div style="color: #999; text-align: center; padding: 20px;">"暂无任务"</div> }.into_view()
                                } else {
                                    view! {
                                        <For
                                            each=move || tasks.get()
                                            key=|task| task.id.clone()
                                            children=move |task: TaskListItem| {
                                                let task_id = task.id.clone();
                                                let task_id_for_delete = task_id.clone();
                                                let status_str = task.status.clone();
                                                let status_str_for_select1 = status_str.clone();
                                                let status_str_for_select2 = status_str.clone();
                                                let status_str_for_select3 = status_str.clone();
                                                let status_str_for_select4 = status_str.clone();
                                                let priority_str = task.priority.clone();
                                                let priority_display = match priority_str.as_str() {
                                                    "low" => TaskPriority::Low,
                                                    "medium" => TaskPriority::Medium,
                                                    "high" => TaskPriority::High,
                                                    "urgent" => TaskPriority::Urgent,
                                                    _ => TaskPriority::Medium,
                                                };
                                                view! {
                                                    <div class="task-item" style=format!("padding: 12px; border-bottom: 1px solid #eee; border-left: 3px solid {};", priority_display.color())>
                                                        <div style="display: flex; justify-content: space-between; align-items: start; margin-bottom: 4px;">
                                                            <strong style="font-size: 14px;">{&task.title}</strong>
                                                            <span class="badge" style=format!("background: {}; color: white; padding: 2px 6px; border-radius: 4px; font-size: 11px;", priority_display.color())>
                                                                {priority_display.display()}
                                                            </span>
                                                        </div>
                                                        {task.doc_title.as_ref().map(|doc_title| view! {
                                                            <div style="font-size: 12px; color: #666; margin-bottom: 4px;">"📄 "{doc_title}</div>
                                                        })}
                                                        <div style="display: flex; justify-content: space-between; align-items: center;">
                                                            <select
                                                                class="select-sm"
                                                                style="padding: 4px 8px; font-size: 12px; border-radius: 4px; border: 1px solid #ddd;"
                                                                on:change=move |ev| {
                                                                    let value = event_target_value(&ev);
                                                                    let status = match value.as_str() {
                                                                        "pending" => TaskStatus::Pending,
                                                                        "in_progress" => TaskStatus::InProgress,
                                                                        "completed" => TaskStatus::Completed,
                                                                        "cancelled" => TaskStatus::Cancelled,
                                                                        _ => TaskStatus::Pending,
                                                                    };
                                                                    update_task_status(task_id.clone(), status);
                                                                }
                                                            >
                                                                <option value="pending" selected=move || status_str_for_select1 == "pending">"待处理"</option>
                                                                <option value="in_progress" selected=move || status_str_for_select2 == "in_progress">"进行中"</option>
                                                                <option value="completed" selected=move || status_str_for_select3 == "completed">"已完成"</option>
                                                                <option value="cancelled" selected=move || status_str_for_select4 == "cancelled">"已取消"</option>
                                                            </select>
                                                            <button
                                                                class="btn-icon btn-danger"
                                                                on:click=move |_| delete_task_fn(task_id_for_delete.clone())
                                                                title="删除"
                                                                style="padding: 4px 8px;"
                                                            >
                                                                "×"
                                                            </button>
                                                        </div>
                                                    </div>
                                                }
                                            }
                                        />
                                    }.into_view()
                                }}
                            </div>
                        </div>
                    </div>
                })}
            </div>
        </div>
    }
}

// ===== 主应用 =====

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Router base="/">
            <Routes>
                <Route path="/" view=LoginPage/>
                <Route path="/register" view=RegisterPage/>
                <Route path="/documents" view=DocumentsPage/>
                <Route path="/editor/:id" view=EditorPage/>
            </Routes>
        </Router>
    }
}
