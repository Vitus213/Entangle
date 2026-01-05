# Entangle 答辩指南

> 本文档帮助你快速理解项目代码，准备答辩演示

---

## 目录

1. [项目一句话介绍](#1-项目一句话介绍)
2. [系统架构图](#2-系统架构图)
3. [核心功能演示流程](#3-核心功能演示流程)
4. [代码导读](#4-代码导读)
5. [技术要点解释](#5-技术要点解释)
6. [答辩常见问题](#6-答辩常见问题)

---

## 1. 项目一句话介绍

**Entangle 是一个基于 Rust + openGauss 的实时协作文档编辑系统**，类似 Google Docs，支持多用户同时在线编辑同一文档，通过 CRDT（无冲突复制数据类型）技术实现实时同步。

---

## 2. 系统架构图

### 2.1 整体架构

```
┌─────────────────────────────────────────────────────────────────┐
│                         浏览器 (前端)                            │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  Leptos (Rust/WASM)                                     │   │
│  │  ├── LoginPage      (登录页面)                           │   │
│  │  ├── DocumentsPage  (文档列表)                           │   │
│  │  └── EditorPage     (编辑器 + WebSocket)                 │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                            ↕ HTTP + WebSocket
┌─────────────────────────────────────────────────────────────────┐
│                       后端服务器 (Axum)                          │
│  ┌───────────────┐  ┌───────────────┐  ┌─────────────────────┐ │
│  │  路由层       │  │  中间件       │  │  WebSocket 处理器   │ │
│  │  documents.rs │  │  auth.rs      │  │  handlers.rs        │ │
│  └───────────────┘  └───────────────┘  └─────────────────────┘ │
│  ┌───────────────┐  ┌───────────────┐  ┌─────────────────────┐ │
│  │  业务逻辑层   │  │  数据访问层   │  │  CRDT 协作层        │ │
│  │  permissions  │  │  repository   │  │  sync.rs            │ │
│  └───────────────┘  └───────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                            ↕ SQL
┌─────────────────────────────────────────────────────────────────┐
│                    openGauss 数据库                              │
│  users  │  documents  │  document_collaborators  │  comments    │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 目录结构

```
Entangle/
├── crates/
│   ├── api/              # 后端 API 服务器
│   │   └── src/
│   │       ├── main.rs           # 服务器入口
│   │       ├── routes/           # RESTful 路由
│   │       ├── middleware/       # JWT 认证中间件
│   │       └── ws/               # WebSocket 处理
│   ├── auth/             # 认证模块 (JWT + 密码加密)
│   ├── collab/           # 协作模块 (CRDT + 房间管理)
│   │   └── src/
│   │       ├── document.rs       # Yrs CRDT 文档封装
│   │       ├── sync.rs           # 房间和广播管理
│   │       └── awareness.rs      # 用户光标感知
│   ├── db/               # 数据库层
│   │   └── src/
│   │       ├── models/           # 数据模型定义
│   │       └── repository/       # 数据访问 (CRUD)
│   └── core/             # 核心业务逻辑
│       └── services/
│           └── document_permission.rs  # 权限检查
│
├── frontend/            # 前端 WASM 应用
│   └── src/
│       ├── lib.rs               # 主文件 (所有页面组件)
│       └── crdt.rs              # CRDT 管理器
│
└── migrations/          # 数据库迁移文件
```

---

## 3. 核心功能演示流程

### 演示步骤建议

1. **登录功能** (30秒)
   - 打开浏览器 → 输入邮箱密码 → 登录

2. **文档列表** (30秒)
   - 显示已有文档
   - 点击"新建"创建文档

3. **实时协作** (重点！2分钟)
   - 打开两个浏览器窗口
   - 登录不同账号
   - 同时编辑同一文档
   - 展示：A 窗口输入文字 → B 窗口实时显示

4. **协作者管理** (1分钟)
   - 添加协作者
   - 设置权限（只读/编辑/管理）

5. **评论和任务** (1分钟)
   - 添加评论
   - 创建任务

### 突出显示的界面元素

- 顶部状态栏：显示在线用户和同步状态
- 编辑器：实时编辑
- 右侧面板：协作者、评论、任务切换

---

## 4. 代码导读

### 4.1 按功能模块阅读

#### 模块1: 用户认证 (最简单，从这里开始)

```
crates/auth/src/
├── lib.rs          # JWT 生成和验证函数
├── password.rs     # Argon2 密码哈希
└── permission.rs   # 权限枚举定义

crates/api/src/middleware/auth.rs
                  # 认证中间件 - 从请求中提取 token
```

**关键函数** (crates/auth/src/lib.rs):
```rust
// 生成 JWT token
pub fn create_token(user_id: Uuid) -> Result<String, AuthError>

// 验证 JWT token
pub fn verify_token(token: &str) -> Result<Claims, AuthError>
```

---

#### 模块2: 文档 CRUD (理解基本业务逻辑)

```
crates/api/src/routes/documents.rs    # 文档路由处理器
crates/db/src/repository/document.rs  # 数据库操作
crates/db/src/models/document.rs      # 数据模型定义
```

**创建文档流程** (crates/api/src/routes/documents.rs):
```rust
async fn create_document(
    State(pool),           // 数据库连接池
    user: AuthUser,        // 当前登录用户
    Json(req)             // { title, content }
) -> Result<Json<Document>, AppError> {
    // 1. 调用 repository 创建
    let doc = DocumentRepository::create(&pool, user.user_id, req).await?;

    // 2. 返回结果
    Ok(Json(doc))
}
```

---

#### 模块3: 实时协作 (最复杂，最核心)

```
crates/collab/src/
├── document.rs      # CRDT 文档封装
├── sync.rs          # 房间管理和广播
└── awareness.rs     # 用户感知状态

crates/api/src/ws/
├── mod.rs           # WsHub (中心管理器)
└── handlers.rs      # WebSocket 连接处理
```

**WebSocket 处理流程** (crates/api/src/ws/handlers.rs):

```rust
async fn websocket_handler(...) {
    // 1. 验证文档存在和用户权限
    let doc = DocumentRepository::find_by_id(&pool, doc_id).await?;
    if !DocumentPermissionService::can_read(&pool, user_id, doc_id).await? {
        return Err(AppError::Forbidden("无权访问".into()));
    }

    // 2. 升级为 WebSocket 连接
    Ok(ws.on_upgrade(|socket| handle_socket(socket, doc_id, user_id, hub)))
}

async fn handle_socket(socket, doc_id, user_id, hub) {
    let (mut sender, mut receiver) = socket.split();

    // 3. 获取或创建文档房间
    let room = hub.room_manager().get_or_create_room(doc_id);
    let mut broadcast_rx = room.subscribe(); // 订阅广播

    // 4. 发送当前文档内容给新用户
    let text_content = room.get_text_content();
    sender.send(Message::Text(text_content)).await;

    // 5. 主循环：同时监听三种事件
    loop {
        tokio::select! {
            // 客户端消息 → 更新文档 → 广播
            msg = receiver.next() => { /* ... */ }

            // 广播消息 → 转发给客户端
            broadcast = broadcast_rx.recv() => { /* ... */ }

            // 定时心跳
            _ = heartbeat_interval.tick() => { /* ... */ }
        }
    }
}
```

---

#### 模块4: 前端组件

```
frontend/src/lib.rs        # 所有页面组件 (约3500行)

主要组件：
├── App()                 # 路由配置
├── LoginPage()           # 登录页面
├── DocumentsPage()       # 文档列表 + 搜索 + 通知
└── EditorPage()          # 编辑器 + WebSocket + CRDT
```

**编辑器关键代码** (frontend/src/lib.rs 第2180-3477行):

```rust
#[component]
fn EditorPage() -> impl IntoView {
    // 1. 状态管理
    let (content, set_content) = create_signal(String::new());
    let (online_users, set_online_users) = create_signal(Vec::new());
    let (ws, set_ws) = create_signal(None::<WebSocket>);

    // 2. 加载文档并建立 WebSocket
    create_effect(move |_| {
        spawn_local(async move {
            // 获取文档
            let doc = fetch_document(&id).await?;

            // 建立 WebSocket
            let ws_url = format!("{}/ws/documents/{}?token={}", WS_BASE, id, token);
            let websocket = WebSocket::new(&ws_url)?;

            // 设置消息处理
            websocket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        });
    });

    // 3. 防抖处理输入
    let on_content_change = move |ev| {
        let new_content = event_target_value(&ev);

        // 清除旧定时器
        window.clear_timeout_with_handle(timer_id);

        // 设置 500ms 延迟
        let timer_id = window.set_timeout_with_callback(callback, 500);
    };
}
```

---

### 4.2 快速定位代码

| 功能 | 文件位置 | 行号 |
|------|----------|------|
| 服务器入口 | `crates/api/src/main.rs` | 全文 |
| WebSocket 处理 | `crates/api/src/ws/handlers.rs` | 23-257 |
| 房间管理 | `crates/collab/src/sync.rs` | 37-196 |
| CRDT 文档 | `crates/collab/src/document.rs` | 24-139 |
| JWT 验证 | `crates/api/src/middleware/auth.rs` | 全文 |
| 权限检查 | `crates/core/src/services/document_permission.rs` | 全文 |
| 前端登录页 | `frontend/src/lib.rs` | 1183-1252 |
| 前端编辑器 | `frontend/src/lib.rs` | 2180-3477 |
| 前端 CRDT | `frontend/src/crdt.rs` | 全文 |

---

## 5. 技术要点解释

### 5.1 为什么选择 Rust？

1. **类型安全**：编译时捕获错误，减少运行时 bug
2. **高性能**：接近 C/C++ 性能，适合高并发 WebSocket
3. **内存安全**：无垃圾回收，无数据竞争
4. **全栈统一**：前后端同一语言，共享数据模型

### 5.2 什么是 CRDT？

**CRDT = Conflict-free Replicated Data Type (无冲突复制数据类型)**

**原理**：
- 不存储最终状态，而是存储操作历史
- 每个操作有唯一 ID（用户ID + 时间戳）
- 所有用户根据确定性规则合并，得到相同结果

**示例**：
```
用户 A 和 B 同时编辑，都在位置 0 插入字符：
  A 插入 "H"，ID: (userA, 100)
  B 插入 "W"，ID: (userB, 101)

最终排序：按 ID 大小 → "HW" (或 "WH"，取决于 ID 规则)
```

**Yrs 库**：我们使用的 Rust CRDT 实现，兼容 JavaScript 的 Yjs

### 5.3 WebSocket vs HTTP

| 特性 | HTTP | WebSocket |
|------|------|-----------|
| 连接 | 短连接 (请求-响应) | 长连接 (保持打开) |
| 方向 | 客户端 → 服务器 | 双向 |
| 延迟 | 每次请求有开销 | 建立后延迟低 |
| 用途 | API 调用 | 实时聊天、协作 |

### 5.4 防抖 (Debounce)

**问题**：用户每输入一个字符就发送请求，会浪费资源

**解决**：等待用户停止输入 500ms 后才发送

```rust
// 用户输入 "hello"
h    → 启动 500ms 定时器
he   → 重置定时器
hel  → 重置定时器
hell → 重置定时器
hello→ 重置定时器
(500ms 无新输入) → 发送请求
```

### 5.5 权限系统

```
权限层级：
  所有者 (owner)    → 完全控制，可删除文档
  ↓
  Admin 协作者      → 可管理其他协作者
  ↓
  Write 协作者      → 可编辑内容
  ↓
  Read 协作者       → 只读
```

---

## 6. 答辩常见问题

### Q1: 这个项目的创新点是什么？

**回答要点**：
1. **全栈 Rust**：前后端统一语言，类型安全
2. **CRDT 实时同步**：支持离线编辑，自动冲突解决
3. **轻量高效**：WASM 前端性能接近原生

### Q2: CRDT 是如何解决冲突的？

**回答要点**：
1. 每个操作带唯一 ID（用户ID + Lamport时间戳）
2. 根据ID确定性排序，保证最终一致
3. 使用 Yrs 库实现，兼容 Yjs 生态

### Q3: WebSocket 连接断开怎么办？

**回答要点**：
1. 心跳检测：30秒发送 ping，10秒超时判定断开
2. 自动保存：最后用户离开时保存文档到数据库
3. 前端可实现自动重连（预留）

### Q4: 如何保证并发安全？

**回答要点**：
1. **Rust 所有权系统**：编译时保证内存安全
2. **Arc<RwLock<T>>**：多线程安全共享数据
   ```rust
   connections: Arc<RwLock<HashMap<Uuid, ConnectionInfo>>>
   ```
3. **数据库事务**：SQL 操作使用事务保证原子性

### Q5: 项目难点是什么？

**回答要点**：
1. **CRDT 状态序列化**：二进制状态的编码/解码
2. **WebSocket 生命周期管理**：连接、心跳、断开处理
3. **前端 Closure 内存管理**：wasm-bindgen 的 Closure 需要 forget()
4. **房间并发访问**：RwLock 避免死锁

### Q6: 可以扩展到多服务器吗？

**回答要点**：
**当前**：单服务器，房间在内存中

**扩展方案**：
1. **Redis Pub/Sub**：服务器间消息转发
2. **数据库 LISTEN/NOTIFY**：PostgreSQL 原生通知
3. **专用消息队列**：RabbitMQ/Kafka（大规模）

### Q7: 为什么用 openGauss？

**回答要点**：
1. **PostgreSQL 兼容**：完全兼容 PostgreSQL 生态
2. **企业级特性**：高可用、安全、性能
3. **国产数据库**：符合信创要求

---

## 演示检查清单

- [ ] 数据库已启动 (`docker-compose up -d`)
- [ ] 后端已运行 (`cargo run`)
- [ ] 前端已构建 (`trunk serve`)
- [ ] 准备两个测试账号
- [ ] 测试浏览器能访问 http://localhost:8080

---

## 关键文件速查

| 需要解释... | 打开这个文件 |
|-------------|--------------|
| 服务器启动流程 | `crates/api/src/main.rs` |
| WebSocket 连接 | `crates/api/src/ws/handlers.rs:79-257` |
| CRDT 文档 | `crates/collab/src/document.rs:24-139` |
| 房间广播 | `crates/collab/src/sync.rs:37-196` |
| 前端编辑器 | `frontend/src/lib.rs:2180-2700` |
| 权限检查 | `crates/core/src/services/document_permission.rs` |

---

祝答辩顺利！
