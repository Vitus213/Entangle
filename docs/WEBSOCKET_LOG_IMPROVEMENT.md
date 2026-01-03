# WebSocket 日志改进说明

> 更新时间: 2026-01-03

## 📝 改进内容

### 原日志格式
```
User 390e3069-4ea6-4170-9c4b-7b6422cf1245 connecting to document 2f93238c-5071-4862-ac2c-15d941a8c80c (owner: f333b9fd-3d85-4105-9186-f121cd81e53a)
```

**问题**: 全是 UUID，不易读，难以快速识别用户和文档。

### 新日志格式
```
User 390e3069-4ea6-4170-9c4b-7b6422cf1245 (张三) connecting to document "项目计划书" (2f93238c-5071-4862-ac2c-15d941a8c80c) (owner: f333b9fd-3d85-4105-9186-f121cd81e53a (李四))
```

**优势**:
- ✅ 显示用户昵称（张三）
- ✅ 显示文档标题（项目计划书）
- ✅ 显示文档所有者昵称（李四）
- ✅ 保留 UUID 用于调试

## 🔍 日志示例

### 连接建立
```
INFO User 390e3069-4ea6-4170-9c4b-7b6422cf1245 (张三) connecting to document "项目计划书" (2f93238c-5071-4862-ac2c-15d941a8c80c) (owner: f333b9fd-3d85-4105-9186-f121cd81e53a (李四))
```

### 用户加入房间
```
INFO User 390e3069-4ea6-4170-9c4b-7b6422cf1245 (张三) joined document 2f93238c-5071-4862-ac2c-15d941a8c80c
```

### 用户离开房间
```
INFO User 390e3069-4ea6-4170-9c4b-7b6422cf1245 (张三) left document 2f93238c-5071-4862-ac2c-15d941a8c80c
```

### CRDT 状态加载
```
INFO Loaded CRDT state for document "项目计划书" (2f93238c-5071-4862-ac2c-15d941a8c80c)
```

### CRDT 状态手动保存
```
INFO Saved CRDT state for document "项目计划书" (2f93238c-5071-4862-ac2c-15d941a8c80c)
```

### CRDT 状态自动保存
```
DEBUG Auto-saved CRDT state for document "项目计划书" (2f93238c-5071-4862-ac2c-15d941a8c80c)
```

### 关闭时保存
```
INFO Saved CRDT state for document "项目计划书" (2f93238c-5071-4862-ac2c-15d941a8c80c) on shutdown
```

### 保存失败
```
ERROR Failed to save CRDT state for document "项目计划书" (2f93238c-5071-4862-ac2c-15d941a8c80c): database error
```

### 房间关闭时保存失败
```
WARN Failed to save document "项目计划书" (2f93238c-5071-4862-ac2c-15d941a8c80c) on room close: database error
```

## 🛠️ 实现细节

### 修改的文件
- `crates/api/src/ws/handlers.rs`
- `crates/api/src/ws/mod.rs`

### 关键改动

#### 1. 连接建立时查询用户和所有者昵称 (handlers.rs 第 44-74 行)
```rust
// 获取用户昵称（用于日志）
let user_nickname = sqlx::query_scalar::<_, String>(
    "SELECT nickname FROM users WHERE id = $1"
)
.bind(user.user_id)
.fetch_optional(&pool)
.await
.ok()
.flatten()
.unwrap_or_else(|| user.user_id.to_string());

// 获取文档所有者昵称（用于日志）
let owner_nickname = sqlx::query_scalar::<_, String>(
    "SELECT nickname FROM users WHERE id = $1"
)
.bind(doc.owner_id)
.fetch_optional(&pool)
.await
.ok()
.flatten()
.unwrap_or_else(|| doc.owner_id.to_string());

tracing::info!(
    "User {} ({}) connecting to document \"{}\" ({}) (owner: {} ({}))",
    user.user_id,
    user_nickname,
    doc.title,
    doc_id,
    doc.owner_id,
    owner_nickname
);
```

#### 2. 用户加入时查询昵称 (第 96-109 行)
```rust
// 获取用户名（用于日志和通知）
let user_nickname = if let Some(pool) = hub.pool() {
    sqlx::query_scalar::<_, String>("SELECT nickname FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
} else {
    None
};

let display_name = user_nickname.as_deref().unwrap_or("Unknown");
tracing::info!("User {} ({}) joined document {}", user_id, display_name, doc_id);
```

#### 3. 用户离开时使用缓存的昵称 (handlers.rs 第 256 行)
```rust
tracing::info!("User {} ({}) left document {}", user_id, display_name, doc_id);
```

#### 4. 文档保存失败警告显示文档名称 (handlers.rs 第 237-251 行)
```rust
// 获取文档标题用于日志
let doc_title = if let Some(pool) = hub.pool() {
    sqlx::query_scalar::<_, String>(
        "SELECT title FROM documents WHERE id = $1"
    )
    .bind(doc_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
} else {
    None
};
let doc_name = doc_title.as_deref().unwrap_or_else(|| "Unknown");
tracing::warn!("Failed to save document \"{}\" ({}) on room close: {}", doc_name, doc_id, e);
```

#### 5. CRDT 状态保存成功日志显示文档名称 (mod.rs 第 110-121 行)
```rust
// 获取文档标题用于日志
let doc_title = sqlx::query_scalar::<_, String>(
    "SELECT title FROM documents WHERE id = $1"
)
.bind(doc_id)
.fetch_optional(pool.as_ref())
.await
.ok()
.flatten()
.unwrap_or_else(|| doc_id.to_string());

tracing::info!("Saved CRDT state for document \"{}\" ({})", doc_title, doc_id);
```

#### 6. CRDT 状态加载日志显示文档名称 (mod.rs 第 137-148 行)
```rust
// 获取文档标题用于日志
let doc_title = sqlx::query_scalar::<_, String>(
    "SELECT title FROM documents WHERE id = $1"
)
.bind(doc_id)
.fetch_optional(pool.as_ref())
.await
.ok()
.flatten()
.unwrap_or_else(|| doc_id.to_string());

tracing::info!("Loaded CRDT state for document \"{}\" ({})", doc_title, doc_id);
```

#### 7. 自动保存 CRDT 状态日志显示文档名称 (mod.rs 第 196-209 行)
```rust
// 获取文档标题用于日志
let doc_title = sqlx::query_scalar::<_, String>(
    "SELECT title FROM documents WHERE id = $1"
)
.bind(doc_id)
.fetch_optional(pool.as_ref())
.await
.ok()
.flatten()
.unwrap_or_else(|| doc_id.to_string());

tracing::debug!("Auto-saved CRDT state for document \"{}\" ({})", doc_title, doc_id);
```

#### 8. 关闭时保存和错误日志显示文档名称 (mod.rs 第 236-264 行)
```rust
// 保存成功
let doc_title = sqlx::query_scalar::<_, String>(
    "SELECT title FROM documents WHERE id = $1"
)
.bind(doc_id)
.fetch_optional(pool.as_ref())
.await
.ok()
.flatten()
.unwrap_or_else(|| doc_id.to_string());

tracing::info!("Saved CRDT state for document \"{}\" ({}) on shutdown", doc_title, doc_id);

// 或保存失败
tracing::error!("Failed to save CRDT state for document \"{}\" ({}): {}", doc_title, doc_id, e);
```

## 📊 性能影响

### 额外的数据库查询
- **连接建立时**: 2 次额外查询（用户昵称 + 所有者昵称）
- **用户加入时**: 1 次额外查询（用户昵称）
- **手动保存时**: 1 次额外查询（文档标题）
- **自动保存时**: 1 次额外查询（文档标题，仅在有更新时）
- **关闭时保存**: 1 次额外查询（文档标题，每个脏文档）
- **保存失败时**: 1 次额外查询（文档标题，仅在错误时）

总计：每次连接约 3 次额外查询，每次保存约 1 次额外查询。

### 优化建议
1. **缓存用户昵称**: 在 Redis 中缓存用户信息，减少数据库查询
2. **批量查询**: 如果有多个用户同时加入，可以批量查询昵称
3. **从 JWT 获取**: 在 AuthUser 中包含用户昵称，避免额外查询
4. **文档元数据缓存**: 在 Room 结构中缓存文档标题，避免重复查询

## 🎯 后续优化

### 优先级 1: 从 JWT 中获取昵称
修改 `AuthUser` 结构，使其包含用户昵称：

```rust
pub struct AuthUser {
    pub user_id: Uuid,
    pub nickname: String,  // 添加这个字段
    pub claims: Claims,
}
```

这样在连接建立时就不需要额外查询用户昵称了。

### 优先级 2: 使用 Redis 缓存
缓存用户信息到 Redis：

```rust
// 伪代码
let user_nickname = redis
    .get(format!("user:{}:nickname", user_id))
    .await
    .or_else(|| {
        // 从数据库查询并缓存
        let nickname = query_from_db(user_id).await;
        redis.set(format!("user:{}:nickname", user_id), &nickname).await;
        nickname
    });
```

### 优先级 3: 结构化日志
使用结构化日志格式，便于日志分析：

```rust
tracing::info!(
    user_id = %user.user_id,
    user_nickname = %user_nickname,
    doc_id = %doc_id,
    doc_title = %doc.title,
    owner_id = %doc.owner_id,
    owner_nickname = %owner_nickname,
    "User connecting to document"
);
```

## 🧪 测试

### 启动服务器并观察日志
```bash
cd /home/vitus/Documents/Entangle
RUST_LOG=info cargo run --release
```

### 连接 WebSocket
在浏览器中打开一个文档，观察后端日志输出。

### 预期日志输出
```
2026-01-03T10:30:00.123Z  INFO entangle_api::ws::handlers: User 390e3069-4ea6-4170-9c4b-7b6422cf1245 (张三) connecting to document "项目计划书" (2f93238c-5071-4862-ac2c-15d941a8c80c) (owner: f333b9fd-3d85-4105-9186-f121cd81e53a (李四))
2026-01-03T10:30:00.156Z  INFO entangle_api::ws: Loaded CRDT state for document "项目计划书" (2f93238c-5071-4862-ac2c-15d941a8c80c)
2026-01-03T10:30:00.234Z  INFO entangle_api::ws::handlers: User 390e3069-4ea6-4170-9c4b-7b6422cf1245 (张三) joined document 2f93238c-5071-4862-ac2c-15d941a8c80c
2026-01-03T10:30:30.456Z DEBUG entangle_api::ws: Auto-saved CRDT state for document "项目计划书" (2f93238c-5071-4862-ac2c-15d941a8c80c)
```

---

*文档版本: 1.0.0*
*作者: Claude Code*
*最后更新: 2026-01-03*
