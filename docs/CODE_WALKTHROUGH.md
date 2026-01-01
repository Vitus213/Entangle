# 核心代码导读（带详细注释）

## 文件1：WebSocket 处理器完整注释

**文件路径：`crates/api/src/ws/handlers.rs`**

这是整个实时协作系统的心脏，理解这个文件就理解了 80% 的核心逻辑。

```rust
use crate::middleware::AuthUser;
use crate::ws::{WsHub, WsMessage};
use axum::{
    extract::{ws::Message, ws::WebSocket, ws::WebSocketUpgrade, Path},
    response::Response,
    Extension,
};
use entangle_collab::BroadcastMessage;
use entangle_core::{AppError, DocumentPermissionService};
use entangle_db::DocumentRepository;
use futures::{sink::SinkExt, stream::StreamExt};
use sqlx::PgPool;
use std::time::Duration;
use tokio::time::{interval, timeout};
use uuid::Uuid;

// ============================================================
// 第一部分：WebSocket 升级处理
// ============================================================

/// WebSocket 连接的入口函数
///
/// 当客户端请求 ws://server/ws/documents/{doc_id}?token=xxx 时，
/// Axum 会调用这个函数处理升级请求。
///
/// **流程：**
/// 1. 验证文档是否存在
/// 2. 验证用户是否有读权限
/// 3. 升级为 WebSocket 连接
///
/// **参数说明：**
/// - `ws: WebSocketUpgrade` - WebSocket 升级请求（Axum 自动注入）
/// - `Path(doc_id): Path<Uuid>` - URL 路径参数，从 /ws/documents/{doc_id} 提取
/// - `user: AuthUser` - 认证用户信息（中间件自动注入）
/// - `Extension(hub): Extension<WsHub>` - WebSocket Hub（共享状态）
/// - `Extension(pool): Extension<PgPool>` - 数据库连接池（共享状态）
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path(doc_id): Path<Uuid>,
    user: AuthUser,
    Extension(hub): Extension<WsHub>,
    Extension(pool): Extension<PgPool>,
) -> Result<Response, AppError> {
    // 步骤1：查询文档是否存在
    // 使用 Repository 模式访问数据库
    let doc = DocumentRepository::find_by_id(&pool, doc_id)
        .await
        .map_err(|e| AppError::Internal(format!("数据库错误: {}", e)))?
        .ok_or_else(|| AppError::NotFound("文档不存在".to_string()))?;

    // 步骤2：检查用户权限
    // 调用权限服务检查用户是否可以读取该文档
    // 权限检查逻辑：所有者 || 公开文档 || 协作者
    if !DocumentPermissionService::can_read(&pool, user.user_id, doc_id)
        .await
        .map_err(|e| AppError::Internal(format!("权限检查失败: {}", e)))?
    {
        return Err(AppError::Forbidden("无权访问该文档".to_string()));
    }

    // 步骤3：记录连接日志
    tracing::info!(
        "User {} connecting to document {} (owner: {})",
        user.user_id,
        doc_id,
        doc.owner_id
    );

    // 步骤4：升级为 WebSocket 连接
    // ws.on_upgrade() 返回一个 Future，当 WebSocket 握手完成后执行
    // 我们传入一个闭包，在闭包中调用 handle_socket 处理实际的 WebSocket 通信
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, doc_id, user.user_id, hub)))
}

// ============================================================
// 第二部分：WebSocket 连接生命周期管理
// ============================================================

/// 处理单个 WebSocket 连接的整个生命周期
///
/// 这是一个长时间运行的异步函数，会一直执行直到连接关闭。
///
/// **主要职责：**
/// 1. 加载文档状态
/// 2. 用户加入房间
/// 3. 发送初始状态和在线用户
/// 4. 循环处理消息（客户端消息、广播消息、心跳）
/// 5. 清理和保存
async fn handle_socket(socket: WebSocket, doc_id: Uuid, user_id: Uuid, hub: WsHub) {
    // ---------------------------------------------
    // 阶段1：初始化
    // ---------------------------------------------

    // split() 将 WebSocket 分离为发送端和接收端
    // 这样我们可以同时读写（Rust 的所有权系统要求这样做）
    let (mut sender, mut receiver) = socket.split();

    // 尝试从数据库加载文档的 CRDT 状态
    // 如果加载失败（例如新文档），会使用空状态初始化
    if let Err(e) = hub.load_document(doc_id).await {
        tracing::debug!("Could not load document from DB (may be new): {}", e);
    }

    // 获取或创建文档房间
    // 房间是一个内存中的数据结构，管理：
    // - CRDT 文档状态
    // - 在线用户列表
    // - 广播通道
    let room = hub.room_manager().get_or_create_room(doc_id);

    // 订阅房间的广播通道
    // 当其他用户发送消息时，这个 receiver 会收到通知
    let mut broadcast_rx = room.subscribe();

    // 将用户添加到房间
    // 这会：
    // 1. 在 connections HashMap 中记录用户
    // 2. 向其他用户广播 UserJoined 消息
    room.user_join(user_id);
    tracing::info!("User {} joined document {}", user_id, doc_id);

    // ---------------------------------------------
    // 阶段2：发送初始状态
    // ---------------------------------------------

    // 获取当前文档的文本内容
    let text_content = room.get_text_content();

    // 构造 Sync 消息并发送给新用户
    // 这样用户就能看到文档的当前内容
    if let Ok(msg) = serde_json::to_string(&WsMessage::Sync { update: text_content }) {
        if sender.send(Message::Text(msg)).await.is_err() {
            // 如果发送失败（例如连接已关闭），立即清理并返回
            tracing::error!("Failed to send initial state to user {}", user_id);
            room.user_leave(&user_id);
            hub.room_manager().remove_room_if_empty(&doc_id);
            return;
        }
    }

    // 发送其他在线用户的感知状态（cursor position, selection 等）
    let awareness_states = room.get_all_awareness();
    for (uid, state) in awareness_states {
        if uid != user_id {  // 跳过自己
            if let Ok(msg) = serde_json::to_string(&WsMessage::Awareness { state }) {
                let _ = sender.send(Message::Text(msg)).await;
            }
        }
    }

    // ---------------------------------------------
    // 阶段3：创建心跳定时器
    // ---------------------------------------------

    // 创建一个定时器，每 30 秒触发一次
    // 用于发送 Ping 消息保持连接活跃
    let mut heartbeat_interval = interval(Duration::from_secs(30));

    // 克隆 hub 用于在异步块中使用
    let hub_clone = hub.clone();

    // ---------------------------------------------
    // 阶段4：主循环 - 并发处理多个事件
    // ---------------------------------------------

    // tokio::select! 宏允许我们同时等待多个异步操作
    // 类似于 JavaScript 的 Promise.race()
    // 哪个操作先完成就先处理哪个
    loop {
        tokio::select! {
            // ==========================================
            // 分支1：处理来自客户端的消息
            // ==========================================
            msg_result = receiver.next() => {
                match msg_result {
                    // 收到文本消息（JSON 格式）
                    Some(Ok(Message::Text(text))) => {
                        // 尝试解析为 WsMessage 枚举
                        if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                            // 调用处理函数
                            // 返回值表示是否修改了文档（需要保存）
                            if handle_client_message(&room, user_id, ws_msg).await {
                                // 标记文档为"脏"（dirty），需要保存到数据库
                                hub_clone.mark_dirty(doc_id).await;
                            }
                        }
                    }

                    // 收到 Ping 消息，回复 Pong
                    Some(Ok(Message::Ping(data))) => {
                        if sender.send(Message::Pong(data)).await.is_err() {
                            break;  // 发送失败，退出循环
                        }
                    }

                    // 收到 Pong 消息（心跳响应）
                    Some(Ok(Message::Pong(_))) => {
                        tracing::trace!("Received pong from user {}", user_id);
                    }

                    // 客户端主动关闭连接
                    Some(Ok(Message::Close(_))) => {
                        tracing::info!("User {} closed connection", user_id);
                        break;  // 退出循环
                    }

                    // WebSocket 错误
                    Some(Err(e)) => {
                        tracing::warn!("WebSocket error for user {}: {}", user_id, e);
                        break;
                    }

                    // 连接流结束
                    None => {
                        tracing::info!("WebSocket stream ended for user {}", user_id);
                        break;
                    }

                    // 其他消息类型（Binary 等）
                    _ => {}
                }
            }

            // ==========================================
            // 分支2：接收广播消息并转发给客户端
            // ==========================================
            broadcast_result = broadcast_rx.recv() => {
                match broadcast_result {
                    Ok(msg) => {
                        // 判断是否应该转发这条消息
                        // 例如：不转发自己发送的消息
                        if should_forward(&msg, user_id) {
                            // 将 BroadcastMessage 转换为 WsMessage
                            if let Some(ws_msg) = broadcast_to_ws_message(msg) {
                                if let Ok(json) = serde_json::to_string(&ws_msg) {
                                    // 通过 WebSocket 发送给客户端
                                    if sender.send(Message::Text(json)).await.is_err() {
                                        break;  // 发送失败，退出循环
                                    }
                                }
                            }
                        }
                    }

                    // 广播通道落后（消息太多来不及处理）
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("User {} lagged {} messages", user_id, n);
                        // 继续运行，不退出
                    }

                    // 广播通道关闭
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        tracing::info!("Broadcast channel closed for user {}", user_id);
                        break;
                    }
                }
            }

            // ==========================================
            // 分支3：定期发送心跳 Ping
            // ==========================================
            _ = heartbeat_interval.tick() => {
                // 发送 Ping，设置 10 秒超时
                let ping_result = timeout(
                    Duration::from_secs(10),
                    sender.send(Message::Ping(vec![]))
                ).await;

                match ping_result {
                    Ok(Ok(_)) => {
                        tracing::trace!("Sent ping to user {}", user_id);
                    }
                    Ok(Err(e)) => {
                        tracing::warn!("Failed to send ping to user {}: {}", user_id, e);
                        break;  // 发送失败，断开连接
                    }
                    Err(_) => {
                        tracing::warn!("Ping timeout for user {}", user_id);
                        break;  // 超时，断开连接
                    }
                }
            }
        }
    }

    // ---------------------------------------------
    // 阶段5：清理和保存
    // ---------------------------------------------

    // 从房间中移除用户
    room.user_leave(&user_id);

    // 如果房间空了（没有其他用户），保存文档状态到数据库
    let user_count = room.get_user_count();
    if user_count == 0 {
        if let Err(e) = hub.save_document(doc_id).await {
            tracing::warn!("Failed to save document {} on room close: {}", doc_id, e);
        }
    }

    // 如果房间空了，从内存中移除房间
    hub.room_manager().remove_room_if_empty(&doc_id);
    tracing::info!("User {} left document {}", user_id, doc_id);
}

// ============================================================
// 第三部分：消息处理逻辑
// ============================================================

/// 处理来自客户端的消息
///
/// **返回值：**
/// - `true`：文档被修改，需要保存
/// - `false`：文档未修改（如 Awareness 更新）
async fn handle_client_message(
    room: &entangle_collab::DocumentRoom,
    user_id: Uuid,
    msg: WsMessage,
) -> bool {
    match msg {
        // ---------------------------------------------
        // Sync 消息：文档内容更新
        // ---------------------------------------------
        WsMessage::Sync { update } => {
            // 方案一：支持两种模式

            // 模式1：尝试 hex 解码（CRDT 二进制模式）
            if let Ok(update_bytes) = hex_decode(&update) {
                // 应用 CRDT 更新
                if room.apply_update(&update_bytes, user_id).is_ok() {
                    tracing::debug!("Applied CRDT update for doc {} from user {}",
                        room.doc_id(), user_id);
                    return true;  // 文档已修改
                } else {
                    tracing::warn!("Failed to apply CRDT update from user {}", user_id);
                }
            } else {
                // 模式2：纯文本模式（当前实现）
                // 直接替换文档内容（last-write-wins）
                if room.set_text_content(&update, user_id).is_ok() {
                    tracing::debug!("Applied text update for doc {} from user {} ({} bytes)",
                        room.doc_id(), user_id, update.len());
                    return true;  // 文档已修改
                } else {
                    tracing::warn!("Failed to apply text update from user {}", user_id);
                }
            }
        }

        // ---------------------------------------------
        // Awareness 消息：用户感知状态（光标、选择等）
        // ---------------------------------------------
        WsMessage::Awareness { state } => {
            room.update_awareness(user_id, state);
            tracing::debug!("Updated awareness for user {}", user_id);
            // Awareness 不修改文档内容，不需要保存
        }

        // 其他消息类型
        _ => {}
    }

    false  // 默认不需要保存
}

// ============================================================
// 第四部分：辅助函数
// ============================================================

/// 判断是否应该转发广播消息给当前用户
///
/// **规则：**
/// - 不转发自己发送的消息（避免循环）
fn should_forward(msg: &BroadcastMessage, current_user: Uuid) -> bool {
    match msg {
        // DocUpdate：CRDT 二进制更新
        BroadcastMessage::DocUpdate { from_user, .. } => *from_user != current_user,

        // TextUpdate：纯文本更新
        BroadcastMessage::TextUpdate { from_user, .. } => *from_user != current_user,

        // AwarenessUpdate：光标/选择更新
        BroadcastMessage::AwarenessUpdate { user_id, .. } => *user_id != current_user,

        // UserJoined：用户加入
        BroadcastMessage::UserJoined { user_id } => *user_id != current_user,

        // UserLeft：用户离开
        BroadcastMessage::UserLeft { user_id } => *user_id != current_user,
    }
}

/// 将内部广播消息转换为 WebSocket 消息
///
/// **用途：**
/// - BroadcastMessage 是后端内部使用的枚举
/// - WsMessage 是前后端约定的 JSON 格式
fn broadcast_to_ws_message(msg: BroadcastMessage) -> Option<WsMessage> {
    match msg {
        // CRDT 二进制更新 → hex 编码后发送
        BroadcastMessage::DocUpdate { update, .. } => {
            Some(WsMessage::Sync {
                update: hex_encode(&update),
            })
        }

        // 纯文本更新 → 直接发送
        BroadcastMessage::TextUpdate { content, .. } => {
            Some(WsMessage::Sync {
                update: content,
            })
        }

        // 感知状态更新
        BroadcastMessage::AwarenessUpdate { state, .. } => {
            Some(WsMessage::Awareness { state })
        }

        // 用户加入
        BroadcastMessage::UserJoined { user_id } => {
            Some(WsMessage::UserJoined {
                user_id,
                nickname: String::new(),  // TODO: 从 awareness 获取昵称
            })
        }

        // 用户离开
        BroadcastMessage::UserLeft { user_id } => {
            Some(WsMessage::UserLeft { user_id })
        }
    }
}

// ============================================================
// 第五部分：编码/解码工具函数
// ============================================================

/// 将二进制数据编码为十六进制字符串
///
/// **示例：**
/// ```
/// [0xFF, 0xAB] → "ffab"
/// ```
fn hex_encode(data: &[u8]) -> String {
    use std::fmt::Write;
    let mut result = String::with_capacity(data.len() * 2);
    for &byte in data {
        write!(&mut result, "{:02x}", byte).unwrap();
    }
    result
}

/// 将十六进制字符串解码为二进制数据
///
/// **示例：**
/// ```
/// "ffab" → [0xFF, 0xAB]
/// ```
fn hex_decode(s: &str) -> Result<Vec<u8>, ()> {
    // 检查长度必须是偶数
    if s.len() % 2 != 0 {
        return Err(());
    }

    let mut result = Vec::with_capacity(s.len() / 2);
    let mut chars = s.chars();

    // 每次取两个字符
    while let (Some(c1), Some(c2)) = (chars.next(), chars.next()) {
        // 解析为 u8
        let byte = u8::from_str_radix(&format!("{}{}", c1, c2), 16).map_err(|_| ())?;
        result.push(byte);
    }

    Ok(result)
}
```

---

## 文件2：房间管理器完整注释

**文件路径：`crates/collab/src/sync.rs`**

这个文件管理所有文档房间和广播机制。

```rust
use crate::awareness::{AwarenessManager, AwarenessState};
use crate::document::CollabDocument;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use uuid::Uuid;

// ============================================================
// 第一部分：广播消息定义
// ============================================================

/// 房间内的广播消息类型
///
/// **用途：**
/// - 在房间内的所有 WebSocket 连接之间传递消息
/// - 使用 Tokio broadcast channel 实现
#[derive(Debug, Clone)]
pub enum BroadcastMessage {
    /// CRDT 二进制更新
    ///
    /// **字段：**
    /// - `from_user`：发送者用户 ID（用于跳过回传）
    /// - `update`：Yrs CRDT 更新的二进制数据
    DocUpdate {
        from_user: Uuid,
        update: Vec<u8>,
    },

    /// 纯文本更新（简化模式）
    ///
    /// **字段：**
    /// - `from_user`：发送者用户 ID
    /// - `content`：完整的文档文本内容
    TextUpdate {
        from_user: Uuid,
        content: String,
    },

    /// 用户感知状态更新
    ///
    /// **字段：**
    /// - `user_id`：用户 ID
    /// - `state`：感知状态（光标位置、选择范围等）
    AwarenessUpdate {
        user_id: Uuid,
        state: AwarenessState,
    },

    /// 用户加入房间
    UserJoined {
        user_id: Uuid,
    },

    /// 用户离开房间
    UserLeft {
        user_id: Uuid,
    },
}

// ============================================================
// 第二部分：文档房间
// ============================================================

/// 文档房间 - 管理单个文档的所有协作会话
///
/// **职责：**
/// 1. 维护 CRDT 文档状态
/// 2. 管理在线用户列表
/// 3. 处理文档更新并广播
/// 4. 管理用户感知状态
pub struct DocumentRoom {
    /// 文档 ID
    doc_id: Uuid,

    /// CRDT 文档（Yrs）
    ///
    /// **作用：**
    /// - 存储文档的实际内容
    /// - 处理 CRDT 操作（插入、删除）
    /// - 生成和应用更新
    document: CollabDocument,

    /// 用户感知管理器
    ///
    /// **作用：**
    /// - 存储每个用户的光标位置
    /// - 存储每个用户的选择范围
    /// - 存储用户昵称和颜色
    awareness: AwarenessManager,

    /// 在线用户连接信息
    ///
    /// **为什么用 Arc<RwLock>？**
    /// - Arc：允许多个所有者（房间可能被克隆）
    /// - RwLock：读写锁，多个读者或一个写者
    /// - HashMap：用户 ID → 连接信息
    connections: Arc<RwLock<HashMap<Uuid, ConnectionInfo>>>,

    /// 广播通道的发送端
    ///
    /// **Tokio broadcast channel：**
    /// - 多生产者、多消费者
    /// - 每个消费者都会收到所有消息
    /// - 消息会被克隆（所以需要 Clone trait）
    broadcast_tx: broadcast::Sender<BroadcastMessage>,
}

/// 用户连接信息
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub user_id: Uuid,
    pub connected_at: chrono::DateTime<chrono::Utc>,
}

impl DocumentRoom {
    /// 创建新的文档房间
    pub fn new(doc_id: Uuid) -> Self {
        // 创建广播通道，容量 256 条消息
        // 如果消费者处理太慢，超过 256 条会触发 Lagged 错误
        let (broadcast_tx, _) = broadcast::channel(256);

        Self {
            doc_id,
            document: CollabDocument::new(doc_id),
            awareness: AwarenessManager::new(doc_id),
            connections: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
        }
    }

    /// 从已有 CRDT 状态创建房间
    ///
    /// **用途：**
    /// - 从数据库加载文档时使用
    /// - 恢复之前的编辑历史
    pub fn from_state(doc_id: Uuid, state: &[u8])
        -> Result<Self, crate::document::CollabError>
    {
        let (broadcast_tx, _) = broadcast::channel(256);
        Ok(Self {
            doc_id,
            document: CollabDocument::from_state(doc_id, state)?,
            awareness: AwarenessManager::new(doc_id),
            connections: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
        })
    }

    // ---------------------------------------------
    // 基本信息获取
    // ---------------------------------------------

    pub fn doc_id(&self) -> Uuid {
        self.doc_id
    }

    pub fn document(&self) -> &CollabDocument {
        &self.document
    }

    pub fn awareness(&self) -> &AwarenessManager {
        &self.awareness
    }

    /// 订阅广播消息
    ///
    /// **返回值：**
    /// - `broadcast::Receiver`：可以调用 recv() 接收消息
    ///
    /// **使用场景：**
    /// - 每个 WebSocket 连接都会调用这个方法
    /// - 获取一个接收器来监听房间内的所有消息
    pub fn subscribe(&self) -> broadcast::Receiver<BroadcastMessage> {
        self.broadcast_tx.subscribe()
    }

    // ---------------------------------------------
    // 用户管理
    // ---------------------------------------------

    /// 用户加入房间
    ///
    /// **步骤：**
    /// 1. 在 connections 中记录用户
    /// 2. 广播 UserJoined 消息
    pub fn user_join(&self, user_id: Uuid) {
        {
            // 获取写锁（独占访问）
            let mut connections = self.connections.write().unwrap();

            // 插入连接信息
            connections.insert(
                user_id,
                ConnectionInfo {
                    user_id,
                    connected_at: chrono::Utc::now(),
                },
            );
        }  // 写锁在这里自动释放

        // 广播用户加入消息
        // send() 返回 Result，我们忽略错误（可能没有订阅者）
        let _ = self.broadcast_tx.send(BroadcastMessage::UserJoined { user_id });
    }

    /// 用户离开房间
    ///
    /// **步骤：**
    /// 1. 从 connections 中移除用户
    /// 2. 清除用户的感知状态
    /// 3. 广播 UserLeft 消息
    pub fn user_leave(&self, user_id: &Uuid) {
        {
            let mut connections = self.connections.write().unwrap();
            connections.remove(user_id);
        }

        // 清除感知状态（光标、选择等）
        self.awareness.remove_state(user_id);

        // 广播用户离开消息
        let _ = self.broadcast_tx.send(BroadcastMessage::UserLeft {
            user_id: *user_id
        });
    }

    /// 获取在线用户列表
    pub fn get_online_users(&self) -> Vec<Uuid> {
        // 获取读锁（允许多个读者）
        let connections = self.connections.read().unwrap();
        connections.keys().copied().collect()
    }

    /// 获取在线用户数量
    pub fn get_user_count(&self) -> usize {
        let connections = self.connections.read().unwrap();
        connections.len()
    }

    // ---------------------------------------------
    // 文档更新
    // ---------------------------------------------

    /// 应用 CRDT 更新并广播
    ///
    /// **CRDT 模式：**
    /// - update 是 Yrs 生成的二进制数据
    /// - 包含操作历史，可以无冲突合并
    ///
    /// **步骤：**
    /// 1. 应用更新到 CRDT 文档
    /// 2. 广播给其他用户
    pub fn apply_update(&self, update: &[u8], from_user: Uuid)
        -> Result<(), crate::document::CollabError>
    {
        // 应用 CRDT 更新
        self.document.apply_update(update)?;

        // 广播更新给其他用户
        let _ = self.broadcast_tx.send(BroadcastMessage::DocUpdate {
            from_user,
            update: update.to_vec(),
        });

        Ok(())
    }

    /// 设置文本内容（简化模式）
    ///
    /// **简化模式：**
    /// - 直接替换文档内容（last-write-wins）
    /// - 不保留操作历史
    /// - 实现简单，但可能丢失并发编辑
    ///
    /// **步骤：**
    /// 1. 更新 CRDT 文档的文本
    /// 2. 广播 TextUpdate 消息
    pub fn set_text_content(&self, content: &str, from_user: Uuid)
        -> Result<(), crate::document::CollabError>
    {
        // 设置文本内容（内部会删除旧内容并插入新内容）
        self.document.set_text("content", content)?;

        // 广播文本更新给其他用户（简化模式）
        let _ = self.broadcast_tx.send(BroadcastMessage::TextUpdate {
            from_user,
            content: content.to_string(),
        });

        Ok(())
    }

    /// 获取文档的 CRDT 状态（用于持久化）
    pub fn get_state(&self) -> Vec<u8> {
        self.document.get_full_state()
    }

    /// 获取文本内容
    pub fn get_text_content(&self) -> String {
        self.document.get_default_text()
    }

    // ---------------------------------------------
    // 感知状态管理
    // ---------------------------------------------

    /// 更新用户感知状态并广播
    ///
    /// **感知状态包括：**
    /// - 光标位置
    /// - 选择范围
    /// - 用户昵称
    /// - 用户颜色
    pub fn update_awareness(&self, user_id: Uuid, state: AwarenessState) {
        // 存储状态
        self.awareness.set_state(user_id, state.clone());

        // 广播感知状态更新
        let _ = self.broadcast_tx.send(BroadcastMessage::AwarenessUpdate {
            user_id,
            state,
        });
    }

    /// 获取所有用户的感知状态
    pub fn get_all_awareness(&self) -> HashMap<Uuid, AwarenessState> {
        self.awareness.get_all_states()
    }
}

// 实现 Clone，因为房间可能被多个地方引用
impl Clone for DocumentRoom {
    fn clone(&self) -> Self {
        Self {
            doc_id: self.doc_id,
            document: self.document.clone(),
            awareness: self.awareness.clone(),
            connections: Arc::clone(&self.connections),  // 共享同一个 HashMap
            broadcast_tx: self.broadcast_tx.clone(),     // 共享同一个广播通道
        }
    }
}

// ============================================================
// 第三部分：房间管理器
// ============================================================

/// 房间管理器 - 管理所有文档房间
///
/// **职责：**
/// 1. 创建和销毁房间
/// 2. 查找房间
/// 3. 清理空房间
pub struct RoomManager {
    /// 所有房间的映射
    ///
    /// **为什么用 Arc<RwLock>？**
    /// - RoomManager 会被克隆到多个地方
    /// - 需要共享同一个 rooms HashMap
    /// - Arc 允许多个所有者
    /// - RwLock 允许并发读取
    rooms: Arc<RwLock<HashMap<Uuid, DocumentRoom>>>,
}

impl RoomManager {
    /// 创建新的房间管理器
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 获取或创建文档房间
    ///
    /// **逻辑：**
    /// 1. 先尝试获取现有房间
    /// 2. 如果不存在，创建新房间
    /// 3. 返回房间的克隆
    pub fn get_or_create_room(&self, doc_id: Uuid) -> DocumentRoom {
        // 获取写锁
        let mut rooms = self.rooms.write().unwrap();

        // 如果房间已存在，返回克隆
        if let Some(room) = rooms.get(&doc_id) {
            return room.clone();
        }

        // 创建新房间
        let room = DocumentRoom::new(doc_id);
        rooms.insert(doc_id, room.clone());
        room
    }

    /// 从已有状态获取或创建房间
    ///
    /// **用途：**
    /// - 从数据库加载文档时使用
    pub fn get_or_create_room_with_state(
        &self,
        doc_id: Uuid,
        state: &[u8],
    ) -> Result<DocumentRoom, crate::document::CollabError> {
        let mut rooms = self.rooms.write().unwrap();

        if let Some(room) = rooms.get(&doc_id) {
            return Ok(room.clone());
        }

        let room = DocumentRoom::from_state(doc_id, state)?;
        rooms.insert(doc_id, room.clone());
        Ok(room)
    }

    /// 获取房间（如果存在）
    pub fn get_room(&self, doc_id: &Uuid) -> Option<DocumentRoom> {
        let rooms = self.rooms.read().unwrap();
        rooms.get(doc_id).cloned()
    }

    /// 移除房间（当没有用户时）
    ///
    /// **逻辑：**
    /// 1. 先获取读锁检查用户数
    /// 2. 如果为 0，升级为写锁并移除
    ///
    /// **为什么要两步？**
    /// - 避免每次都获取写锁（写锁会阻塞其他读者）
    /// - 大部分情况房间不为空，只需读锁
    pub fn remove_room_if_empty(&self, doc_id: &Uuid) {
        // 步骤1：读锁检查
        let rooms = self.rooms.read().unwrap();

        if let Some(room) = rooms.get(doc_id) {
            if room.get_user_count() == 0 {
                // 步骤2：释放读锁，获取写锁
                drop(rooms);
                let mut rooms = self.rooms.write().unwrap();
                rooms.remove(doc_id);
            }
        }
    }

    /// 获取活跃房间数量
    pub fn get_active_room_count(&self) -> usize {
        let rooms = self.rooms.read().unwrap();
        rooms.len()
    }

    /// 获取所有房间 ID
    pub fn get_all_room_ids(&self) -> Vec<Uuid> {
        let rooms = self.rooms.read().unwrap();
        rooms.keys().copied().collect()
    }
}

// 实现 Default trait
impl Default for RoomManager {
    fn default() -> Self {
        Self::new()
    }
}

// 实现 Clone（共享同一个 rooms HashMap）
impl Clone for RoomManager {
    fn clone(&self) -> Self {
        Self {
            rooms: Arc::clone(&self.rooms),
        }
    }
}
```

---

## 关键概念总结

### 1. Arc<RwLock<T>> 模式

```rust
// Arc：Atomic Reference Counted（原子引用计数）
// - 允许多个所有者
// - 线程安全
let shared_data = Arc::new(data);
let clone1 = Arc::clone(&shared_data);
let clone2 = Arc::clone(&shared_data);

// RwLock：读写锁
// - 多个读者 OR 一个写者
// - 读优先（除非有写者等待）
let data = Arc::new(RwLock::new(HashMap::new()));

// 读取（可以多个线程同时读）
let readers = data.read().unwrap();
let value = readers.get(&key);

// 写入（独占访问）
let mut writers = data.write().unwrap();
writers.insert(key, value);
```

### 2. Tokio broadcast channel

```rust
// 创建通道（容量 256）
let (tx, _rx) = broadcast::channel(256);

// 订阅（创建新的接收器）
let mut rx1 = tx.subscribe();
let mut rx2 = tx.subscribe();

// 发送（所有接收器都会收到）
tx.send(message).ok();

// 接收
let msg = rx1.recv().await?;
```

### 3. tokio::select! 宏

```rust
loop {
    tokio::select! {
        // 等待多个异步操作，哪个先完成就处理哪个
        result1 = future1 => {
            // 处理 future1 的结果
        }
        result2 = future2 => {
            // 处理 future2 的结果
        }
        _ = timer.tick() => {
            // 定时器触发
        }
    }
}
```

---

**使用建议：**
1. 先阅读注释版代码，理解整体流程
2. 对照实际代码，加深理解
3. 运行程序，观察日志输出
4. 修改代码，观察行为变化
