use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{Doc, GetString, ReadTxn, StateVector, Text, Transact, Update, WriteTxn};

#[derive(Debug, Error)]
pub enum CollabError {
    #[error("Failed to apply update: {0}")]
    ApplyUpdateError(String),

    #[error("Failed to serialize state: {0}")]
    SerializeError(String),

    #[error("Failed to get text: {0}")]
    GetTextError(String),

    #[error("Lock poisoned")]
    LockError,
}

/// CRDT 文档，基于 Yrs
#[derive(Clone)]
pub struct CollabDocument {
    doc_id: Uuid,
    doc: Arc<Doc>,
}

impl CollabDocument {
    /// 创建新的协作文档
    pub fn new(doc_id: Uuid) -> Self {
        Self {
            doc_id,
            doc: Arc::new(Doc::new()),
        }
    }

    /// 从已有状态创建文档
    pub fn from_state(doc_id: Uuid, state: &[u8]) -> Result<Self, CollabError> {
        let doc = Doc::new();
        let mut txn = doc.transact_mut();

        if !state.is_empty() {
            let update = Update::decode_v1(state)
                .map_err(|e| CollabError::ApplyUpdateError(e.to_string()))?;
            txn.apply_update(update);
        }

        drop(txn);

        Ok(Self {
            doc_id,
            doc: Arc::new(doc),
        })
    }

    /// 获取文档 ID
    pub fn doc_id(&self) -> Uuid {
        self.doc_id
    }

    /// 获取底层 Yrs Doc 的引用
    pub fn get_doc(&self) -> &Doc {
        &self.doc
    }

    /// 应用更新
    pub fn apply_update(&self, update: &[u8]) -> Result<(), CollabError> {
        let mut txn = self.doc.transact_mut();
        let update = Update::decode_v1(update)
            .map_err(|e| CollabError::ApplyUpdateError(e.to_string()))?;
        txn.apply_update(update);
        Ok(())
    }

    /// 获取当前状态向量
    pub fn get_state_vector(&self) -> Vec<u8> {
        let txn = self.doc.transact();
        let sv = txn.state_vector();
        sv.encode_v1()
    }

    /// 获取从某个状态向量以来的更新
    pub fn get_update_since(&self, state_vector: &[u8]) -> Result<Vec<u8>, CollabError> {
        let txn = self.doc.transact();
        let sv = StateVector::decode_v1(state_vector)
            .map_err(|e| CollabError::SerializeError(e.to_string()))?;
        let update = txn.encode_diff_v1(&sv);
        Ok(update)
    }

    /// 获取完整状态（用于持久化）
    pub fn get_full_state(&self) -> Vec<u8> {
        let txn = self.doc.transact();
        txn.encode_state_as_update_v1(&StateVector::default())
    }

    /// 获取文本内容（如果文档包含文本）
    pub fn get_text(&self, text_name: &str) -> Result<String, CollabError> {
        let txn = self.doc.transact();
        let text = txn
            .get_text(text_name)
            .ok_or_else(|| CollabError::GetTextError(format!("Text '{}' not found", text_name)))?;
        Ok(text.get_string(&txn))
    }

    /// 获取默认文本内容（使用 "content" 作为名称）
    pub fn get_default_text(&self) -> String {
        self.get_text("content").unwrap_or_default()
    }

    /// 设置文本内容
    pub fn set_text(&self, text_name: &str, content: &str) -> Result<(), CollabError> {
        let mut txn = self.doc.transact_mut();
        let text = txn.get_or_insert_text(text_name);

        // 删除现有内容
        let len = text.len(&txn);
        if len > 0 {
            text.remove_range(&mut txn, 0, len);
        }

        // 插入新内容
        text.insert(&mut txn, 0, content);

        Ok(())
    }

    /// 获取元数据
    pub fn get_metadata(&self) -> DocumentMetadata {
        let state = self.get_full_state();
        DocumentMetadata {
            doc_id: self.doc_id,
            state_size: state.len(),
        }
    }
}

impl Default for CollabDocument {
    fn default() -> Self {
        Self::new(Uuid::new_v4())
    }
}

/// 文档元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub doc_id: Uuid,
    pub state_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_document() {
        let doc_id = Uuid::new_v4();
        let doc = CollabDocument::new(doc_id);
        assert_eq!(doc.doc_id(), doc_id);
    }

    #[test]
    fn test_text_operations() {
        let doc = CollabDocument::new(Uuid::new_v4());

        // 设置文本
        doc.set_text("content", "Hello, World!").unwrap();

        // 获取文本
        let text = doc.get_text("content").unwrap();
        assert_eq!(text, "Hello, World!");
    }

    #[test]
    fn test_state_persistence() {
        let doc_id = Uuid::new_v4();
        let doc1 = CollabDocument::new(doc_id);

        // 在第一个文档中设置文本
        doc1.set_text("content", "Test content").unwrap();

        // 获取状态
        let state = doc1.get_full_state();

        // 从状态创建第二个文档
        let doc2 = CollabDocument::from_state(doc_id, &state).unwrap();

        // 验证内容相同
        let text = doc2.get_text("content").unwrap();
        assert_eq!(text, "Test content");
    }
}
