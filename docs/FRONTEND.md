# Entangle 前端开发文档

> 版本: 1.0.0 | 最后更新: 2026-01-01 | 状态: 完成

---

## 目录

- [概述](#概述)
- [技术栈](#技术栈)
- [环境配置](#环境配置)
- [开发工作流](#开发工作流)
- [项目结构](#项目结构)
- [API 集成](#api-集成)
- [页面组件](#页面组件)
- [构建部署](#构建部署)
- [调试技巧](#调试技巧)
- [性能优化](#性能优化)
- [常见问题](#常见问题)

---

## 概述

Entangle 前端使用 Leptos 框架构建，编译为 WebAssembly 在浏览器中运行，提供高性能的响应式用户界面。

### 核心特性

| 特性 | 说明 |
|------|------|
| 响应式框架 | Leptos 0.6 细粒度响应式系统 |
| WebAssembly | 高性能浏览器运行时 |
| 客户端路由 | Leptos Router 实现 SPA |
| 热重载 | Trunk 开发服务器支持 |

---

## 技术栈

### 核心依赖

```toml
[dependencies]
leptos = { version = "0.6", features = ["csr"] }
leptos_router = { version = "0.6", features = ["csr"] }
leptos_meta = { version = "0.6", features = ["csr"] }
gloo-net = "0.5"           # HTTP 客户端
web-sys = "0.3"            # Web API 绑定
console_error_panic_hook = "0.1"  # 错误处理
```

### 构建工具

| 工具 | 版本 | 用途 |
|------|------|------|
| Trunk | 0.18+ | WASM 打包和开发服务器 |
| wasm-bindgen | 自动 | Rust/JS 互操作 |
| wasm-opt | 自动 | WASM 优化 |

---

## 环境配置

### 方式 1: Nix (推荐)

项目提供完整的 Nix Flakes 配置：

```bash
# 进入开发环境
nix develop

# 或使用 direnv 自动加载
direnv allow .
```

Nix 环境包含:
- Rust 工具链 + wasm32-unknown-unknown target
- trunk
- wasm-bindgen-cli
- binaryen (wasm-opt)

进入环境后显示：
```
🚀 Entangle Development Environment
==================================
Rust: rustc 1.x.x
Trunk: trunk 0.x.x
📦 WASM Target: wasm32-unknown-unknown ✅
```

### 方式 2: 手动安装

```bash
# 1. 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. 添加 WASM target
rustup target add wasm32-unknown-unknown

# 3. 安装 Trunk
cargo install trunk

# 4. 验证安装
rustc --version
trunk --version
rustup target list | grep wasm32
```

---

## 开发工作流

### 全栈开发

```bash
# 终端 1: 启动后端 (端口 3000)
cargo run --bin entangle-api

# 终端 2: 启动前端 (端口 8080)
cd frontend && trunk serve
```

访问: http://localhost:8080

### 前端独立开发

```bash
cd frontend

# 开发模式 - 热重载
trunk serve

# 自定义配置
trunk serve --port 3001           # 自定义端口
trunk serve --address 0.0.0.0     # 监听所有接口
trunk serve --open                # 自动打开浏览器
```

### 构建命令

```bash
cd frontend

# 开发构建
trunk build

# 生产构建（优化）
trunk build --release

# 清理
trunk clean
cargo clean
```

---

## 项目结构

```
frontend/
├── Cargo.toml          # Rust 依赖配置
├── Trunk.toml          # Trunk 构建配置
├── index.html          # HTML 模板（含内联 CSS）
├── src/
│   ├── main.rs         # WASM 入口点
│   └── lib.rs          # 主要应用逻辑
│       ├── 类型定义    # Request/Response 结构
│       ├── API 函数    # HTTP 请求封装
│       ├── 页面组件    # 各页面实现
│       └── 路由配置    # Router 设置
└── dist/               # 构建输出（自动生成）
    ├── index.html
    ├── *.js            # WASM 加载器
    └── *.wasm          # 编译后的应用
```

### 配置文件

**Trunk.toml:**
```toml
[build]
target = "index.html"
dist = "dist"

[serve]
address = "127.0.0.1"
port = 8080

[watch]
ignore = ["dist", "target"]
```

**Cargo.toml (release 优化):**
```toml
[profile.release]
opt-level = 'z'       # 优化大小
lto = true            # 链接时优化
codegen-units = 1     # 更好的优化
```

---

## API 集成

### 基础配置

```rust
const API_BASE: &str = "http://127.0.0.1:3000";
```

### 请求示例

```rust
use gloo_net::http::Request;

// 登录请求
async fn login_api(email: String, password: String) -> Result<AuthResponse, String> {
    let response = Request::post(&format!("{}/api/auth/login", API_BASE))
        .json(&LoginRequest { email, password })
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("登录失败: {}", response.status()))
    }
}

// 带认证的请求
async fn fetch_documents(token: &str) -> Result<Vec<Document>, String> {
    Request::get(&format!("{}/api/documents/my", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}
```

### Token 管理

```rust
use web_sys::window;

// 保存 Token
fn save_token(token: &str) {
    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("token", token);
        }
    }
}

// 获取 Token
fn get_token() -> Option<String> {
    let window = window()?;
    let storage = window.local_storage().ok()??;
    storage.get_item("token").ok()?
}

// 清除 Token
fn clear_token() {
    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item("token");
        }
    }
}
```

---

## 页面组件

### 已实现页面

| 路由 | 组件 | 功能 |
|------|------|------|
| `/` | `LoginPage` | 用户登录 |
| `/register` | `RegisterPage` | 用户注册 |
| `/documents` | `DocumentsPage` | 文档列表、文件夹、标签管理 |
| `/editor/:id` | `EditorPage` | 文档编辑器 |

### 组件示例

```rust
#[component]
fn LoginPage() -> impl IntoView {
    let (email, set_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (error, set_error) = create_signal(None::<String>);
    let navigate = use_navigate();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let email_val = email.get();
        let password_val = password.get();

        spawn_local(async move {
            match login_api(email_val, password_val).await {
                Ok(response) => {
                    save_token(&response.token);
                    navigate("/documents", Default::default());
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    view! {
        <div class="auth-container">
            <h1>"登录"</h1>
            {move || error.get().map(|e| view! { <p class="error">{e}</p> })}
            <form on:submit=on_submit>
                <input
                    type="email"
                    placeholder="邮箱"
                    on:input=move |ev| set_email.set(event_target_value(&ev))
                />
                <input
                    type="password"
                    placeholder="密码"
                    on:input=move |ev| set_password.set(event_target_value(&ev))
                />
                <button type="submit">"登录"</button>
            </form>
        </div>
    }
}
```

### 路由配置

```rust
#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                <Route path="/" view=LoginPage />
                <Route path="/register" view=RegisterPage />
                <Route path="/documents" view=DocumentsPage />
                <Route path="/editor/:id" view=EditorPage />
            </Routes>
        </Router>
    }
}
```

---

## 构建部署

### 生产构建

```bash
cd frontend

# 构建优化版本
trunk build --release

# 输出目录
ls dist/
# -> index.html
# -> entangle_frontend.js
# -> entangle_frontend_bg.wasm
```

### 部署方式

构建后的 `dist/` 可部署到任何静态托管服务：

```bash
# 部署到服务器
rsync -avz dist/ user@server:/var/www/entangle/

# 或使用 S3/CDN
aws s3 sync dist/ s3://your-bucket/
```

### Nginx 配置

```nginx
server {
    listen 80;
    server_name entangle.example.com;

    root /var/www/entangle;
    index index.html;

    # SPA 路由支持
    location / {
        try_files $uri $uri/ /index.html;
    }

    # WASM MIME 类型
    location ~* \.wasm$ {
        types {
            application/wasm wasm;
        }
        add_header Cache-Control "public, max-age=31536000";
    }

    # API 代理
    location /api {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
    }

    # WebSocket 代理
    location /ws {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

### CI/CD 示例 (GitHub Actions)

```yaml
name: Build Frontend

on: [push]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Nix
        uses: cachix/install-nix-action@v22
        with:
          extra_nix_config: |
            experimental-features = nix-command flakes

      - name: Build Frontend
        run: |
          nix develop --command bash -c "
            cd frontend
            trunk build --release
          "

      - name: Upload Artifacts
        uses: actions/upload-artifact@v3
        with:
          name: frontend-dist
          path: frontend/dist/
```

---

## 调试技巧

### 浏览器控制台

```rust
use leptos::logging;

// 基本日志
logging::log!("Debug: {:?}", data);
logging::warn!("Warning!");
logging::error!("Error!");
```

### 使用 web_sys::console

```rust
use web_sys::console;

console::log_1(&"Hello from WASM".into());
console::log_2(&"Key:".into(), &value.into());
```

### 检查 WASM 状态

```javascript
// 浏览器控制台
console.log(localStorage.getItem('token'));  // 检查 Token
console.log(window.location.pathname);        // 检查路由
```

### 创建调试页面

```html
<!-- debug.html -->
<!DOCTYPE html>
<html>
<head><title>WASM Debug</title></head>
<body>
    <div id="status">加载中...</div>
    <script type="module">
        try {
            const wasmModule = await import('/entangle-frontend.js');
            await wasmModule.default();
            document.getElementById('status').textContent = '✓ WASM 加载成功';
        } catch (error) {
            document.getElementById('status').textContent = '✗ ' + error.message;
        }
    </script>
</body>
</html>
```

---

## 性能优化

### 构建优化

已在 `Cargo.toml` 配置：

```toml
[profile.release]
opt-level = 'z'      # 优化大小
lto = true           # 链接时优化
codegen-units = 1    # 更好的优化
```

### 产物大小

| 阶段 | 大小 |
|------|------|
| WASM (release) | ~200-500 KB |
| JS 胶水代码 | ~10-20 KB |
| Gzip 后 | ~60-150 KB |

### 服务器优化

```nginx
# 启用 Brotli 压缩
brotli on;
brotli_types application/wasm application/javascript;

# 缓存策略
location ~* \.(wasm|js)$ {
    add_header Cache-Control "public, max-age=31536000";
}
```

### 构建时间

| 模式 | 时间 |
|------|------|
| 开发构建 | ~10-30 秒 |
| 生产构建 | ~30-60 秒 |
| 热重载 | <2 秒 |

---

## 常见问题

### Q: trunk serve 启动失败？

检查端口占用：
```bash
lsof -i :8080
trunk serve --port 3001  # 使用其他端口
```

### Q: WASM 文件很大？

确保使用 release 模式：
```bash
trunk build --release
```

### Q: API 请求跨域错误？

确保后端 CORS 配置：
```rust
CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any)
```

### Q: 页面刷新后 404？

配置服务器返回 index.html：
```nginx
location / {
    try_files $uri $uri/ /index.html;
}
```

### Q: "unreachable executed" 错误？

确保 Router 配置正确：
```rust
<Router base="/">
```

确保 Cargo.toml 有 CSR 特性：
```toml
leptos_router = { version = "0.6", features = ["csr"] }
```

### Q: WASM target 未安装？

```bash
rustup target add wasm32-unknown-unknown
# 或使用 Nix 环境
nix develop
```

---

## 待实现功能

- [ ] WebSocket 实时协作集成
- [ ] 富文本编辑器 (TipTap)
- [ ] 文档版本历史 UI
- [ ] 协作者管理界面
- [ ] 评论和通知系统
- [ ] 响应式移动端适配
- [ ] 离线支持 (Service Worker)

---

## 参考资源

- [Leptos 官方文档](https://leptos.dev/)
- [Leptos 示例](https://github.com/leptos-rs/leptos/tree/main/examples)
- [Trunk 文档](https://trunkrs.dev/)
- [Rust WASM 指南](https://rustwasm.github.io/docs/book/)

---

*最后更新: 2026-01-01*
