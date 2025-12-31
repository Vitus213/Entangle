use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

/// 用户感知状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwarenessState {
    /// 用户信息
    pub user: UserInfo,
    /// 光标位置（可选）
    pub cursor: Option<CursorPosition>,
    /// 选区（可选）
    pub selection: Option<Selection>,
    /// 用户颜色（用于显示）
    pub color: String,
}

/// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub user_id: Uuid,
    pub nickname: String,
    pub avatar_url: Option<String>,
}

/// 光标位置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub line: u32,
    pub column: u32,
}

/// 选区
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Selection {
    pub start: CursorPosition,
    pub end: CursorPosition,
}

/// 感知管理器
pub struct AwarenessManager {
    doc_id: Uuid,
    states: Arc<RwLock<HashMap<Uuid, AwarenessState>>>,
}

impl AwarenessManager {
    /// 创建新的感知管理器
    pub fn new(doc_id: Uuid) -> Self {
        Self {
            doc_id,
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 添加或更新用户状态
    pub fn set_state(&self, user_id: Uuid, state: AwarenessState) {
        let mut states = self.states.write().unwrap();
        states.insert(user_id, state);
    }

    /// 获取用户状态
    pub fn get_state(&self, user_id: &Uuid) -> Option<AwarenessState> {
        let states = self.states.read().unwrap();
        states.get(user_id).cloned()
    }

    /// 移除用户状态
    pub fn remove_state(&self, user_id: &Uuid) -> Option<AwarenessState> {
        let mut states = self.states.write().unwrap();
        states.remove(user_id)
    }

    /// 获取所有在线用户状态
    pub fn get_all_states(&self) -> HashMap<Uuid, AwarenessState> {
        let states = self.states.read().unwrap();
        states.clone()
    }

    /// 获取在线用户数量
    pub fn get_online_count(&self) -> usize {
        let states = self.states.read().unwrap();
        states.len()
    }

    /// 清空所有状态
    pub fn clear(&self) {
        let mut states = self.states.write().unwrap();
        states.clear();
    }

    /// 获取文档 ID
    pub fn doc_id(&self) -> Uuid {
        self.doc_id
    }
}

impl Clone for AwarenessManager {
    fn clone(&self) -> Self {
        Self {
            doc_id: self.doc_id,
            states: Arc::clone(&self.states),
        }
    }
}

/// 生成随机颜色（用于用户标识）
pub fn generate_user_color(user_id: &Uuid) -> String {
    let colors = [
        "#FF6B6B", "#4ECDC4", "#45B7D1", "#FFA07A", "#98D8C8", "#F7DC6F", "#BB8FCE", "#85C1E2",
        "#F8B500", "#52B788",
    ];

    // 使用用户 ID 的哈希值来选择颜色
    let bytes = user_id.as_bytes();
    let index = bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b)) as usize % colors.len();

    colors[index].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_awareness_manager() {
        let doc_id = Uuid::new_v4();
        let manager = AwarenessManager::new(doc_id);

        let user_id = Uuid::new_v4();
        let state = AwarenessState {
            user: UserInfo {
                user_id,
                nickname: "Test User".to_string(),
                avatar_url: None,
            },
            cursor: Some(CursorPosition { line: 1, column: 0 }),
            selection: None,
            color: generate_user_color(&user_id),
        };

        // 设置状态
        manager.set_state(user_id, state.clone());

        // 获取状态
        let retrieved = manager.get_state(&user_id).unwrap();
        assert_eq!(retrieved.user.nickname, "Test User");

        // 检查在线数量
        assert_eq!(manager.get_online_count(), 1);

        // 移除状态
        manager.remove_state(&user_id);
        assert_eq!(manager.get_online_count(), 0);
    }

    #[test]
    fn test_generate_color() {
        let user_id = Uuid::new_v4();
        let color = generate_user_color(&user_id);
        assert!(color.starts_with('#'));
        assert_eq!(color.len(), 7);
    }
}
