use yrs::{Doc, GetString, ReadTxn, StateVector, Transact, Update, Text};
use yrs::updates::decoder::Decode;
use std::rc::Rc;
use std::cell::RefCell;
use web_sys::HtmlTextAreaElement;

/// CRDT 文档管理器
pub struct CrdtManager {
    doc: Rc<Doc>,
    text_name: String,
    // 防止循环更新的标志
    updating: Rc<RefCell<bool>>,
}

impl CrdtManager {
    /// 创建新的 CRDT 管理器
    pub fn new() -> Self {
        Self {
            doc: Rc::new(Doc::new()),
            text_name: "content".to_string(),
            updating: Rc::new(RefCell::new(false)),
        }
    }

    /// 从二进制状态初始化文档
    pub fn init_from_state(&mut self, state: &[u8]) -> Result<(), String> {
        let update = Update::decode_v1(state)
            .map_err(|e| format!("解码状态失败: {:?}", e))?;

        let mut txn = self.doc.transact_mut();
        txn.apply_update(update);

        Ok(())
    }

    /// 获取当前文档的完整状态
    pub fn get_state(&self) -> Vec<u8> {
        let txn = self.doc.transact();
        txn.encode_state_as_update_v1(&StateVector::default())
    }

    /// 获取文档的文本内容
    pub fn get_text(&self) -> String {
        let txn = self.doc.transact();
        if let Some(text) = txn.get_text(self.text_name.as_str()) {
            text.get_string(&txn)
        } else {
            String::new()
        }
    }

    /// 设置文档的文本内容（替换全部）
    pub fn set_text(&mut self, content: &str) {
        if *self.updating.borrow() {
            return; // 防止循环更新
        }

        *self.updating.borrow_mut() = true;

        let text = self.doc.get_or_insert_text(self.text_name.as_str());
        let mut txn = self.doc.transact_mut();

        // 删除现有内容
        let current_len = text.len(&txn);
        if current_len > 0 {
            text.remove_range(&mut txn, 0, current_len);
        }

        // 插入新内容
        if !content.is_empty() {
            text.insert(&mut txn, 0, content);
        }

        drop(txn);
        *self.updating.borrow_mut() = false;
    }

    /// 应用远程更新
    pub fn apply_update(&mut self, update: &[u8]) -> Result<(), String> {
        if *self.updating.borrow() {
            return Ok(()); // 防止循环更新
        }

        *self.updating.borrow_mut() = true;

        let update = Update::decode_v1(update)
            .map_err(|e| format!("解码更新失败: {:?}", e))?;

        let mut txn = self.doc.transact_mut();
        txn.apply_update(update);
        drop(txn);

        *self.updating.borrow_mut() = false;

        Ok(())
    }

    /// 获取 Doc 引用（用于设置观察者）
    #[allow(dead_code)]
    pub fn get_doc(&self) -> Rc<Doc> {
        self.doc.clone()
    }

    /// 获取文本名称
    #[allow(dead_code)]
    pub fn get_text_name(&self) -> &str {
        &self.text_name
    }

    /// 将文档内容同步到 textarea
    #[allow(dead_code)]
    pub fn sync_to_textarea(&self, textarea: &HtmlTextAreaElement) {
        let text = self.get_text();
        textarea.set_value(&text);
    }

    /// 从 textarea 同步内容到文档
    #[allow(dead_code)]
    pub fn sync_from_textarea(&mut self, textarea: &HtmlTextAreaElement) {
        let value = textarea.value();
        self.set_text(&value);
    }

    /// 标记正在更新
    #[allow(dead_code)]
    pub fn set_updating(&self, value: bool) {
        *self.updating.borrow_mut() = value;
    }

    /// 检查是否正在更新
    #[allow(dead_code)]
    pub fn is_updating(&self) -> bool {
        *self.updating.borrow()
    }
}

/// 辅助函数：将字节数组转换为十六进制字符串
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// 辅助函数：将十六进制字符串转换为字节数组
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    hex::decode(hex).map_err(|e| format!("解码十六进制失败: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crdt_manager_basic() {
        let mut manager = CrdtManager::new();

        // 设置文本
        manager.set_text("Hello, World!");
        assert_eq!(manager.get_text(), "Hello, World!");

        // 获取状态
        let state = manager.get_state();
        assert!(!state.is_empty());
    }

    #[test]
    fn test_crdt_manager_update() {
        let mut manager1 = CrdtManager::new();
        let mut manager2 = CrdtManager::new();

        // manager1 设置文本
        manager1.set_text("Hello");

        // 获取更新并应用到 manager2
        let state = manager1.get_state();
        manager2.apply_update(&state).unwrap();

        assert_eq!(manager2.get_text(), "Hello");
    }

    #[test]
    fn test_hex_encoding() {
        let bytes = vec![0x01, 0x02, 0x03, 0xFF];
        let hex = bytes_to_hex(&bytes);
        assert_eq!(hex, "010203ff");

        let decoded = hex_to_bytes(&hex).unwrap();
        assert_eq!(decoded, bytes);
    }
}
