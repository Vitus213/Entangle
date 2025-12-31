use entangle_collab::{AwarenessState, RoomManager};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

pub mod handlers;

/// WebSocket 消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    /// 同步文档更新
    Sync { update: String },
    /// 用户感知状态
    Awareness { state: AwarenessState },
    /// 用户加入
    UserJoined { user_id: Uuid, nickname: String },
    /// 用户离开
    UserLeft { user_id: Uuid },
    /// 错误消息
    Error { message: String },
}

/// WebSocket 连接管理器
pub struct WsHub {
    room_manager: Arc<RoomManager>,
}

impl WsHub {
    pub fn new() -> Self {
        Self {
            room_manager: Arc::new(RoomManager::new()),
        }
    }

    pub fn room_manager(&self) -> &RoomManager {
        &self.room_manager
    }
}

impl Default for WsHub {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for WsHub {
    fn clone(&self) -> Self {
        Self {
            room_manager: Arc::clone(&self.room_manager),
        }
    }
}
