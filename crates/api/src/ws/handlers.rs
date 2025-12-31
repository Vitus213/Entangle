use crate::middleware::AuthUser;
use crate::ws::WsHub;
use axum::{
    extract::{ws::WebSocketUpgrade, ws::Message, ws::WebSocket, Path},
    response::Response,
    Extension,
};
use futures::{sink::SinkExt, stream::StreamExt};
use uuid::Uuid;

/// WebSocket 连接处理
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path(doc_id): Path<Uuid>,
    user: AuthUser,
    Extension(hub): Extension<WsHub>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, doc_id, user.user_id, hub))
}

async fn handle_socket(socket: WebSocket, doc_id: Uuid, user_id: Uuid, hub: WsHub) {
    let (mut sender, mut receiver) = socket.split();

    // 获取或创建房间
    let room = hub.room_manager().get_or_create_room(doc_id);

    // 用户加入房间
    room.user_join(user_id);

    // 发送当前文档状态
    let state = room.get_state();
    let state_b64 = base64::encode(&state);

    if let Ok(msg) = serde_json::to_string(&crate::ws::WsMessage::Sync {
        update: state_b64,
    }) {
        let _: Result<(), _> = sender.send(Message::Text(msg)).await;
    }

    // 处理传入消息
    while let Some(msg_result) = receiver.next().await {
        if let Ok(Message::Text(text)) = msg_result {
            // 解析消息
            if let Ok(ws_msg) = serde_json::from_str::<crate::ws::WsMessage>(&text) {
                match ws_msg {
                    crate::ws::WsMessage::Sync { update } => {
                        // 应用更新
                        if let Ok(update_bytes) = base64::decode(&update) {
                            if room.apply_update(&update_bytes).is_ok() {
                                // 这里应该广播给其他用户，但简化版本暂时跳过
                                tracing::info!("Applied update for doc {} from user {}", doc_id, user_id);
                            }
                        }
                    }
                    crate::ws::WsMessage::Awareness { state } => {
                        room.update_awareness(user_id, state);
                    }
                    _ => {}
                }
            }
        }
    }

    // 用户离开
    room.user_leave(&user_id);
    hub.room_manager().remove_room_if_empty(&doc_id);
}

// 添加 base64 依赖的简单实现
mod base64 {
    pub fn encode(data: &[u8]) -> String {
        use std::fmt::Write;
        let mut result = String::new();
        for &byte in data {
            write!(&mut result, "{:02x}", byte).unwrap();
        }
        result
    }

    pub fn decode(s: &str) -> Result<Vec<u8>, ()> {
        let mut result = Vec::new();
        let mut chars = s.chars();

        while let (Some(c1), Some(c2)) = (chars.next(), chars.next()) {
            let byte = u8::from_str_radix(&format!("{}{}", c1, c2), 16).map_err(|_| ())?;
            result.push(byte);
        }

        Ok(result)
    }
}
