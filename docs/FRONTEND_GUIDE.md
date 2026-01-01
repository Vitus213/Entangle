# Entangle 前端开发指南

> Leptos + WebAssembly 前端应用

---

## 📋 目录

1. [技术栈](#技术栈)
2. [开发环境设置](#开发环境设置)
3. [开发工作流](#开发工作流)
4. [项目结构](#项目结构)
5. [API 集成](#api-集成)
6. [构建部署](#构建部署)

---

## 技术栈

### 核心框架
- **Leptos 0.6** - Rust 全栈响应式 Web 框架
- **WebAssembly** - 高性能浏览器运行时
- **Leptos Router** - 客户端路由
- **Leptos Meta** - 元数据管理

### 构建工具
- **Trunk** - WASM 应用打包器和开发服务器
- **wasm-bindgen** - Rust/JS 互操作
- **wasm-opt** - WASM 优化器

### HTTP 客户端
- **gloo-net** - WASM 友好的 HTTP 客户端

---

## 开发环境设置

### 方式 1: 使用 Nix (推荐)

所有依赖已配置在 `flake.nix` 中：

```bash
# 进入 Nix 开发环境
nix develop

# 或使用 direnv 自动加载
direnv allow .
```

Nix 环境自动包含:
- ✅ Rust 工具链 + wasm32 target
- ✅ trunk
- ✅ wasm-bindgen-cli
- ✅ binaryen (wasm-opt)

### 方式 2: 手动安装

```bash
# 1. 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. 添加 wasm32 target
rustup target add wasm32-unknown-unknown

# 3. 安装 trunk
cargo install trunk

# 4. 安装 wasm-bindgen-cli (可选，trunk 会自动处理)
cargo install wasm-bindgen-cli
```

---

## 开发工作流

### 启动开发服务器

**完整全栈开发：**

```bash
# Terminal 1: 启动后端 (端口 3000)
just dev

# Terminal 2: 启动前端开发服务器 (端口 8080)
cd frontend
trunk serve
```

然后访问: **http://localhost:8080**

### 前端独立开发

```bash
cd frontend

# 开发模式 - 热重载
trunk serve

# 监听所有接口（用于网络访问）
trunk serve --address 0.0.0.0

# 自定义端口
trunk serve --port 3001
```

### 构建生产版本

```bash
cd frontend

# 生产构建（优化）
trunk build --release

# 输出在: frontend/dist/
```

---

## 项目结构

```
frontend/
├── Cargo.toml          # 依赖配置
├── Trunk.toml          # Trunk 构建配置
├── index.html          # HTML 模板 + 内联 CSS
├── src/
│   ├── main.rs        # 应用入口
│   └── lib.rs         # 主要应用逻辑
│       ├── 类型定义 (Request/Response)
│       ├── API 客户端函数
│       ├── 页面组件
│       └── 路由配置
└── dist/              # 构建输出（自动生成）
    ├── index.html
    ├── *.wasm
    └── *.js
```

---

## API 集成

### API 基础配置

```rust
const API_BASE: &str = "http://127.0.0.1:3000";
```

### 示例 API 调用

```rust
// 登录
async fn login_api(email: String, password: String) -> Result<AuthResponse, String> {
    let response = Request::post(&format!("{}/api/auth/login", API_BASE))
        .json(&LoginRequest { email, password })?
        .send()
        .await?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("登录失败: {}", response.status()))
    }
}
```

### Token 管理

```rust
// 保存 Token 到 LocalStorage
fn save_token(token: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("token", token);
        }
    }
}

// 获取 Token
fn get_token() -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    storage.get_item("token").ok()?
}
```

---

## 页面组件

### 当前已实现页面

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
    let navigate = use_navigate();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        // API 调用...
    };

    view! {
        <div class="auth-container">
            <h1>"登录"</h1>
            <form on:submit=on_submit>
                // 表单内容...
            </form>
        </div>
    }
}
```

---

## 构建部署

### 开发构建

```bash
cd frontend
trunk build

# 输出在 dist/，未优化
```

### 生产构建

```bash
cd frontend
trunk build --release

# 输出优化后的 WASM
# - 代码压缩
# - wasm-opt 优化
# - 移除调试信息
```

### 部署静态文件

构建后的 `dist/` 目录包含所有需要的文件:

```
dist/
├── index.html           # HTML 入口
├── entangle_frontend.js # WASM 加载器
└── entangle_frontend_bg.wasm  # 编译后的应用
```

可以部署到任何静态托管服务:
- Nginx
- Apache
- Cloudflare Pages
- Vercel
- Netlify

**Nginx 配置示例：**

```nginx
server {
    listen 80;
    server_name your-domain.com;
    root /path/to/entangle/frontend/dist;

    location / {
        try_files $uri $uri/ /index.html;
    }

    # WASM 文件 MIME 类型
    location ~* \.wasm$ {
        types {
            application/wasm wasm;
        }
        add_header Cache-Control "public, max-age=31536000";
    }

    # API 代理（如果需要）
    location /api {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
    }
}
```

---

## 性能优化

### WASM 大小优化

已在 `Cargo.toml` 中配置:

```toml
[profile.release]
opt-level = 'z'      # 优化大小
lto = true           # 链接时优化
codegen-units = 1    # 更好的优化
```

### 额外优化建议

1. **启用 brotli 压缩** (服务器端):
   ```nginx
   brotli on;
   brotli_types application/wasm;
   ```

2. **使用 CDN** 缓存静态资源

3. **代码分割** (未来):
   - Leptos 支持服务端渲染 (SSR)
   - 可以按路由分割代码

---

## 调试技巧

### 浏览器控制台

```rust
// 使用 leptos::logging
leptos::logging::log!("Debug message: {:?}", data);
leptos::logging::warn!("Warning!");
leptos::logging::error!("Error occurred!");
```

### 开发工具

- **Chrome DevTools**: 支持 WASM 调试
- **Firefox Developer Tools**: 更好的 WASM 支持
- **Trunk 日志**: 查看构建和重载信息

---

## 常见问题

### Q: trunk serve 启动失败？

检查端口是否被占用:
```bash
lsof -i :8080
# 或更换端口
trunk serve --port 3001
```

### Q: 构建后 WASM 文件很大？

确保使用 `--release` 模式:
```bash
trunk build --release
```

### Q: API 请求跨域错误？

确保后端 CORS 配置正确:
```rust
// 在 flake.nix 或 .env 中
CORS_ALLOWED_ORIGINS=http://localhost:8080
```

### Q: 页面刷新后 404？

使用 SPA 路由需要配置服务器:
- 所有路径都返回 `index.html`
- Trunk 开发服务器自动处理

---

## 下一步开发

### 待实现功能

- [ ] 文档搜索功能
- [ ] WebSocket 实时协作
- [ ] 富文本编辑器集成 (TipTap)
- [ ] 文档版本历史 UI
- [ ] 协作者管理界面
- [ ] 评论和通知系统
- [ ] 文件上传和附件
- [ ] 响应式移动端适配

### 建议优化

- [ ] 添加加载骨架屏
- [ ] 实现页面过渡动画
- [ ] 添加错误边界
- [ ] 实现离线支持 (Service Worker)
- [ ] 添加单元测试
- [ ] 实现 E2E 测试

---

## 参考资源

- [Leptos 官方文档](https://leptos.dev/)
- [Leptos 示例](https://github.com/leptos-rs/leptos/tree/main/examples)
- [Trunk 文档](https://trunkrs.dev/)
- [Rust WASM 指南](https://rustwasm.github.io/docs/book/)

---

*最后更新: 2026-01-01*
