use entangle_collab::{AwarenessState, RoomManager};
use entangle_db::repository::DocumentRepository;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
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

/// 持久化状态跟踪
struct PersistenceState {
    /// 上次保存时间
    last_save: std::time::Instant,
    /// 是否有未保存的更改
    dirty: bool,
}

/// WebSocket 连接管理器
pub struct WsHub {
    room_manager: Arc<RoomManager>,
    /// 数据库连接池
    pool: Option<Arc<PgPool>>,
    /// 持久化状态跟踪
    persistence_states: Arc<RwLock<std::collections::HashMap<Uuid, PersistenceState>>>,
}

/// 自动保存间隔（秒）
const AUTO_SAVE_INTERVAL: u64 = 30;

impl WsHub {
    pub fn new() -> Self {
        Self {
            room_manager: Arc::new(RoomManager::new()),
            pool: None,
            persistence_states: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// 创建带数据库连接的 WsHub
    pub fn with_pool(pool: PgPool) -> Self {
        let hub = Self {
            room_manager: Arc::new(RoomManager::new()),
            pool: Some(Arc::new(pool)),
            persistence_states: Arc::new(RwLock::new(std::collections::HashMap::new())),
        };

        // 启动后台持久化任务
        hub.start_persistence_task();
        hub
    }

    pub fn room_manager(&self) -> &RoomManager {
        &self.room_manager
    }

    /// 获取数据库连接池
    pub fn pool(&self) -> Option<&PgPool> {
        self.pool.as_deref()
    }

    /// 标记文档为脏（有未保存的更改）
    pub async fn mark_dirty(&self, doc_id: Uuid) {
        let mut states = self.persistence_states.write().await;
        states
            .entry(doc_id)
            .and_modify(|s| s.dirty = true)
            .or_insert(PersistenceState {
                last_save: std::time::Instant::now(),
                dirty: true,
            });
    }

    /// 保存文档 CRDT 状态到数据库
    pub async fn save_document(&self, doc_id: Uuid) -> Result<(), String> {
        let pool = self.pool.as_ref().ok_or("No database pool available")?;
        let room = self.room_manager.get_room(&doc_id).ok_or("Room not found")?;

        let state = room.get_state();
        let content = room.document().get_default_text();

        DocumentRepository::save_crdt_state_with_content(pool, doc_id, &state, &content)
            .await
            .map_err(|e| e.to_string())?;

        // 更新持久化状态
        let mut states = self.persistence_states.write().await;
        if let Some(ps) = states.get_mut(&doc_id) {
            ps.last_save = std::time::Instant::now();
            ps.dirty = false;
        }

        tracing::info!("Saved CRDT state for document {}", doc_id);
        Ok(())
    }

    /// 从数据库加载文档 CRDT 状态
    pub async fn load_document(&self, doc_id: Uuid) -> Result<(), String> {
        let pool = self.pool.as_ref().ok_or("No database pool available")?;

        if let Some(state) = DocumentRepository::get_crdt_state(pool, doc_id)
            .await
            .map_err(|e| e.to_string())?
        {
            self.room_manager
                .get_or_create_room_with_state(doc_id, &state)
                .map_err(|e| e.to_string())?;
            tracing::info!("Loaded CRDT state for document {}", doc_id);
        }

        Ok(())
    }

    /// 启动后台持久化任务
    fn start_persistence_task(&self) {
        let room_manager = Arc::clone(&self.room_manager);
        let pool = self.pool.clone();
        let persistence_states = Arc::clone(&self.persistence_states);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(AUTO_SAVE_INTERVAL));

            loop {
                interval.tick().await;

                if let Some(ref pool) = pool {
                    // 获取所有活跃房间
                    let room_ids = room_manager.get_all_room_ids();

                    for doc_id in room_ids {
                        // 检查是否需要保存
                        let should_save = {
                            let states = persistence_states.read().await;
                            states.get(&doc_id).map(|s| s.dirty).unwrap_or(false)
                        };

                        if should_save {
                            if let Some(room) = room_manager.get_room(&doc_id) {
                                let state = room.get_state();
                                let content = room.document().get_default_text();

                                if let Err(e) = DocumentRepository::save_crdt_state_with_content(
                                    pool, doc_id, &state, &content,
                                )
                                .await
                                {
                                    tracing::error!("Failed to save CRDT state for {}: {}", doc_id, e);
                                } else {
                                    let mut states = persistence_states.write().await;
                                    if let Some(ps) = states.get_mut(&doc_id) {
                                        ps.last_save = std::time::Instant::now();
                                        ps.dirty = false;
                                    }
                                    tracing::debug!("Auto-saved CRDT state for {}", doc_id);
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    /// 保存所有脏文档（用于关闭时）
    pub async fn save_all_dirty(&self) {
        let pool = match &self.pool {
            Some(p) => p,
            None => return,
        };

        let room_ids = self.room_manager.get_all_room_ids();

        for doc_id in room_ids {
            let should_save = {
                let states = self.persistence_states.read().await;
                states.get(&doc_id).map(|s| s.dirty).unwrap_or(false)
            };

            if should_save {
                if let Some(room) = self.room_manager.get_room(&doc_id) {
                    let state = room.get_state();
                    let content = room.document().get_default_text();

                    if let Err(e) =
                        DocumentRepository::save_crdt_state_with_content(pool, doc_id, &state, &content).await
                    {
                        tracing::error!("Failed to save CRDT state for {}: {}", doc_id, e);
                    } else {
                        tracing::info!("Saved CRDT state for {} on shutdown", doc_id);
                    }
                }
            }
        }
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
            pool: self.pool.clone(),
            persistence_states: Arc::clone(&self.persistence_states),
        }
    }
}
