# Entangle 项目快速参考指南

> 演示和答辩时的速查手册

## 1. 项目一句话介绍

**Entangle 是一个基于 Rust 的实时协作文档编辑系统，支持多用户同时编辑、WebSocket 实时通信和 CRDT 冲突解决。**

## 2. 核心技术栈

### 后端
- **Axum**：Web 框架（类似 Express.js）
- **SQLx**：数据库操作（类型安全的 SQL）
- **Yrs**：CRDT 库（实时协作核心）
- **Tokio**：异步运行时（处理并发）

### 前端
- **Leptos**：响应式 UI 框架（类似 React）
- **WASM**：WebAssembly（高性能）
- **WebSocket**：实时双向通信

### 数据库
- **PostgreSQL/openGauss**：关系型数据库

## 3. 关键文件位置

| 功能 | 文件路径 | 行数参考 |
|------|----------|---------|
| **服务器入口** | `crates/api/src/main.rs` | 全文 60 行 |
| **认证中间件** | `crates/api/src/middleware/auth.rs` | 第 24-60 行 |
| **文档路由** | `crates/api/src/routes/documents.rs` | 第 100-150 行（协作者）|
| **WebSocket 处理** | `crates/api/src/ws/handlers.rs` | 第 54-205 行（核心）|
| **CRDT 文档** | `crates/collab/src/document.rs` | 第 24-139 行 |
| **房间管理** | `crates/collab/src/sync.rs` | 第 32-183 行 |
| **前端主应用** | `frontend/src/lib.rs` | 第 1-50 行（App）|
| **编辑器页面** | `frontend/src/lib.rs` | 第 900-1200 行 |

## 4. 数据库表关系

```
users (用户)
  ↓ owner_id
documents (文档)
  ↓ document_id
document_collaborators (协作者)
  ↓ user_id
users (用户)
```

**核心字段：**
- `documents.crdt_state`：CRDT 二进制状态（BYTEA）
- `document_collaborators.permission`：权限（read/write/admin）

## 5. 实时同步流程（重点）

```
用户 A 编辑
  ↓ (500ms 防抖)
WebSocket 发送 { type: "sync", update: "内容" }
  ↓
后端接收 → handle_client_message()
  ↓
room.set_text_content() → 更新 CRDT 文档
  ↓
broadcast_tx.send(TextUpdate) → 广播
  ↓
用户 B、C、D 的 WebSocket 接收
  ↓
前端更新编辑器内容
```

**关键代码位置：**
- 前端防抖：`frontend/src/lib.rs` 第 1150 行
- 后端处理：`crates/api/src/ws/handlers.rs` 第 207 行
- 广播机制：`crates/collab/src/sync.rs` 第 157 行

## 6. 常见问题快速回答

### Q: 为什么选择 Rust？
**A:** 1) 类型安全减少 bug  2) 高性能适合实时应用  3) 前后端统一  4) 支持 WASM

### Q: CRDT 解决什么问题？
**A:** 多用户同时编辑时的冲突合并，保证最终一致性，支持离线编辑

### Q: WebSocket 优势？
**A:** 双向实时通信，服务器可主动推送，延迟低，适合实时编辑

### Q: 防抖作用？
**A:** 延迟执行，避免频繁网络请求。用户输入"hello"只发送1次而不是5次

### Q: 如何保证安全？
**A:** JWT 认证 + 权限检查（can_read/can_write/can_manage）+ SQL 防注入

### Q: 权限层级？
**A:** 所有者 > admin 协作者 > write 协作者 > read 协作者

## 7. 演示流程建议

### 准备
```bash
# 1. 启动数据库（如果未启动）
# 2. 启动后端
cargo run -p entangle-api

# 3. 启动前端（新终端）
cd frontend && trunk serve
```

### 演示步骤
1. **注册登录**：展示用户系统
2. **创建文档**：展示文档管理
3. **添加协作者**：输入另一个用户邮箱
4. **实时编辑**：
   - 打开两个浏览器窗口
   - 不同用户登录
   - 同时编辑同一文档
   - 展示实时同步效果
5. **查看在线用户**：点击"👥 协作者"按钮

## 8. 关键代码讲解

### 代码片段 1：JWT 认证（认证中间件）

```rust
// 文件：crates/api/src/middleware/auth.rs:24-60
#[async_trait]
impl<S> FromRequestParts<S> for AuthUser {
    async fn from_request_parts(...) -> Result<Self, Self::Rejection> {
        // 1. 从 Header 或 Query 获取 token
        let token = if let Ok(TypedHeader(Authorization(bearer))) = ... {
            bearer.token().to_string()
        } else {
            // WebSocket 通过 query 传递
            parts.extract::<Query<TokenQuery>>().await?.token
        };

        // 2. 验证 JWT
        let claims = entangle_auth::verify_token(&token)?;

        // 3. 返回认证用户
        Ok(AuthUser { user_id: claims.sub, claims })
    }
}
```

**讲解要点：**
- Axum 的 `FromRequestParts` trait 自动注入认证信息
- 支持两种 token 传递方式（Header 和 Query）
- WebSocket 只能用 Query 参数

### 代码片段 2：WebSocket 消息处理

```rust
// 文件：crates/api/src/ws/handlers.rs:207-243
async fn handle_client_message(
    room: &DocumentRoom,
    user_id: Uuid,
    msg: WsMessage,
) -> bool {
    match msg {
        WsMessage::Sync { update } => {
            // 尝试 hex 解码（CRDT 模式）
            if let Ok(update_bytes) = hex_decode(&update) {
                room.apply_update(&update_bytes, user_id).ok();
            } else {
                // 简化模式：纯文本同步
                room.set_text_content(&update, user_id).ok();
            }
            true  // 文档已修改
        }
        WsMessage::Awareness { state } => {
            // 更新用户光标位置等
            room.update_awareness(user_id, state);
            false
        }
        _ => false
    }
}
```

**讲解要点：**
- 两种同步模式：CRDT 二进制 或 纯文本
- 返回值表示是否需要保存文档
- Awareness 用于显示其他用户的光标

### 代码片段 3：前端防抖

```rust
// 文件：frontend/src/lib.rs:1150-1170
let on_content_change = move |ev| {
    let new_content = event_target_value(&ev);
    set_content.set(new_content.clone());

    // 清除旧定时器
    if let Some(timer_id) = debounce_timer.get() {
        window().unwrap().clear_timeout_with_handle(timer_id);
    }

    // 设置 500ms 延迟
    let callback = Closure::wrap(Box::new(move || {
        // 发送 WebSocket 消息
        if let Some(ws) = ws.get() {
            let msg = WsMessage::Sync { update: new_content };
            ws.send_with_str(&serde_json::to_string(&msg).unwrap()).ok();
        }
    }) as Box<dyn Fn()>);

    let timer_id = window().unwrap()
        .set_timeout_with_callback_and_timeout(..., 500).unwrap();

    debounce_timer.set(Some(timer_id));
    callback.forget();
};
```

**讲解要点：**
- 每次输入都重置 500ms 定时器
- 只有停止输入 500ms 后才发送
- `Closure::forget()` 防止内存被释放

### 代码片段 4：房间广播

```rust
// 文件：crates/collab/src/sync.rs:157-169
pub fn set_text_content(&self, content: &str, from_user: Uuid)
    -> Result<(), CollabError>
{
    // 1. 更新 CRDT 文档
    self.document.set_text("content", content)?;

    // 2. 广播给其他用户
    self.broadcast_tx.send(BroadcastMessage::TextUpdate {
        from_user,
        content: content.to_string(),
    }).ok();

    Ok(())
}
```

**讲解要点：**
- `broadcast_tx` 是 Tokio 的多生产者多消费者通道
- 所有订阅者（WebSocket 连接）都会收到消息
- `from_user` 用于跳过发送者本人

## 9. 项目统计数据

- **总代码行数**：约 3000 行（不含注释）
- **后端文件**：15 个
- **前端文件**：1 个（单文件应用）
- **数据库表**：6 个（users, documents, folders, tags, document_collaborators, document_tags）
- **API 端点**：约 20 个

## 10. 技术难点与解决

| 难点 | 解决方案 |
|------|----------|
| **WebSocket 认证** | 支持 Query 参数传递 token |
| **并发控制** | Arc<RwLock> + 事务 |
| **前端状态管理** | Leptos Signals（响应式）|
| **防止频繁请求** | 500ms 防抖 |
| **CRDT 持久化** | 序列化为 BYTEA 存数据库 |
| **跨域问题** | CORS 配置允许所有来源（开发环境）|

## 11. 可能的改进（如果老师问）

1. **性能优化**
   - 增量同步（只发送差异）
   - 分页加载文档列表
   - 压缩 WebSocket 消息

2. **功能扩展**
   - 评论和批注
   - 版本历史和回滚
   - Markdown 渲染
   - 富文本编辑器

3. **运维支持**
   - Docker 部署
   - 监控和日志
   - 自动备份

4. **多服务器扩展**
   - Redis Pub/Sub
   - 负载均衡
   - Session 持久化

## 12. 紧急问题应对

### 如果演示时 WebSocket 连接失败
**检查：**
1. 后端是否在 3000 端口运行？（`ss -tlnp | grep 3000`）
2. 前端是否在 8080 端口运行？
3. 浏览器控制台是否有错误？
4. Token 是否有效？（查看 localStorage）

### 如果实时同步不工作
**检查：**
1. 两个用户是否打开了同一个文档？
2. WebSocket 是否连接成功？（浏览器控制台）
3. 后端日志是否有 "Applied text update" 消息？
4. 是否等待了 500ms（防抖时间）？

### 如果数据库连接失败
**检查：**
1. 数据库是否启动？
2. DATABASE_URL 环境变量是否正确？
3. 密码是否正确？（Entangle@2024）

## 13. 快速命令

```bash
# 查看后端日志
cargo run -p entangle-api

# 查看数据库数据
PGPASSWORD='Entangle@2024' psql -h localhost -U entangle -d postgres -c "SELECT * FROM entangle.users;"

# 查看所有文档
PGPASSWORD='Entangle@2024' psql -h localhost -U entangle -d postgres -c "SELECT d.id, d.title, u.nickname FROM entangle.documents d JOIN entangle.users u ON d.owner_id = u.id;"

# 查看协作者
PGPASSWORD='Entangle@2024' psql -h localhost -U entangle -d postgres -c "SELECT * FROM entangle.document_collaborators;"

# 重启后端（如果卡死）
pkill -9 entangle-api
cargo run -p entangle-api

# 重新编译前端
cd frontend && trunk build
```

## 14. 最后检查清单

演示前确认：
- [ ] 数据库已启动
- [ ] 后端服务器运行在 3000 端口
- [ ] 前端运行在 8080 端口
- [ ] 至少有 2 个测试用户
- [ ] 至少有 1 个测试文档
- [ ] 浏览器已打开（Chrome/Firefox）
- [ ] 网络连接正常
- [ ] 文档已阅读一遍

---

**祝演示成功！遇到问题保持冷静，参考本文档快速定位。**
