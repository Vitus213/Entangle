# Entangle 协作文档系统 - 完整技术文档

> 本文档提供了 Entangle 项目的完整技术架构说明，从整体设计到具体实现细节的全面讲解。

## 目录

1. [项目概述](#项目概述)
2. [整体架构](#整体架构)
3. [技术栈详解](#技术栈详解)
4. [后端架构深度解析](#后端架构深度解析)
5. [前端架构深度解析](#前端架构深度解析)
6. [实时协作系统详解](#实时协作系统详解)
7. [数据库设计](#数据库设计)
8. [关键业务流程](#关键业务流程)
9. [代码导读](#代码导读)

---

## 项目概述

### 项目定位
Entangle 是一个现代化的**实时协作文档编辑系统**，类似 Google Docs，支持：
- 多用户实时编辑同一文档
- 协作者权限管理
- WebSocket 实时通信
- CRDT（无冲突复制数据类型）文本同步

### 核心特性
1. **用户系统**：注册、登录、JWT 认证
2. **文档管理**：CRUD 操作、文件夹组织、标签分类
3. **协作功能**：添加协作者、权限控制、实时在线状态
4. **实时编辑**：WebSocket 双向通信、自动同步、防抖优化
5. **权限系统**：文档所有者、协作者（只读/编辑/管理员）

### 技术亮点
- **全栈 Rust**：前后端均使用 Rust 编写
- **WASM 前端**：Leptos 框架 + WebAssembly
- **类型安全**：编译时类型检查，减少运行时错误
- **异步优先**：Tokio 异步运行时，高并发处理
- **CRDT 同步**：基于 Yjs CRDT 库，支持离线编辑和冲突解决

---

## 整体架构

### 系统架构图

```
┌─────────────────────────────────────────────────────────────┐
│                         浏览器客户端                          │
│  ┌──────────────────────────────────────────────────────┐   │
│  │          Leptos (Rust WASM) 前端应用                  │   │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────────┐  │   │
│  │  │  组件层     │  │  状态管理   │  │  WebSocket     │  │   │
│  │  │  (Pages)   │  │  (Signals) │  │  客户端        │  │   │
│  │  └────────────┘  └────────────┘  └────────────────┘  │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                           │ HTTP/WS
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                      后端服务器 (Rust)                        │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              Axum Web 框架                            │   │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────────┐  │   │
│  │  │  路由层     │  │  中间件层   │  │  WebSocket     │  │   │
│  │  │  (Routes)  │  │  (Auth)    │  │  处理器        │  │   │
│  │  └────────────┘  └────────────┘  └────────────────┘  │   │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────────┐  │   │
│  │  │  业务逻辑层 │  │  数据访问层 │  │  CRDT 协作层   │  │   │
│  │  │  (Core)    │  │  (DB)      │  │  (Collab)      │  │   │
│  │  └────────────┘  └────────────┘  └────────────────┘  │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                           │ SQL
                           ▼
┌─────────────────────────────────────────────────────────────┐
│              PostgreSQL/openGauss 数据库                      │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────────────┐  │
│  │  users   │  │documents │  │ document_collaborators   │  │
│  └──────────┘  └──────────┘  └──────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### 项目目录结构

```
Entangle/
├── crates/                    # 后端 Rust crates
│   ├── api/                   # API 服务器主程序
│   │   ├── src/
│   │   │   ├── main.rs        # 服务器入口
│   │   │   ├── routes/        # RESTful API 路由
│   │   │   ├── middleware/    # 认证等中间件
│   │   │   └── ws/            # WebSocket 处理
│   │   └── Cargo.toml
│   ├── auth/                  # 认证模块
│   │   ├── src/lib.rs         # JWT 生成和验证
│   │   └── Cargo.toml
│   ├── collab/                # 实时协作模块
│   │   ├── src/
│   │   │   ├── document.rs    # CRDT 文档
│   │   │   ├── sync.rs        # 房间管理和同步
│   │   │   └── awareness.rs   # 用户感知状态
│   │   └── Cargo.toml
│   ├── core/                  # 核心业务逻辑
│   │   ├── src/
│   │   │   ├── permissions.rs # 权限检查
│   │   │   └── errors.rs      # 错误类型
│   │   └── Cargo.toml
│   └── db/                    # 数据库层
│       ├── src/
│       │   ├── models/        # 数据模型
│       │   └── repository/    # 数据访问接口
│       └── Cargo.toml
├── frontend/                  # 前端 Leptos 应用
│   ├── src/
│   │   └── lib.rs             # 主应用文件（包含所有页面组件）
│   ├── Cargo.toml
│   └── index.html
├── docs/                      # 文档目录
│   └── ARCHITECTURE.md        # 本文档
└── Cargo.toml                 # Workspace 配置
```

---

## 技术栈详解

### 后端技术栈

#### 1. **Axum Web 框架**
```rust
// crates/api/src/main.rs
use axum::{routing::get, Router};
```

**为什么选择 Axum？**
- **类型安全**：基于 Tower Service，编译时保证路由安全
- **高性能**：基于 Tokio 异步运行时
- **组合性好**：中间件、提取器设计优雅
- **WebSocket 原生支持**：无需额外库

**核心概念：**
- **Router**：定义路由和处理函数
- **Handler**：异步处理函数，自动提取参数
- **Extractor**：从请求中提取数据（如 JSON、Path 参数）
- **State**：在整个应用中共享的状态

#### 2. **SQLx 数据库**
```rust
use sqlx::{PgPool, query_as};
```

**特点：**
- **编译时检查 SQL**：通过宏在编译时验证 SQL 语句
- **异步优先**：所有操作都是异步的
- **类型安全**：自动映射数据库行到 Rust 结构体
- **连接池管理**：自动管理数据库连接

#### 3. **Yrs CRDT 库**
```rust
use yrs::{Doc, Text, Transact};
```

**CRDT 是什么？**
- **Conflict-free Replicated Data Type**（无冲突复制数据类型）
- 允许多个用户同时编辑，自动合并更改
- 不需要中央协调器，最终一致性保证

**Yrs 特点：**
- 兼容 Yjs（JavaScript CRDT 库）
- 高性能，用 Rust 实现
- 支持多种数据类型（Text、Map、Array）

#### 4. **Tokio 异步运行时**
```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ...
}
```

**作用：**
- 提供异步任务调度
- 管理并发 I/O 操作
- 支持 async/await 语法

### 前端技术栈

#### 1. **Leptos 框架**
```rust
use leptos::*;

#[component]
fn App() -> impl IntoView {
    view! { <div>"Hello"</div> }
}
```

**为什么选择 Leptos？**
- **响应式系统**：类似 React/Vue 的 Signal 状态管理
- **编译到 WASM**：高性能，接近原生速度
- **服务端渲染支持**：可选 SSR
- **无虚拟 DOM**：直接操作真实 DOM，性能更好

**核心概念：**
- **Component**：使用 `#[component]` 宏定义组件
- **Signal**：响应式状态，类似 React useState
- **Effect**：响应式副作用，类似 React useEffect
- **Resource**：异步数据加载

#### 2. **Web-sys + wasm-bindgen**
```rust
use web_sys::{WebSocket, window};
use wasm_bindgen::JsCast;
```

**作用：**
- **web-sys**：Rust 绑定 Web API（如 WebSocket、DOM）
- **wasm-bindgen**：在 Rust 和 JavaScript 之间传递数据

---

## 后端架构深度解析

### 1. 入口文件：main.rs

#### 文件位置：`crates/api/src/main.rs`

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1️⃣ 初始化日志系统
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "entangle_api=debug,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 2️⃣ 加载环境变量
    dotenvy::dotenv().ok();

    // 3️⃣ 获取数据库连接字符串
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    // 4️⃣ 创建数据库连接池
    tracing::info!("Connecting to database...");
    let pool = create_pool(&database_url).await?;
    tracing::info!("Database connected successfully");

    // 5️⃣ 创建 WebSocket Hub（实时协作核心）
    let ws_hub = WsHub::with_pool(pool.clone());
    tracing::info!("WebSocket hub initialized with auto-save enabled");

    // 6️⃣ 配置 CORS（跨域资源共享）
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 7️⃣ 构建路由
    let app = Router::new()
        // 健康检查
        .route("/health", get(|| async { "OK" }))

        // 认证路由
        .nest("/api/auth", routes::auth::router())

        // 文档路由（需要认证）
        .nest("/api/documents", routes::documents::router())

        // 标签路由（需要认证）
        .nest("/api/tags", routes::tags::router())

        // WebSocket 路由（需要认证）
        .nest("/ws", routes::ws::router())

        // 添加共享状态
        .layer(Extension(pool))
        .layer(Extension(ws_hub))

        // 添加 CORS
        .layer(cors);

    // 8️⃣ 启动服务器
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
```

**关键点解析：**

1. **日志系统**：使用 `tracing` 记录应用运行状态
2. **数据库连接池**：`PgPool` 管理多个数据库连接，提高性能
3. **WsHub**：管理所有 WebSocket 连接和文档房间
4. **Extension**：Axum 的状态共享机制，类似依赖注入
5. **nest**：路由模块化，每个功能一个子路由

### 2. 认证中间件

#### 文件位置：`crates/api/src/middleware/auth.rs`

```rust
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};

// 认证用户信息
#[derive(Clone, Debug)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub claims: Claims,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // 1️⃣ 从 Header 或 Query 参数获取 token
        let token = if let Ok(TypedHeader(Authorization(bearer))) =
            parts.extract::<TypedHeader<Authorization<Bearer>>>().await
        {
            bearer.token().to_string()
        } else {
            // 支持 WebSocket 通过 query 传递 token
            let query = parts.extract::<Query<TokenQuery>>().await
                .map_err(|_| (StatusCode::UNAUTHORIZED, "Missing token".to_string()))?;

            query.token.clone()
                .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Missing token".to_string()))?
        };

        // 2️⃣ 验证 JWT token
        let claims = entangle_auth::verify_token(&token)
            .map_err(|e| (StatusCode::UNAUTHORIZED, format!("Invalid token: {}", e)))?;

        // 3️⃣ 返回认证用户
        Ok(AuthUser {
            user_id: claims.sub,
            claims,
        })
    }
}
```

**工作流程：**
1. 请求进来 → 提取 token（Header 或 Query）
2. 验证 token → 解析 JWT Claims
3. 成功 → 注入 `AuthUser` 到 Handler
4. 失败 → 返回 401 Unauthorized

**为什么支持 Query 参数？**
- WebSocket 浏览器 API 不支持自定义 Header
- 所以需要通过 URL 传递 token：`/ws/documents/{id}?token=xxx`

### 3. 文档路由

#### 文件位置：`crates/api/src/routes/documents.rs`

```rust
pub fn router() -> Router {
    Router::new()
        // 文档 CRUD
        .route("/", post(create_document).get(list_documents))
        .route("/:id", get(get_document).put(update_document).delete(delete_document))

        // 协作者管理
        .route("/:id/collaborators",
            get(list_collaborators)
            .post(add_collaborator))
        .route("/:id/collaborators/:user_id",
            delete(remove_collaborator))

        // 获取可访问文档
        .route("/accessible", get(list_accessible_documents))
}
```

#### 添加协作者（核心功能）

```rust
async fn add_collaborator(
    State(pool): State<PgPool>,           // 数据库连接池
    user: AuthUser,                        // 当前用户（自动认证）
    Path(doc_id): Path<Uuid>,              // URL 路径参数
    Json(collab_data): Json<AddCollaboratorByEmail>, // 请求体
) -> AppResult<StatusCode> {
    // 1️⃣ 检查权限：只有文档管理员可以添加协作者
    if !DocumentPermissionService::can_manage(&pool, user.user_id, doc_id).await? {
        return Err(AppError::Forbidden("无权管理该文档协作者".to_string()));
    }

    // 2️⃣ 通过邮箱查找用户并添加为协作者
    DocumentRepository::add_collaborator_by_email(
        &pool,
        doc_id,
        &collab_data.email,
        collab_data.permission
    ).await.map_err(|e| {
        // 友好的错误信息
        if e.to_string().contains("no rows") {
            AppError::NotFound(format!("用户不存在: {}", collab_data.email))
        } else {
            AppError::Database(e)
        }
    })?;

    Ok(StatusCode::CREATED)
}
```

**数据流：**
```
前端发送请求
  ↓
AuthUser 中间件验证身份
  ↓
提取路径参数和请求体
  ↓
检查权限（can_manage）
  ↓
调用 Repository 层
  ↓
数据库操作（查询用户 → 插入协作者）
  ↓
返回结果
```

### 4. 数据访问层

#### 文件位置：`crates/db/src/repository/document.rs`

```rust
pub struct DocumentRepository;

impl DocumentRepository {
    // 通过邮箱添加协作者
    pub async fn add_collaborator_by_email(
        pool: &PgPool,
        doc_id: Uuid,
        email: &str,
        permission: CollaboratorPermission,
    ) -> Result<DocumentCollaborator, sqlx::Error> {
        // 步骤1：根据邮箱查找用户 ID
        let user_id: Uuid = sqlx::query_scalar(
            "SELECT id FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_one(pool)
        .await?;

        // 步骤2：创建协作者关联
        let collab = AddCollaborator { user_id, permission };
        Self::add_collaborator(pool, doc_id, &collab).await
    }

    // 查询协作者列表（包含用户信息）
    pub async fn list_collaborators_with_users(
        pool: &PgPool,
        doc_id: Uuid,
    ) -> Result<Vec<CollaboratorResponse>, sqlx::Error> {
        sqlx::query_as::<_, CollaboratorResponse>(
            r#"
            SELECT
                dc.user_id,
                u.nickname,
                u.email,
                dc.permission,
                dc.created_at
            FROM document_collaborators dc
            JOIN users u ON dc.user_id = u.id
            WHERE dc.document_id = $1
            ORDER BY dc.created_at ASC
            "#,
        )
        .bind(doc_id)
        .fetch_all(pool)
        .await
    }
}
```

**设计模式：Repository 模式**
- 将数据访问逻辑封装到独立的层
- 业务层不直接写 SQL
- 便于测试和维护

### 5. WebSocket 处理器

#### 文件位置：`crates/api/src/ws/handlers.rs`

这是实时协作的**核心**，我们详细分析：

```rust
pub async fn websocket_handler(
    ws: WebSocketUpgrade,              // WebSocket 升级请求
    Path(doc_id): Path<Uuid>,          // 文档 ID
    user: AuthUser,                    // 认证用户
    Extension(hub): Extension<WsHub>,  // WebSocket Hub
    Extension(pool): Extension<PgPool>,// 数据库连接池
) -> Result<Response, AppError> {
    // 1️⃣ 验证文档存在
    let doc = DocumentRepository::find_by_id(&pool, doc_id).await?
        .ok_or_else(|| AppError::NotFound("文档不存在".to_string()))?;

    // 2️⃣ 验证用户权限（至少读权限）
    if !DocumentPermissionService::can_read(&pool, user.user_id, doc_id).await? {
        return Err(AppError::Forbidden("无权访问该文档".to_string()));
    }

    tracing::info!("User {} connecting to document {}", user.user_id, doc_id);

    // 3️⃣ 升级为 WebSocket 连接
    Ok(ws.on_upgrade(move |socket|
        handle_socket(socket, doc_id, user.user_id, hub)
    ))
}
```

#### WebSocket 连接处理

```rust
async fn handle_socket(
    socket: WebSocket,
    doc_id: Uuid,
    user_id: Uuid,
    hub: WsHub
) {
    // 1️⃣ 分离读写流
    let (mut sender, mut receiver) = socket.split();

    // 2️⃣ 加载或创建文档房间
    if let Err(e) = hub.load_document(doc_id).await {
        tracing::debug!("Could not load document from DB: {}", e);
    }
    let room = hub.room_manager().get_or_create_room(doc_id);

    // 3️⃣ 订阅房间广播
    let mut broadcast_rx = room.subscribe();

    // 4️⃣ 用户加入房间
    room.user_join(user_id);

    // 5️⃣ 发送当前文档内容给新用户
    let text_content = room.get_text_content();
    if let Ok(msg) = serde_json::to_string(&WsMessage::Sync {
        update: text_content
    }) {
        sender.send(Message::Text(msg)).await.ok();
    }

    // 6️⃣ 发送其他在线用户的感知状态
    let awareness_states = room.get_all_awareness();
    for (uid, state) in awareness_states {
        if uid != user_id {
            if let Ok(msg) = serde_json::to_string(&WsMessage::Awareness {
                state
            }) {
                sender.send(Message::Text(msg)).await.ok();
            }
        }
    }

    // 7️⃣ 主循环：处理消息
    loop {
        tokio::select! {
            // 处理客户端消息
            msg_result = receiver.next() => {
                match msg_result {
                    Some(Ok(Message::Text(text))) => {
                        // 解析并处理消息
                        if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                            if handle_client_message(&room, user_id, ws_msg).await {
                                // 如果修改了文档，标记为脏
                                hub.mark_dirty(doc_id).await;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => break,
                    Some(Err(e)) => {
                        tracing::warn!("WebSocket error: {}", e);
                        break;
                    }
                    None => break,
                    _ => {}
                }
            }

            // 接收广播消息并转发给客户端
            broadcast_result = broadcast_rx.recv() => {
                match broadcast_result {
                    Ok(msg) => {
                        // 跳过自己发送的消息
                        if should_forward(&msg, user_id) {
                            if let Some(ws_msg) = broadcast_to_ws_message(msg) {
                                if let Ok(json) = serde_json::to_string(&ws_msg) {
                                    sender.send(Message::Text(json)).await.ok();
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
            }

            // 定期发送心跳
            _ = heartbeat_interval.tick() => {
                if sender.send(Message::Ping(vec![])).await.is_err() {
                    break;
                }
            }
        }
    }

    // 8️⃣ 清理：用户离开
    room.user_leave(&user_id);

    // 如果房间空了，保存文档并清理
    if room.get_user_count() == 0 {
        hub.save_document(doc_id).await.ok();
    }
    hub.room_manager().remove_room_if_empty(&doc_id);
}
```

**关键概念：**

**tokio::select!** - 并发等待多个异步操作：
- 同时监听客户端消息、广播消息、心跳定时器
- 哪个先就绪就处理哪个
- 类似 JavaScript 的 `Promise.race()`

#### 消息处理

```rust
async fn handle_client_message(
    room: &DocumentRoom,
    user_id: Uuid,
    msg: WsMessage,
) -> bool {
    match msg {
        WsMessage::Sync { update } => {
            // 方案一：简化文本同步
            // 尝试 hex 解码（CRDT 模式）
            if let Ok(update_bytes) = hex_decode(&update) {
                // CRDT 模式：应用二进制更新
                if room.apply_update(&update_bytes, user_id).is_ok() {
                    return true; // 文档已修改
                }
            } else {
                // 简化模式：直接替换文本内容
                if room.set_text_content(&update, user_id).is_ok() {
                    return true; // 文档已修改
                }
            }
        }
        WsMessage::Awareness { state } => {
            // 更新用户感知状态（光标位置、选择等）
            room.update_awareness(user_id, state);
        }
        _ => {}
    }
    false
}
```

**两种同步模式：**
1. **CRDT 模式**：发送二进制 update（支持复杂的冲突解决）
2. **简化模式**：发送纯文本（当前实现，last-write-wins）

---

## 前端架构深度解析

### 文件位置：`frontend/src/lib.rs`

前端所有代码都在这一个文件中（约 1200 行）。这是 Leptos 小型应用的常见做法。

### 1. 应用入口

```rust
#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                <Route path="/" view=HomePage/>
                <Route path="/register" view=RegisterPage/>
                <Route path="/login" view=LoginPage/>
                <Route path="/documents" view=DocumentsPage/>
                <Route path="/editor/:id" view=EditorPage/>
            </Routes>
        </Router>
    }
}
```

**Leptos Router：**
- 类似 React Router
- `<Route>` 定义路由和对应的组件
- `view=HomePage` 指定渲染的组件

### 2. 登录页面示例

```rust
#[component]
fn LoginPage() -> impl IntoView {
    // 1️⃣ 创建响应式状态
    let (email, set_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (error_msg, set_error_msg) = create_signal(None::<String>);

    // 2️⃣ 获取导航函数
    let navigate = leptos_router::use_navigate();

    // 3️⃣ 定义提交处理函数
    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default(); // 阻止默认表单提交

        // 创建异步任务
        spawn_local(async move {
            // 构建请求体
            let login_data = LoginRequest {
                email: email.get(),
                password: password.get(),
            };

            // 发送 POST 请求
            let response = Request::post("http://127.0.0.1:3000/api/auth/login")
                .json(&login_data)
                .unwrap()
                .send()
                .await;

            match response {
                Ok(resp) if resp.ok() => {
                    // 解析响应
                    let data: LoginResponse = resp.json().await.unwrap();

                    // 保存 token 到 localStorage
                    if let Some(window) = window() {
                        if let Some(storage) = window.local_storage().ok().flatten() {
                            storage.set_item("token", &data.token).ok();
                        }
                    }

                    // 跳转到文档列表
                    navigate("/documents", Default::default());
                }
                _ => {
                    set_error_msg.set(Some("登录失败".to_string()));
                }
            }
        });
    };

    // 4️⃣ 渲染 UI
    view! {
        <div class="auth-container">
            <h1>"登录"</h1>
            {move || error_msg.get().map(|msg| view! {
                <div class="error">{msg}</div>
            })}
            <form on:submit=on_submit>
                <input
                    type="email"
                    placeholder="邮箱"
                    prop:value=move || email.get()
                    on:input=move |ev| set_email.set(event_target_value(&ev))
                />
                <input
                    type="password"
                    placeholder="密码"
                    prop:value=move || password.get()
                    on:input=move |ev| set_password.set(event_target_value(&ev))
                />
                <button type="submit">"登录"</button>
            </form>
        </div>
    }
}
```

**Leptos 核心概念：**

1. **Signal（信号）**：
   ```rust
   let (email, set_email) = create_signal(String::new());
   ```
   - `email` 是读取函数
   - `set_email` 是更新函数
   - 类似 React 的 `useState`

2. **事件处理**：
   ```rust
   on:input=move |ev| set_email.set(event_target_value(&ev))
   ```
   - `move` 捕获外部变量
   - 自动更新 UI

3. **异步操作**：
   ```rust
   spawn_local(async move { /* ... */ });
   ```
   - 在 WASM 中运行异步代码

### 3. 编辑器页面（最复杂）

#### 文件位置：`frontend/src/lib.rs` 第 900-1200 行

```rust
#[component]
fn EditorPage() -> impl IntoView {
    // 1️⃣ 获取文档 ID
    let params = use_params_map();
    let doc_id = move || params.with(|p| p.get("id").cloned().unwrap_or_default());

    // 2️⃣ 状态管理
    let (content, set_content) = create_signal(String::new());
    let (title, set_title) = create_signal(String::new());
    let (loading, set_loading) = create_signal(true);
    let (ws, set_ws) = create_signal(None::<WebSocket>);
    let (syncing, set_syncing) = create_signal(false);
    let (show_collab, set_show_collab) = create_signal(false);
    let (online_users, set_online_users) = create_signal(Vec::<OnlineUser>::new());
    let (collaborators, set_collaborators) = create_signal(Vec::<CollaboratorResponse>::new());

    // 3️⃣ 加载文档数据
    create_effect(move |_| {
        let id = doc_id();
        if id.is_empty() { return; }

        spawn_local(async move {
            // 从 API 获取文档
            let doc: DocumentResponse = fetch_document(&id).await.unwrap();
            set_title.set(doc.title);
            set_content.set(doc.content);
            set_loading.set(false);

            // 建立 WebSocket 连接
            setup_websocket(&id, set_ws, set_content, set_syncing, set_online_users);
        });
    });

    // 4️⃣ WebSocket 连接设置
    fn setup_websocket(/* ... */) {
        let token = get_token_from_storage();
        let ws_url = format!("ws://127.0.0.1:3000/ws/documents/{}?token={}",
            doc_id, token);

        let websocket = WebSocket::new(&ws_url).unwrap();

        // 连接打开事件
        let onopen_callback = Closure::wrap(Box::new(move || {
            leptos::logging::log!("WebSocket connected");
        }) as Box<dyn Fn()>);
        websocket.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        // 接收消息事件
        let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                let msg_str: String = txt.into();

                // 解析消息
                if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&msg_str) {
                    match ws_msg {
                        WsMessage::Sync { update } => {
                            // 更新编辑器内容
                            set_content.set(update);
                            set_syncing.set(false);
                        }
                        WsMessage::UserJoined { user_id, nickname } => {
                            // 添加在线用户
                            set_online_users.update(|users| {
                                users.push(OnlineUser { user_id, nickname });
                            });
                        }
                        WsMessage::UserLeft { user_id } => {
                            // 移除离线用户
                            set_online_users.update(|users| {
                                users.retain(|u| u.user_id != user_id);
                            });
                        }
                        _ => {}
                    }
                }
            }
        }) as Box<dyn Fn(MessageEvent)>);
        websocket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        set_ws.set(Some(websocket));
    }

    // 5️⃣ 内容变化处理（带防抖）
    let debounce_timer = create_signal(None::<i32>).1;

    let on_content_change = move |ev| {
        let new_content = event_target_value(&ev);
        set_content.set(new_content.clone());

        // 清除旧定时器
        if let Some(timer_id) = debounce_timer.get() {
            window().unwrap().clear_timeout_with_handle(timer_id);
        }

        // 设置新的 500ms 延迟
        let ws_clone = ws.get();
        let callback = Closure::wrap(Box::new(move || {
            // 500ms 后发送更新
            if let Some(websocket) = ws_clone.as_ref() {
                let msg = WsMessage::Sync { update: new_content.clone() };
                let msg_json = serde_json::to_string(&msg).unwrap();
                websocket.send_with_str(&msg_json).ok();
            }
        }) as Box<dyn Fn()>);

        let timer_id = window().unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                500
            ).unwrap();

        debounce_timer.set(Some(timer_id));
        callback.forget();
    };

    // 6️⃣ 渲染 UI
    view! {
        <div class="editor-container">
            {move || if loading.get() {
                view! { <div>"Loading..."</div> }.into_view()
            } else {
                view! {
                    <div class="editor-header">
                        <input
                            type="text"
                            class="title-input"
                            prop:value=move || title.get()
                            on:input=move |ev| set_title.set(event_target_value(&ev))
                        />
                        <button on:click=move |_| set_show_collab.set(!show_collab.get())>
                            "👥 协作者"
                        </button>
                    </div>

                    <textarea
                        class="editor-textarea"
                        prop:value=move || content.get()
                        on:input=on_content_change
                    />

                    // 同步状态显示
                    <div class="sync-status">
                        {move || if syncing.get() {
                            view! { <span>"⟳ 正在同步..."</span> }
                        } else {
                            view! { <span>"✓ 已同步"</span> }
                        }}
                    </div>

                    // 协作者面板
                    {move || if show_collab.get() {
                        view! { <CollaboratorPanel doc_id=doc_id() /> }
                    } else {
                        view! { <div/> }
                    }}
                }.into_view()
            }}
        </div>
    }
}
```

**关键技术点：**

1. **防抖（Debouncing）**：
   - 用户每次输入后等待 500ms
   - 如果在 500ms 内又输入，重置定时器
   - 避免每次按键都发送请求

2. **WebSocket 生命周期**：
   - `onopen`：连接建立
   - `onmessage`：接收消息
   - `onerror`：错误处理
   - `onclose`：连接关闭

3. **Closure.forget()**：
   - JavaScript 回调需要保持活跃
   - `forget()` 防止 Rust 释放内存

---

## 实时协作系统详解

### 协作房间（DocumentRoom）

#### 文件位置：`crates/collab/src/sync.rs`

```rust
pub struct DocumentRoom {
    doc_id: Uuid,                              // 文档 ID
    document: CollabDocument,                  // CRDT 文档
    awareness: AwarenessManager,               // 用户感知状态
    connections: Arc<RwLock<HashMap<Uuid, ConnectionInfo>>>, // 在线用户
    broadcast_tx: broadcast::Sender<BroadcastMessage>,       // 广播通道
}

impl DocumentRoom {
    // 用户加入房间
    pub fn user_join(&self, user_id: Uuid) {
        // 记录连接信息
        self.connections.write().unwrap().insert(
            user_id,
            ConnectionInfo {
                user_id,
                connected_at: chrono::Utc::now(),
            },
        );

        // 广播用户加入消息
        self.broadcast_tx.send(BroadcastMessage::UserJoined { user_id }).ok();
    }

    // 设置文本内容（简化模式）
    pub fn set_text_content(&self, content: &str, from_user: Uuid)
        -> Result<(), CollabError>
    {
        // 更新 CRDT 文档
        self.document.set_text("content", content)?;

        // 广播给其他用户
        self.broadcast_tx.send(BroadcastMessage::TextUpdate {
            from_user,
            content: content.to_string(),
        }).ok();

        Ok(())
    }

    // 获取文本内容
    pub fn get_text_content(&self) -> String {
        self.document.get_default_text()
    }
}
```

**广播机制：**
```
用户 A 发送更新
    ↓
DocumentRoom.set_text_content()
    ↓
broadcast_tx.send(TextUpdate)
    ↓
所有订阅者（broadcast_rx）收到消息
    ↓
通过 WebSocket 转发给用户 B、C、D...
```

### CRDT 文档

#### 文件位置：`crates/collab/src/document.rs`

```rust
pub struct CollabDocument {
    doc_id: Uuid,
    doc: Arc<Doc>,  // Yrs CRDT 文档
}

impl CollabDocument {
    // 设置文本
    pub fn set_text(&self, text_name: &str, content: &str)
        -> Result<(), CollabError>
    {
        let mut txn = self.doc.transact_mut();
        let text = txn.get_or_insert_text(text_name);

        // 删除旧内容
        let len = text.len(&txn);
        if len > 0 {
            text.remove_range(&mut txn, 0, len);
        }

        // 插入新内容
        text.insert(&mut txn, 0, content);

        Ok(())
    }

    // 获取完整状态（用于持久化）
    pub fn get_full_state(&self) -> Vec<u8> {
        let txn = self.doc.transact();
        txn.encode_state_as_update_v1(&StateVector::default())
    }
}
```

**为什么使用 CRDT？**
- **离线编辑**：用户断网后继续编辑，重连后自动合并
- **冲突解决**：多人同时编辑不同部分，自动合并
- **最终一致性**：所有用户最终看到相同内容

---

## 数据库设计

### 核心表结构

#### 1. users（用户表）
```sql
CREATE TABLE entangle.users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    nickname VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

#### 2. documents（文档表）
```sql
CREATE TABLE entangle.documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    crdt_state BYTEA,  -- 新增：CRDT 状态存储
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    folder_id UUID REFERENCES folders(id) ON DELETE SET NULL
);

CREATE INDEX idx_documents_owner ON documents(owner_id);
CREATE INDEX idx_documents_updated_at ON documents(updated_at DESC);
```

**crdt_state 字段：**
- 存储 Yrs CRDT 的二进制状态
- 用于文档恢复和离线编辑

#### 3. document_collaborators（协作者表）
```sql
CREATE TABLE entangle.document_collaborators (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permission VARCHAR(50) NOT NULL,  -- 'read', 'write', 'admin'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(document_id, user_id)
);

CREATE INDEX idx_collab_document ON document_collaborators(document_id);
CREATE INDEX idx_collab_user ON document_collaborators(user_id);
```

**唯一约束**：同一用户不能重复添加为协作者

---

## 关键业务流程

### 流程1：用户登录

```
1. 前端：用户输入邮箱密码
   ↓
2. 前端：发送 POST /api/auth/login
   ↓
3. 后端：查询数据库验证密码
   ↓
4. 后端：生成 JWT token（包含 user_id、过期时间）
   ↓
5. 后端：返回 { token, user: { id, email, nickname } }
   ↓
6. 前端：保存 token 到 localStorage
   ↓
7. 前端：跳转到 /documents
```

**JWT Token 结构：**
```rust
pub struct Claims {
    pub sub: Uuid,        // Subject（用户 ID）
    pub exp: usize,       // Expiration（过期时间）
    pub iat: usize,       // Issued At（签发时间）
}
```

### 流程2：添加协作者

```
1. 前端：文档所有者输入协作者邮箱
   ↓
2. 前端：POST /api/documents/{doc_id}/collaborators
   Body: { email: "user@example.com", permission: "write" }
   Header: Authorization: Bearer {token}
   ↓
3. 后端：AuthUser 中间件验证 token
   ↓
4. 后端：检查当前用户是否有 manage 权限
   ↓
5. 后端：根据邮箱查询用户 ID
   SELECT id FROM users WHERE email = $1
   ↓
6. 后端：插入协作者记录
   INSERT INTO document_collaborators (document_id, user_id, permission)
   VALUES ($1, $2, $3)
   ↓
7. 后端：返回 201 Created
   ↓
8. 前端：刷新协作者列表
```

### 流程3：实时编辑同步

```
【用户 A 编辑】
1. 用户 A：在编辑器输入文字
   ↓
2. 前端 A：on_input 事件触发
   ↓
3. 前端 A：启动 500ms 防抖定时器
   ↓
4. （500ms 后）前端 A：通过 WebSocket 发送
   { type: "sync", update: "新的文档内容" }
   ↓
5. 后端：接收消息 → handle_client_message()
   ↓
6. 后端：检测为纯文本模式 → room.set_text_content()
   ↓
7. 后端：更新 CRDT 文档
   ↓
8. 后端：广播 TextUpdate { from_user: A, content: "..." }
   ↓

【用户 B 接收】
9. 后端：broadcast_rx 收到消息
   ↓
10. 后端：通过 WebSocket 发送给用户 B
    { type: "sync", update: "新的文档内容" }
    ↓
11. 前端 B：onmessage 事件触发
    ↓
12. 前端 B：解析消息 → WsMessage::Sync
    ↓
13. 前端 B：set_content.set(update)
    ↓
14. 前端 B：编辑器内容自动更新
```

**关键点：**
- 防抖避免频繁发送
- 广播机制确保其他用户实时收到
- 跳过自己的更新（避免循环）

---

## 代码导读

### 如何阅读这个项目？

#### 推荐阅读顺序

1. **数据模型**（理解数据结构）
   - `crates/db/src/models/user.rs`
   - `crates/db/src/models/document.rs`
   - `crates/db/src/models/mod.rs`

2. **认证系统**（理解用户身份验证）
   - `crates/auth/src/lib.rs`（JWT 生成和验证）
   - `crates/api/src/middleware/auth.rs`（认证中间件）
   - `crates/api/src/routes/auth.rs`（登录注册路由）

3. **文档 CRUD**（理解基本业务逻辑）
   - `crates/db/src/repository/document.rs`（数据访问层）
   - `crates/core/src/permissions.rs`（权限检查）
   - `crates/api/src/routes/documents.rs`（RESTful API）

4. **实时协作核心**（理解最复杂的部分）
   - `crates/collab/src/document.rs`（CRDT 文档）
   - `crates/collab/src/sync.rs`（房间管理）
   - `crates/api/src/ws/handlers.rs`（WebSocket 处理）

5. **前端页面**（理解用户交互）
   - `frontend/src/lib.rs`（从 App 组件开始）
   - 顺序阅读：LoginPage → DocumentsPage → EditorPage

### 关键函数详解

#### 1. 权限检查

```rust
// 文件：crates/core/src/permissions.rs
pub struct DocumentPermissionService;

impl DocumentPermissionService {
    // 检查用户是否可以读取文档
    pub async fn can_read(
        pool: &PgPool,
        user_id: Uuid,
        doc_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        // 1. 检查是否是文档所有者
        let is_owner: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM documents WHERE id = $1 AND owner_id = $2)"
        )
        .bind(doc_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        if is_owner {
            return Ok(true);
        }

        // 2. 检查是否是公开文档
        let is_public: bool = sqlx::query_scalar(
            "SELECT is_public FROM documents WHERE id = $1"
        )
        .bind(doc_id)
        .fetch_one(pool)
        .await?;

        if is_public {
            return Ok(true);
        }

        // 3. 检查是否是协作者
        let is_collaborator: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM document_collaborators
             WHERE document_id = $1 AND user_id = $2)"
        )
        .bind(doc_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(is_collaborator)
    }

    // 检查是否可以管理协作者（只有 admin 权限）
    pub async fn can_manage(
        pool: &PgPool,
        user_id: Uuid,
        doc_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        // 1. 文档所有者可以管理
        let is_owner: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM documents WHERE id = $1 AND owner_id = $2)"
        )
        .bind(doc_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        if is_owner {
            return Ok(true);
        }

        // 2. admin 权限的协作者可以管理
        let is_admin: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM document_collaborators
             WHERE document_id = $1 AND user_id = $2 AND permission = 'admin')"
        )
        .bind(doc_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(is_admin)
    }
}
```

**权限层级：**
- **所有者**：完全控制（可删除文档）
- **admin 协作者**：可管理其他协作者
- **write 协作者**：可编辑内容
- **read 协作者**：只读

#### 2. WebSocket Hub（中心管理器）

```rust
// 文件：crates/api/src/ws/mod.rs
pub struct WsHub {
    room_manager: RoomManager,           // 房间管理器
    pool: Option<PgPool>,                // 数据库连接池
    dirty_docs: Arc<RwLock<HashSet<Uuid>>>, // 脏文档集合（需要保存）
}

impl WsHub {
    // 加载文档从数据库
    pub async fn load_document(&self, doc_id: Uuid) -> anyhow::Result<()> {
        let pool = self.pool.as_ref().ok_or(anyhow::anyhow!("No pool"))?;

        // 查询文档
        let doc = DocumentRepository::find_by_id(pool, doc_id).await?
            .ok_or(anyhow::anyhow!("Document not found"))?;

        // 加载 CRDT 状态
        let state = doc.crdt_state.unwrap_or_default();

        // 创建或更新房间
        if state.is_empty() {
            self.room_manager.get_or_create_room(doc_id);
        } else {
            self.room_manager.get_or_create_room_with_state(doc_id, &state)?;
        }

        tracing::info!("Loaded CRDT state for document {}", doc_id);
        Ok(())
    }

    // 保存文档到数据库
    pub async fn save_document(&self, doc_id: Uuid) -> anyhow::Result<()> {
        let pool = self.pool.as_ref().ok_or(anyhow::anyhow!("No pool"))?;

        // 获取房间
        let room = self.room_manager.get_room(&doc_id)
            .ok_or(anyhow::anyhow!("Room not found"))?;

        // 获取 CRDT 状态
        let state = room.get_state();

        // 更新数据库
        sqlx::query(
            "UPDATE documents SET crdt_state = $1, updated_at = NOW() WHERE id = $2"
        )
        .bind(&state)
        .bind(doc_id)
        .execute(pool)
        .await?;

        // 从脏文档集合移除
        self.dirty_docs.write().unwrap().remove(&doc_id);

        tracing::info!("Saved CRDT state for document {}", doc_id);
        Ok(())
    }

    // 标记文档为脏（需要保存）
    pub async fn mark_dirty(&self, doc_id: Uuid) {
        self.dirty_docs.write().unwrap().insert(doc_id);
    }
}
```

**自动保存机制：**
- 用户编辑时标记文档为"脏"
- 最后一个用户离开时自动保存
- 定期后台任务保存所有脏文档（可选）

---

## 回答老师可能的问题

### Q1: 为什么使用 Rust？

**回答：**
1. **类型安全**：编译时捕获大部分错误，减少运行时 bug
2. **性能**：接近 C/C++ 的性能，适合 WebSocket 高并发场景
3. **内存安全**：无垃圾回收，无空指针，无数据竞争
4. **全栈统一**：前后端同一语言，代码复用（如数据模型）
5. **WASM 支持**：前端编译为 WebAssembly，性能优于 JavaScript

### Q2: CRDT 是如何工作的？

**回答：**
CRDT（无冲突复制数据类型）通过以下方式解决冲突：

1. **操作基础**：不存储最终状态，而是存储操作历史
   ```
   用户 A：在位置 0 插入 "H"
   用户 B：在位置 0 插入 "W"
   ```

2. **操作 ID**：每个操作有唯一 ID（user_id + 时间戳 + 序号）
   ```
   { id: (userA, 100, 1), pos: 0, insert: "H" }
   { id: (userB, 101, 1), pos: 0, insert: "W" }
   ```

3. **确定性合并**：根据 ID 排序，保证所有用户看到相同顺序
   ```
   结果：根据 ID 比较，可能是 "HW" 或 "WH"（取决于 ID 大小）
   ```

4. **最终一致性**：所有用户收到全部操作后，文档状态完全相同

**Yrs 实现细节：**
- 使用 Lamport 时间戳
- 墓碑机制处理删除
- 压缩历史减少内存占用

### Q3: WebSocket 和 HTTP 的区别？

**回答：**

| 特性 | HTTP | WebSocket |
|------|------|-----------|
| 连接 | 短连接（请求-响应后关闭）| 长连接（保持打开）|
| 通信 | 单向（客户端 → 服务器）| 双向（客户端 ⇄ 服务器）|
| 协议 | 无状态 | 有状态 |
| 开销 | 每次请求都有 Header | 建立连接后开销小 |
| 适用场景 | API、页面加载 | 实时聊天、协作编辑 |

**为什么实时编辑需要 WebSocket？**
- HTTP 轮询浪费资源（每秒发请求检查更新）
- WebSocket 服务器可主动推送，延迟低
- 适合高频率小数据传输（如键盘输入）

### Q4: 防抖（Debounce）是什么？

**回答：**
防抖是一种优化技术，延迟执行函数直到一段时间内没有新的触发。

**示例：**
```rust
// 用户输入 "hello"
// 每次按键触发 on_input

// 不使用防抖：
h → 发送请求
he → 发送请求
hel → 发送请求
hell → 发送请求
hello → 发送请求
// 总共 5 次请求！

// 使用 500ms 防抖：
h → 启动定时器（500ms）
he → 重置定时器（500ms）
hel → 重置定时器（500ms）
hell → 重置定时器（500ms）
hello → 重置定时器（500ms）
（500ms 后）→ 发送请求
// 总共 1 次请求！
```

**代码实现：**
```rust
// 清除旧定时器
if let Some(timer_id) = old_timer.get() {
    window.clear_timeout_with_handle(timer_id);
}

// 设置新定时器
let timer_id = window.set_timeout_with_callback_and_timeout(
    callback,
    500  // 500ms 延迟
).unwrap();
```

### Q5: 如何保证并发安全？

**回答：**

1. **数据库层**：使用事务和唯一约束
   ```sql
   -- 防止重复添加协作者
   UNIQUE(document_id, user_id)
   ```

2. **应用层**：使用 Rust 的所有权系统
   ```rust
   // Arc<RwLock<T>>：多线程共享可变数据
   connections: Arc<RwLock<HashMap<Uuid, ConnectionInfo>>>

   // 读锁：多个线程可同时读
   let users = connections.read().unwrap();

   // 写锁：独占访问
   let mut users = connections.write().unwrap();
   users.insert(user_id, info);
   ```

3. **WebSocket 层**：每个连接独立的 tokio 任务
   - 任务间通过 channel 通信（send/recv）
   - 避免共享状态

### Q6: 如何扩展到多服务器？

**当前架构限制：**
- 所有 WebSocket 连接在同一进程
- 房间数据只在内存中

**扩展方案：**

1. **Redis Pub/Sub**
   ```rust
   // 服务器 A：用户编辑
   redis.publish("doc:{doc_id}", TextUpdate { ... });

   // 服务器 B：订阅该文档
   redis.subscribe("doc:{doc_id}", |msg| {
       // 转发给本地 WebSocket 连接
   });
   ```

2. **数据库作为消息队列**
   - 使用 PostgreSQL LISTEN/NOTIFY
   - 适合小规模部署

3. **专用消息队列**
   - RabbitMQ / Kafka
   - 适合大规模部署

---

## 总结

### 项目亮点

1. **全栈 Rust**：前后端类型安全，代码复用
2. **实时协作**：WebSocket + CRDT，支持多人编辑
3. **权限系统**：细粒度权限控制（owner/admin/write/read）
4. **异步优先**：Tokio 运行时，高并发性能
5. **模块化设计**：清晰的层次结构，易于维护

### 技术难点

1. **CRDT 集成**：理解 Yrs 库，处理状态序列化
2. **WebSocket 生命周期**：连接管理、错误处理、心跳检测
3. **前端 WASM**：Closure 内存管理，JavaScript 互操作
4. **并发控制**：Arc/RwLock 使用，避免死锁

### 改进方向

1. **冲突可视化**：显示其他用户的光标和选择
2. **离线支持**：Service Worker + IndexedDB
3. **版本历史**：文档快照和回滚
4. **性能优化**：增量同步（只发送差异）
5. **监控告警**：WebSocket 连接数、延迟统计

---

**文档结束**

如有疑问，可以查看代码注释或联系开发者。祝演示成功！
