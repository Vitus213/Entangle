use crate::awareness::{AwarenessManager, AwarenessState};
use crate::document::CollabDocument;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use uuid::Uuid;

/// 广播消息类型
#[derive(Debug, Clone)]
pub enum BroadcastMessage {
    /// 文档更新（CRDT 二进制）
    DocUpdate {
        from_user: Uuid,
        update: Vec<u8>,
    },
    /// 文本更新（简化模式）
    TextUpdate {
        from_user: Uuid,
        content: String,
    },
    /// 用户感知状态更新
    AwarenessUpdate {
        user_id: Uuid,
        state: AwarenessState,
    },
    /// 用户加入
    UserJoined {
        user_id: Uuid,
        nickname: String,
    },
    /// 用户离开
    UserLeft {
        user_id: Uuid,
    },
}

/// 文档房间 - 管理单个文档的所有协作会话
pub struct DocumentRoom {
    /// 文档 ID
    doc_id: Uuid,
    /// CRDT 文档
    document: CollabDocument,
    /// 用户感知管理器
    awareness: AwarenessManager,
    /// 在线用户连接
    connections: Arc<RwLock<HashMap<Uuid, ConnectionInfo>>>,
    /// 广播通道发送端
    broadcast_tx: broadcast::Sender<BroadcastMessage>,
}

/// 连接信息
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub user_id: Uuid,
    pub connected_at: chrono::DateTime<chrono::Utc>,
}

impl DocumentRoom {
    /// 创建新的文档房间
    pub fn new(doc_id: Uuid) -> Self {
        let (broadcast_tx, _) = broadcast::channel(256);
        Self {
            doc_id,
            document: CollabDocument::new(doc_id),
            awareness: AwarenessManager::new(doc_id),
            connections: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
        }
    }

    /// 从已有状态创建房间
    pub fn from_state(doc_id: Uuid, state: &[u8]) -> Result<Self, crate::document::CollabError> {
        let (broadcast_tx, _) = broadcast::channel(256);
        Ok(Self {
            doc_id,
            document: CollabDocument::from_state(doc_id, state)?,
            awareness: AwarenessManager::new(doc_id),
            connections: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
        })
    }

    /// 获取文档 ID
    pub fn doc_id(&self) -> Uuid {
        self.doc_id
    }

    /// 获取文档引用
    pub fn document(&self) -> &CollabDocument {
        &self.document
    }

    /// 获取感知管理器
    pub fn awareness(&self) -> &AwarenessManager {
        &self.awareness
    }

    /// 订阅广播消息
    pub fn subscribe(&self) -> broadcast::Receiver<BroadcastMessage> {
        self.broadcast_tx.subscribe()
    }

    /// 用户加入房间
    pub fn user_join(&self, user_id: Uuid, nickname: String) {
        {
            let mut connections = self.connections.write().unwrap();
            connections.insert(
                user_id,
                ConnectionInfo {
                    user_id,
                    connected_at: chrono::Utc::now(),
                },
            );
        }
        // 广播用户加入消息
        let _ = self.broadcast_tx.send(BroadcastMessage::UserJoined { user_id, nickname });
    }

    /// 用户离开房间
    pub fn user_leave(&self, user_id: &Uuid) {
        {
            let mut connections = self.connections.write().unwrap();
            connections.remove(user_id);
        }
        self.awareness.remove_state(user_id);
        // 广播用户离开消息
        let _ = self.broadcast_tx.send(BroadcastMessage::UserLeft { user_id: *user_id });
    }

    /// 获取在线用户列表
    pub fn get_online_users(&self) -> Vec<Uuid> {
        let connections = self.connections.read().unwrap();
        connections.keys().copied().collect()
    }

    /// 获取在线用户数量
    pub fn get_user_count(&self) -> usize {
        let connections = self.connections.read().unwrap();
        connections.len()
    }

    /// 应用文档更新并广播
    pub fn apply_update(&self, update: &[u8], from_user: Uuid) -> Result<(), crate::document::CollabError> {
        self.document.apply_update(update)?;
        // 广播更新给其他用户
        let _ = self.broadcast_tx.send(BroadcastMessage::DocUpdate {
            from_user,
            update: update.to_vec(),
        });
        Ok(())
    }

    /// 仅应用文档更新（不广播，用于初始化）
    pub fn apply_update_silent(&self, update: &[u8]) -> Result<(), crate::document::CollabError> {
        self.document.apply_update(update)
    }

    /// 设置文本内容（简化模式）- 直接替换文档内容并广播
    pub fn set_text_content(&self, content: &str, from_user: Uuid) -> Result<(), crate::document::CollabError> {
        // 设置文本内容
        self.document.set_text("content", content)?;

        // 广播文本更新给其他用户（简化模式）
        let _ = self.broadcast_tx.send(BroadcastMessage::TextUpdate {
            from_user,
            content: content.to_string(),
        });

        Ok(())
    }

    /// 获取文档状态
    pub fn get_state(&self) -> Vec<u8> {
        self.document.get_full_state()
    }

    /// 获取文本内容
    pub fn get_text_content(&self) -> String {
        self.document.get_default_text()
    }

    /// 更新用户感知状态并广播
    pub fn update_awareness(&self, user_id: Uuid, state: AwarenessState) {
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

impl Clone for DocumentRoom {
    fn clone(&self) -> Self {
        Self {
            doc_id: self.doc_id,
            document: self.document.clone(),
            awareness: self.awareness.clone(),
            connections: Arc::clone(&self.connections),
            broadcast_tx: self.broadcast_tx.clone(),
        }
    }
}

/// 房间管理器 - 管理所有文档房间
pub struct RoomManager {
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
    pub fn get_or_create_room(&self, doc_id: Uuid) -> DocumentRoom {
        let mut rooms = self.rooms.write().unwrap();

        if let Some(room) = rooms.get(&doc_id) {
            return room.clone();
        }

        let room = DocumentRoom::new(doc_id);
        rooms.insert(doc_id, room.clone());
        room
    }

    /// 从已有状态获取或创建房间
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

    /// 获取房间
    pub fn get_room(&self, doc_id: &Uuid) -> Option<DocumentRoom> {
        let rooms = self.rooms.read().unwrap();
        rooms.get(doc_id).cloned()
    }

    /// 移除房间（当没有用户时）
    pub fn remove_room_if_empty(&self, doc_id: &Uuid) {
        let rooms = self.rooms.read().unwrap();

        if let Some(room) = rooms.get(doc_id) {
            if room.get_user_count() == 0 {
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

impl Default for RoomManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for RoomManager {
    fn clone(&self) -> Self {
        Self {
            rooms: Arc::clone(&self.rooms),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_room() {
        let doc_id = Uuid::new_v4();
        let room = DocumentRoom::new(doc_id);

        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();

        // 用户加入
        room.user_join(user1, "User 1".to_string());
        room.user_join(user2, "User 2".to_string());

        assert_eq!(room.get_user_count(), 2);

        // 用户离开
        room.user_leave(&user1);
        assert_eq!(room.get_user_count(), 1);
    }

    #[test]
    fn test_room_manager() {
        let manager = RoomManager::new();

        let doc_id1 = Uuid::new_v4();
        let doc_id2 = Uuid::new_v4();

        // 创建房间
        let room1 = manager.get_or_create_room(doc_id1);
        let _room2 = manager.get_or_create_room(doc_id2);

        assert_eq!(manager.get_active_room_count(), 2);

        // 用户加入房间1
        room1.user_join(Uuid::new_v4(), "Test User".to_string());

        // 移除空房间
        manager.remove_room_if_empty(&doc_id2);
        assert_eq!(manager.get_active_room_count(), 1);
    }

    #[tokio::test]
    async fn test_broadcast() {
        let doc_id = Uuid::new_v4();
        let room = DocumentRoom::new(doc_id);

        let user1 = Uuid::new_v4();

        // 订阅广播
        let mut rx = room.subscribe();

        // 用户加入
        room.user_join(user1, "Test User".to_string());

        // 接收广播消息
        if let Ok(BroadcastMessage::UserJoined { user_id, nickname }) = rx.recv().await {
            assert_eq!(user_id, user1);
            assert_eq!(nickname, "Test User");
        } else {
            panic!("Expected UserJoined message");
        }

        // 用户离开
        room.user_leave(&user1);

        if let Ok(BroadcastMessage::UserLeft { user_id }) = rx.recv().await {
            assert_eq!(user_id, user1);
        } else {
            panic!("Expected UserLeft message");
        }
    }
}
