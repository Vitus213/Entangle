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

/// 心跳间隔（秒）
const HEARTBEAT_INTERVAL: u64 = 30;
/// 心跳超时（秒）
const HEARTBEAT_TIMEOUT: u64 = 10;

/// WebSocket 连接处理
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path(doc_id): Path<Uuid>,
    user: AuthUser,
    Extension(hub): Extension<WsHub>,
    Extension(pool): Extension<PgPool>,
) -> Result<Response, AppError> {
    // 检查文档是否存在
    let doc = DocumentRepository::find_by_id(&pool, doc_id)
        .await
        .map_err(|e| AppError::Internal(format!("数据库错误: {}", e)))?
        .ok_or_else(|| AppError::NotFound("文档不存在".to_string()))?;

    // 检查用户是否有读取权限（WebSocket 需要至少读取权限）
    if !DocumentPermissionService::can_read(&pool, user.user_id, doc_id)
        .await
        .map_err(|e| AppError::Internal(format!("权限检查失败: {}", e)))?
    {
        return Err(AppError::Forbidden("无权访问该文档".to_string()));
    }

    tracing::info!(
        "User {} connecting to document {} (owner: {})",
        user.user_id,
        doc_id,
        doc.owner_id
    );

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, doc_id, user.user_id, hub)))
}

async fn handle_socket(socket: WebSocket, doc_id: Uuid, user_id: Uuid, hub: WsHub) {
    let (mut sender, mut receiver) = socket.split();

    // 尝试从数据库加载文档状态
    if let Err(e) = hub.load_document(doc_id).await {
        tracing::debug!("Could not load document from DB (may be new): {}", e);
    }

    // 获取或创建房间
    let room = hub.room_manager().get_or_create_room(doc_id);

    // 订阅房间广播
    let mut broadcast_rx = room.subscribe();

    // 用户加入房间
    room.user_join(user_id);
    tracing::info!("User {} joined document {}", user_id, doc_id);

    // 发送当前文档状态（方案一：发送文本内容）
    let text_content = room.get_text_content();

    if let Ok(msg) = serde_json::to_string(&WsMessage::Sync { update: text_content }) {
        if sender.send(Message::Text(msg)).await.is_err() {
            tracing::error!("Failed to send initial state to user {}", user_id);
            room.user_leave(&user_id);
            hub.room_manager().remove_room_if_empty(&doc_id);
            return;
        }
    }

    // 发送当前在线用户感知状态
    let awareness_states = room.get_all_awareness();
    for (uid, state) in awareness_states {
        if uid != user_id {
            if let Ok(msg) = serde_json::to_string(&WsMessage::Awareness { state }) {
                let _ = sender.send(Message::Text(msg)).await;
            }
        }
    }

    // 创建心跳定时器
    let mut heartbeat_interval = interval(Duration::from_secs(HEARTBEAT_INTERVAL));

    // 克隆 hub 用于异步任务
    let hub_clone = hub.clone();

    // 主循环
    loop {
        tokio::select! {
            // 处理传入消息
            msg_result = receiver.next() => {
                match msg_result {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                            if handle_client_message(&room, user_id, ws_msg).await {
                                // 标记文档为脏
                                hub_clone.mark_dirty(doc_id).await;
                            }
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        // 回复 Pong
                        if sender.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {
                        // 收到 Pong，心跳正常
                        tracing::trace!("Received pong from user {}", user_id);
                    }
                    Some(Ok(Message::Close(_))) => {
                        tracing::info!("User {} closed connection", user_id);
                        break;
                    }
                    Some(Err(e)) => {
                        tracing::warn!("WebSocket error for user {}: {}", user_id, e);
                        break;
                    }
                    None => {
                        tracing::info!("WebSocket stream ended for user {}", user_id);
                        break;
                    }
                    _ => {}
                }
            }

            // 处理广播消息
            broadcast_result = broadcast_rx.recv() => {
                match broadcast_result {
                    Ok(msg) => {
                        // 跳过自己发送的消息
                        if should_forward(&msg, user_id) {
                            if let Some(ws_msg) = broadcast_to_ws_message(msg) {
                                if let Ok(json) = serde_json::to_string(&ws_msg) {
                                    if sender.send(Message::Text(json)).await.is_err() {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("User {} lagged {} messages", user_id, n);
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        tracing::info!("Broadcast channel closed for user {}", user_id);
                        break;
                    }
                }
            }

            // 发送心跳
            _ = heartbeat_interval.tick() => {
                // 发送 Ping
                let ping_result = timeout(
                    Duration::from_secs(HEARTBEAT_TIMEOUT),
                    sender.send(Message::Ping(vec![]))
                ).await;

                match ping_result {
                    Ok(Ok(_)) => {
                        tracing::trace!("Sent ping to user {}", user_id);
                    }
                    Ok(Err(e)) => {
                        tracing::warn!("Failed to send ping to user {}: {}", user_id, e);
                        break;
                    }
                    Err(_) => {
                        tracing::warn!("Ping timeout for user {}", user_id);
                        break;
                    }
                }
            }
        }
    }

    // 用户离开
    room.user_leave(&user_id);

    // 如果房间为空，保存状态并清理
    let user_count = room.get_user_count();
    if user_count == 0 {
        // 保存文档状态
        if let Err(e) = hub.save_document(doc_id).await {
            tracing::warn!("Failed to save document {} on room close: {}", doc_id, e);
        }
    }

    hub.room_manager().remove_room_if_empty(&doc_id);
    tracing::info!("User {} left document {}", user_id, doc_id);
}

/// 处理客户端消息，返回是否修改了文档
async fn handle_client_message(
    room: &entangle_collab::DocumentRoom,
    user_id: Uuid,
    msg: WsMessage,
) -> bool {
    match msg {
        WsMessage::Sync { update } => {
            // 方案一：简化文本同步模式
            // 尝试 hex 解码，如果失败则视为纯文本内容
            if let Ok(update_bytes) = hex_decode(&update) {
                // CRDT 模式：应用二进制更新
                if room.apply_update(&update_bytes, user_id).is_ok() {
                    tracing::debug!("Applied CRDT update for doc {} from user {}", room.doc_id(), user_id);
                    return true;
                } else {
                    tracing::warn!("Failed to apply CRDT update from user {}", user_id);
                }
            } else {
                // 简化模式：直接用纯文本替换文档内容
                if room.set_text_content(&update, user_id).is_ok() {
                    tracing::debug!("Applied text update for doc {} from user {} ({} bytes)",
                        room.doc_id(), user_id, update.len());
                    return true;
                } else {
                    tracing::warn!("Failed to apply text update from user {}", user_id);
                }
            }
        }
        WsMessage::Awareness { state } => {
            room.update_awareness(user_id, state);
            tracing::debug!("Updated awareness for user {}", user_id);
        }
        _ => {}
    }
    false
}

/// 判断是否应该转发广播消息
fn should_forward(msg: &BroadcastMessage, current_user: Uuid) -> bool {
    match msg {
        BroadcastMessage::DocUpdate { from_user, .. } => *from_user != current_user,
        BroadcastMessage::TextUpdate { from_user, .. } => *from_user != current_user,
        BroadcastMessage::AwarenessUpdate { user_id, .. } => *user_id != current_user,
        BroadcastMessage::UserJoined { user_id } => *user_id != current_user,
        BroadcastMessage::UserLeft { user_id } => *user_id != current_user,
    }
}

/// 将广播消息转换为 WebSocket 消息
fn broadcast_to_ws_message(msg: BroadcastMessage) -> Option<WsMessage> {
    match msg {
        BroadcastMessage::DocUpdate { update, .. } => {
            Some(WsMessage::Sync {
                update: hex_encode(&update),
            })
        }
        BroadcastMessage::TextUpdate { content, .. } => {
            Some(WsMessage::Sync {
                update: content,
            })
        }
        BroadcastMessage::AwarenessUpdate { state, .. } => {
            Some(WsMessage::Awareness { state })
        }
        BroadcastMessage::UserJoined { user_id } => {
            Some(WsMessage::UserJoined {
                user_id,
                nickname: String::new(), // 可以从 awareness 获取
            })
        }
        BroadcastMessage::UserLeft { user_id } => {
            Some(WsMessage::UserLeft { user_id })
        }
    }
}

/// 十六进制编码
fn hex_encode(data: &[u8]) -> String {
    use std::fmt::Write;
    let mut result = String::with_capacity(data.len() * 2);
    for &byte in data {
        write!(&mut result, "{:02x}", byte).unwrap();
    }
    result
}

/// 十六进制解码
fn hex_decode(s: &str) -> Result<Vec<u8>, ()> {
    if s.len() % 2 != 0 {
        return Err(());
    }

    let mut result = Vec::with_capacity(s.len() / 2);
    let mut chars = s.chars();

    while let (Some(c1), Some(c2)) = (chars.next(), chars.next()) {
        let byte = u8::from_str_radix(&format!("{}{}", c1, c2), 16).map_err(|_| ())?;
        result.push(byte);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_encode_decode() {
        let data = vec![0xFF, 0xAB, 0x12, 0x00];
        let encoded = hex_encode(&data);
        assert_eq!(encoded, "ffab1200");

        let decoded = hex_decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_hex_decode_invalid() {
        assert!(hex_decode("invalid").is_err());
        assert!(hex_decode("abc").is_err()); // 奇数长度
    }
}
