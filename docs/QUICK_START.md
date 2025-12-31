# Entangle 快速开发指南

> 全栈 Rust 方案 - 本地演示 - 快速产出

---

## 技术栈调整 (快速产出版)

### 推荐技术栈

| 组件 | 选择 | 原因 |
|------|------|------|
| **后端** | Axum | 简洁、高性能 |
| **前端** | Leptos | 全栈 Rust，无需学 JS |
| **数据库** | SQLite → openGauss | 开发用 SQLite，演示用 openGauss |
| **实时协作** | yrs + WebSocket | 成熟的 CRDT 方案 |
| **样式** | TailwindCSS | 快速 UI 开发 |

### 为什么选 Leptos？

```
┌─────────────────────────────────────────────────────────────┐
│  Leptos 优势 (快速产出)                                      │
├─────────────────────────────────────────────────────────────┤
│  ✅ 全栈 Rust，一种语言搞定前后端                            │
│  ✅ 响应式系统类似 SolidJS，性能好                           │
│  ✅ SSR + Hydration 支持                                    │
│  ✅ 组件化开发，代码复用                                     │
│  ✅ 社区活跃，文档完善                                       │
│  ✅ 与 Axum 无缝集成                                        │
└─────────────────────────────────────────────────────────────┘
```

---

## 简化版功能清单

### 必做 (MVP)

```
✅ 用户注册/登录 (JWT)
✅ 文档 CRUD
✅ 富文本编辑 (简化版)
✅ 实时协作 (核心)
✅ 基础权限
✅ 评论功能
```

### 选做 (加分)

```
🟢 Markdown 支持
🟢 版本历史
🟢 文档导出
```

### 砍掉 (节省时间)

```
❌ 手机号注册 (只做邮箱)
❌ 邮件验证 (本地演示不需要)
❌ 头像上传 (用默认头像)
❌ 视频会议
❌ 离线编辑
```

---

## 项目结构 (简化版)

```
entangle/
├── Cargo.toml
├── src/
│   ├── main.rs              # 入口
│   ├── app.rs               # Leptos App 组件
│   ├── server/              # 后端逻辑
│   │   ├── mod.rs
│   │   ├── auth.rs          # 认证
│   │   ├── document.rs      # 文档 API
│   │   ├── collab.rs        # 协作 WebSocket
│   │   └── db.rs            # 数据库
│   ├── components/          # Leptos 组件
│   │   ├── mod.rs
│   │   ├── editor.rs        # 编辑器
│   │   ├── sidebar.rs       # 侧边栏
│   │   └── ...
│   ├── pages/               # 页面组件
│   │   ├── mod.rs
│   │   ├── home.rs
│   │   ├── login.rs
│   │   ├── document.rs
│   │   └── ...
│   └── models/              # 数据模型
│       └── ...
├── style/
│   └── main.css             # TailwindCSS
├── migrations/              # 数据库迁移
└── public/                  # 静态资源
```

---

## 快速开始

### 1. 初始化项目

```bash
# 创建项目
cargo new entangle
cd entangle

# 添加依赖
cargo add axum tokio --features full
cargo add leptos --features ssr,hydrate
cargo add leptos_axum
cargo add sqlx --features runtime-tokio,postgres
cargo add yrs
cargo add serde --features derive
cargo add serde_json
cargo add jsonwebtoken
cargo add argon2
cargo add uuid --features v4,serde
cargo add chrono --features serde
cargo add tower-http --features cors,fs
cargo add tokio-tungstenite
cargo add thiserror
cargo add tracing tracing-subscriber

# 安装 trunk (前端构建工具)
cargo install trunk

# 安装 cargo-leptos
cargo install cargo-leptos
```

### 2. Cargo.toml 配置

```toml
[package]
name = "entangle"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
# Web 框架
axum = { version = "0.7", features = ["ws", "macros"] }
tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.5", features = ["cors", "fs"] }

# Leptos 全栈
leptos = { version = "0.6", features = ["ssr"] }
leptos_axum = "0.6"
leptos_meta = "0.6"
leptos_router = "0.6"

# 数据库
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "uuid", "chrono"] }

# 实时协作
yrs = "0.18"
tokio-tungstenite = "0.21"

# 认证
jsonwebtoken = "9"
argon2 = "0.5"

# 工具
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1"
tracing = "0.1"
tracing-subscriber = "0.3"

[features]
ssr = ["leptos/ssr", "leptos_axum"]
hydrate = ["leptos/hydrate"]

[[bin]]
name = "entangle"
path = "src/main.rs"

[profile.release]
lto = true
opt-level = 3
```

### 3. 环境配置

```env
# .env
DATABASE_URL=postgres://gaussdb:password@localhost:5432/entangle
JWT_SECRET=your-super-secret-key-at-least-32-characters
APP_PORT=3000
```

---

## 核心代码骨架

### main.rs

```rust
use axum::{routing::get, Router};
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use std::net::SocketAddr;
use tokio::net::TcpListener;

mod app;
mod server;
mod components;
mod pages;
mod models;

use app::App;
use server::collab::collab_ws_handler;

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 初始化数据库
    let db_pool = server::db::init_pool().await.expect("Failed to connect to database");

    // Leptos 配置
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // 构建路由
    let app = Router::new()
        // API 路由
        .nest("/api", server::api_routes(db_pool.clone()))
        // WebSocket 协作
        .route("/ws/doc/:doc_id", get(collab_ws_handler))
        // Leptos 路由
        .leptos_routes(&leptos_options, routes, App)
        .fallback(leptos_axum::file_and_error_handler(leptos_options))
        .with_state(db_pool);

    // 启动服务器
    let listener = TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Server running at http://{}", addr);
    axum::serve(listener, app.into_make_service()).await.unwrap();
}
```

### app.rs (Leptos 根组件)

```rust
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::pages::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/entangle.css"/>
        <Title text="Entangle - 协作文档"/>

        <Router>
            <main class="min-h-screen bg-gray-50">
                <Routes>
                    <Route path="/" view=HomePage/>
                    <Route path="/login" view=LoginPage/>
                    <Route path="/register" view=RegisterPage/>
                    <Route path="/documents" view=DocumentListPage/>
                    <Route path="/doc/:id" view=DocumentEditorPage/>
                    <Route path="/*any" view=NotFoundPage/>
                </Routes>
            </main>
        </Router>
    }
}
```

### server/db.rs (数据库)

```rust
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;

pub type DbPool = PgPool;

pub async fn init_pool() -> Result<DbPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
}
```

### server/auth.rs (认证)

```rust
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,        // user_id
    pub email: String,
    pub role: String,
    pub exp: usize,       // expiration
}

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    Ok(argon2.hash_password(password.as_bytes(), &salt)?.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    let parsed_hash = PasswordHash::new(hash).ok();
    parsed_hash.map_or(false, |h| {
        Argon2::default().verify_password(password.as_bytes(), &h).is_ok()
    })
}

pub fn create_token(user_id: Uuid, email: &str, role: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        email: email.to_string(),
        role: role.to_string(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}
```

### server/collab.rs (实时协作)

```rust
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use yrs::{Doc, Transact, Update};

// 文档房间
pub struct DocRoom {
    pub doc: Doc,
    pub tx: broadcast::Sender<Vec<u8>>,
}

// 全局房间管理
pub type Rooms = Arc<RwLock<HashMap<String, Arc<DocRoom>>>>;

pub async fn collab_ws_handler(
    ws: WebSocketUpgrade,
    Path(doc_id): Path<String>,
    State(rooms): State<Rooms>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, doc_id, rooms))
}

async fn handle_socket(socket: WebSocket, doc_id: String, rooms: Rooms) {
    let (mut sender, mut receiver) = socket.split();

    // 获取或创建房间
    let room = {
        let mut rooms_guard = rooms.write().await;
        rooms_guard
            .entry(doc_id.clone())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(100);
                Arc::new(DocRoom {
                    doc: Doc::new(),
                    tx,
                })
            })
            .clone()
    };

    // 订阅房间广播
    let mut rx = room.tx.subscribe();

    // 发送当前文档状态
    let state = room.doc.transact().state_vector();
    let _ = sender.send(Message::Binary(state.encode_v1())).await;

    // 处理接收和广播
    loop {
        tokio::select! {
            // 接收客户端消息
            Some(msg) = receiver.next() => {
                if let Ok(Message::Binary(data)) = msg {
                    // 应用更新到文档
                    if let Ok(update) = Update::decode_v1(&data) {
                        room.doc.transact_mut().apply_update(update);
                        // 广播给其他用户
                        let _ = room.tx.send(data);
                    }
                }
            }
            // 接收广播消息
            Ok(data) = rx.recv() => {
                let _ = sender.send(Message::Binary(data)).await;
            }
            else => break,
        }
    }
}
```

### pages/login.rs (登录页面)

```rust
use leptos::*;
use leptos_router::*;

#[server(Login, "/api")]
pub async fn login(email: String, password: String) -> Result<String, ServerFnError> {
    use crate::server::{auth, db::DbPool};
    use sqlx::Row;

    let pool = use_context::<DbPool>().expect("DB pool not found");

    // 查询用户
    let row = sqlx::query("SELECT id, password_hash, role FROM users WHERE email = $1")
        .bind(&email)
        .fetch_optional(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?
        .ok_or_else(|| ServerFnError::ServerError("用户不存在".to_string()))?;

    let user_id: uuid::Uuid = row.get("id");
    let password_hash: String = row.get("password_hash");
    let role: String = row.get("role");

    // 验证密码
    if !auth::verify_password(&password, &password_hash) {
        return Err(ServerFnError::ServerError("密码错误".to_string()));
    }

    // 生成 JWT
    auth::create_token(user_id, &email, &role)
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let login_action = create_server_action::<Login>();
    let (error, set_error) = create_signal(None::<String>);

    let on_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        let form_data = ev.target();
        // 表单处理...
    };

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50">
            <div class="max-w-md w-full space-y-8 p-8 bg-white rounded-lg shadow">
                <h2 class="text-3xl font-bold text-center text-gray-900">
                    "登录 Entangle"
                </h2>

                <ActionForm action=login_action class="mt-8 space-y-6">
                    <div>
                        <label for="email" class="block text-sm font-medium text-gray-700">
                            "邮箱"
                        </label>
                        <input
                            type="email"
                            name="email"
                            required
                            class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                        />
                    </div>

                    <div>
                        <label for="password" class="block text-sm font-medium text-gray-700">
                            "密码"
                        </label>
                        <input
                            type="password"
                            name="password"
                            required
                            class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                        />
                    </div>

                    <button
                        type="submit"
                        class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                    >
                        "登录"
                    </button>
                </ActionForm>

                <p class="mt-4 text-center text-sm text-gray-600">
                    "还没有账号？"
                    <A href="/register" class="text-indigo-600 hover:text-indigo-500">
                        "立即注册"
                    </A>
                </p>
            </div>
        </div>
    }
}
```

### components/editor.rs (简化编辑器)

```rust
use leptos::*;
use wasm_bindgen::prelude::*;

#[component]
pub fn Editor(
    #[prop(into)] doc_id: String,
    #[prop(into)] content: RwSignal<String>,
) -> impl IntoView {
    let ws_connected = create_rw_signal(false);

    // WebSocket 连接
    create_effect(move |_| {
        let doc_id = doc_id.clone();
        spawn_local(async move {
            // 连接 WebSocket
            let ws_url = format!("ws://localhost:3000/ws/doc/{}", doc_id);
            // WebSocket 连接逻辑...
        });
    });

    view! {
        <div class="editor-container h-full flex flex-col">
            // 工具栏
            <div class="toolbar flex items-center gap-2 p-2 border-b bg-gray-50">
                <button
                    class="px-3 py-1 rounded hover:bg-gray-200 font-bold"
                    on:click=move |_| {
                        // 加粗
                    }
                >
                    "B"
                </button>
                <button
                    class="px-3 py-1 rounded hover:bg-gray-200 italic"
                    on:click=move |_| {
                        // 斜体
                    }
                >
                    "I"
                </button>
                <button
                    class="px-3 py-1 rounded hover:bg-gray-200 underline"
                    on:click=move |_| {
                        // 下划线
                    }
                >
                    "U"
                </button>
                <div class="flex-1"/>
                <div class="flex items-center gap-2">
                    <span
                        class=move || {
                            if ws_connected.get() {
                                "w-2 h-2 rounded-full bg-green-500"
                            } else {
                                "w-2 h-2 rounded-full bg-red-500"
                            }
                        }
                    />
                    <span class="text-sm text-gray-500">
                        {move || if ws_connected.get() { "已连接" } else { "连接中..." }}
                    </span>
                </div>
            </div>

            // 编辑区域
            <div
                class="flex-1 p-4 overflow-auto"
                contenteditable="true"
                on:input=move |ev| {
                    // 处理输入，同步到 CRDT
                }
                inner_html=move || content.get()
            />
        </div>
    }
}
```

---

## 数据库迁移脚本

### migrations/001_init.sql

```sql
-- 创建扩展
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- 用户表
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    nickname VARCHAR(100) NOT NULL,
    role VARCHAR(20) DEFAULT 'editor',
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- 文档表
CREATE TABLE documents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title VARCHAR(500) NOT NULL DEFAULT '无标题文档',
    content BYTEA,  -- Yjs 文档状态
    owner_id UUID NOT NULL REFERENCES users(id),
    is_public BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- 文档协作者
CREATE TABLE doc_collaborators (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    doc_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL DEFAULT 'viewer',
    created_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(doc_id, user_id)
);

-- 评论表
CREATE TABLE comments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    doc_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    parent_id UUID REFERENCES comments(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    position JSONB,
    is_resolved BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT NOW()
);

-- 版本历史（选做）
CREATE TABLE doc_versions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    doc_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    version INTEGER NOT NULL,
    content BYTEA NOT NULL,
    created_by UUID REFERENCES users(id),
    message VARCHAR(500),
    created_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(doc_id, version)
);

-- 索引
CREATE INDEX idx_documents_owner ON documents(owner_id);
CREATE INDEX idx_doc_collaborators_doc ON doc_collaborators(doc_id);
CREATE INDEX idx_doc_collaborators_user ON doc_collaborators(user_id);
CREATE INDEX idx_comments_doc ON comments(doc_id);
```

---

## 开发流程

### 第 1 天：项目初始化

```bash
# 1. 创建项目
cargo new entangle && cd entangle

# 2. 配置 Cargo.toml (复制上面的配置)

# 3. 安装 sqlx-cli
cargo install sqlx-cli

# 4. 创建数据库
createdb entangle

# 5. 运行迁移
sqlx migrate run

# 6. 初始化 TailwindCSS
npm init -y
npm install -D tailwindcss
npx tailwindcss init
```

### 第 2-3 天：认证系统

- [ ] 用户注册 API
- [ ] 用户登录 API
- [ ] JWT 中间件
- [ ] 登录/注册页面

### 第 4-5 天：文档 CRUD

- [ ] 创建文档 API
- [ ] 文档列表 API
- [ ] 文档详情 API
- [ ] 文档列表页面

### 第 6-8 天：实时协作 (核心)

- [ ] WebSocket 服务
- [ ] yrs 集成
- [ ] 文档编辑器组件
- [ ] 多用户同步测试

### 第 9-10 天：评论与完善

- [ ] 评论功能
- [ ] 权限控制
- [ ] UI 优化

### 第 11-12 天：选做功能

- [ ] Markdown 支持
- [ ] 版本历史
- [ ] 导出功能

### 第 13-14 天：文档与答辩

- [ ] 课程报告
- [ ] 演示视频
- [ ] 代码复习

---

## 运行命令

```bash
# 开发模式
cargo leptos watch

# 构建发布版
cargo leptos build --release

# 运行
./target/release/entangle
```

---

## 快捷键参考

| 功能 | 命令 |
|------|------|
| 启动开发服务器 | `cargo leptos watch` |
| 数据库迁移 | `sqlx migrate run` |
| 创建迁移 | `sqlx migrate add <name>` |
| 检查代码 | `cargo clippy` |
| 格式化 | `cargo fmt` |
| 测试 | `cargo test` |

---

## 常见问题

### Q: openGauss 连接问题

openGauss 兼容 PostgreSQL 协议，使用 sqlx 的 postgres 特性即可：

```toml
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres"] }
```

### Q: WASM 编译错误

确保安装了 wasm32 target：

```bash
rustup target add wasm32-unknown-unknown
```

### Q: TailwindCSS 不生效

确保 `tailwind.config.js` 配置正确：

```js
module.exports = {
  content: ["./src/**/*.rs"],
  theme: {
    extend: {},
  },
  plugins: [],
}
```

---

## 下一步

1. 先跑通最小可用版本
2. 逐步添加功能
3. 保持代码简洁，避免过度设计
4. 优先保证核心功能 (实时协作) 的稳定性

**加油！有问题随时问。**
