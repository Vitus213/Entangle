# Entangle 编译与启动指南

> 本文档详细介绍如何编译、启动和调试 Entangle 项目

## 目录

- [环境要求](#环境要求)
- [快速启动](#快速启动)
- [后端编译与启动](#后端编译与启动)
- [前端编译与启动](#前端编译与启动)
- [生产环境部署](#生产环境部署)
- [常见问题](#常见问题)

---

## 环境要求

### 必需工具

| 工具 | 版本要求 | 用途 |
|------|----------|------|
| Rust | 1.75+ | 后端和前端编译 |
| PostgreSQL | 14+ | 数据库 |
| trunk | 0.18+ | WASM 打包工具 |
| wasm32-unknown-unknown | - | WASM 编译目标 |

### 安装依赖

```bash
# 1. 安装 Rust (如果未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. 添加 WASM 编译目标
rustup target add wasm32-unknown-unknown

# 3. 安装 trunk
cargo install trunk

# 4. 验证安装
rustc --version
trunk --version
```

### 使用 Nix (推荐)

项目提供 Nix Flakes 配置，可一键配置开发环境：

```bash
# 进入开发环境
nix develop

# 或使用 direnv 自动加载
direnv allow
```

---

## 快速启动

### 一键启动脚本

```bash
# 启动所有服务 (数据库 + 后端 + 前端)
./scripts/start_all.sh

# 或手动分步启动
./scripts/start_backend.sh &
./scripts/start_frontend.sh &
```

### 手动启动

```bash
# 终端 1: 启动后端 API (端口 3000)
cargo run --bin entangle-api

# 终端 2: 启动前端开发服务器 (端口 8080)
cd frontend && trunk serve
```

### 访问地址

| 服务 | 地址 | 说明 |
|------|------|------|
| 前端 | http://localhost:8080 | Leptos Web 应用 |
| 后端 API | http://localhost:3000 | REST API |
| WebSocket | ws://localhost:3000/ws | 实时协作 |

---

## 后端编译与启动

### 环境变量配置

创建 `.env` 文件：

```bash
# 数据库连接
DATABASE_URL=postgres://entangle:your_password@localhost:5432/postgres

# JWT 密钥
JWT_SECRET=your-secret-key-at-least-32-characters

# 服务器配置
SERVER_HOST=127.0.0.1
SERVER_PORT=3000

# 日志级别
RUST_LOG=info,entangle_api=debug
```

### 数据库初始化

```bash
# 1. 创建数据库用户和数据库
sudo -u postgres psql -c "CREATE USER entangle WITH PASSWORD 'your_password';"
sudo -u postgres psql -c "CREATE DATABASE postgres OWNER entangle;"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE postgres TO entangle;"

# 2. 运行数据库迁移
sqlx migrate run

# 或使用 cargo-sqlx
cargo sqlx migrate run
```

### 编译命令

```bash
# 开发模式编译 (快速，包含调试信息)
cargo build --bin entangle-api

# 发布模式编译 (优化，适合生产)
cargo build --release --bin entangle-api

# 检查编译错误 (不生成二进制)
cargo check --bin entangle-api

# 编译所有 crate
cargo build --workspace
```

### 启动命令

```bash
# 开发模式启动
cargo run --bin entangle-api

# 发布模式启动
cargo run --release --bin entangle-api

# 直接运行编译后的二进制
./target/debug/entangle-api
./target/release/entangle-api

# 后台运行
nohup cargo run --bin entangle-api > backend.log 2>&1 &
```

### 热重载开发

使用 `cargo-watch` 实现代码修改后自动重启：

```bash
# 安装 cargo-watch
cargo install cargo-watch

# 启动热重载
cargo watch -x 'run --bin entangle-api'
```

---

## 前端编译与启动

### 项目结构

```
frontend/
├── Cargo.toml      # Rust 依赖配置
├── Trunk.toml      # Trunk 构建配置
├── index.html      # HTML 模板
├── src/
│   ├── main.rs     # WASM 入口点
│   └── lib.rs      # 应用组件
└── dist/           # 构建输出目录
```

### Trunk 配置 (Trunk.toml)

```toml
[build]
# 输出目录
dist = "dist"

[serve]
# 开发服务器配置
address = "0.0.0.0"
port = 8080
open = false
```

### 编译命令

```bash
cd frontend

# 开发模式编译
trunk build

# 发布模式编译 (优化 WASM 体积)
trunk build --release

# 清理构建缓存
trunk clean
cargo clean
```

### 启动开发服务器

```bash
cd frontend

# 启动开发服务器 (支持热重载)
trunk serve

# 指定端口
trunk serve --port 8080

# 禁止自动打开浏览器
trunk serve --open false

# 监听所有网络接口
trunk serve --address 0.0.0.0
```

### 构建产物

编译后的文件位于 `frontend/dist/` 目录：

```
dist/
├── index.html                              # 处理后的 HTML
├── entangle-frontend-{hash}.js             # WASM 绑定 JS
└── entangle-frontend-{hash}_bg.wasm        # WebAssembly 二进制
```

---

## 生产环境部署

### 后端部署

```bash
# 1. 编译发布版本
cargo build --release --bin entangle-api

# 2. 复制二进制和配置
cp target/release/entangle-api /opt/entangle/
cp .env.production /opt/entangle/.env

# 3. 使用 systemd 管理服务
sudo systemctl enable entangle-api
sudo systemctl start entangle-api
```

### 前端部署

```bash
# 1. 编译发布版本
cd frontend
trunk build --release

# 2. 部署静态文件到 Web 服务器
cp -r dist/* /var/www/entangle/

# 3. Nginx 配置示例
# 见下方 Nginx 配置
```

### Nginx 配置示例

```nginx
server {
    listen 80;
    server_name entangle.example.com;

    # 前端静态文件
    location / {
        root /var/www/entangle;
        try_files $uri $uri/ /index.html;

        # WASM MIME 类型
        types {
            application/wasm wasm;
        }
    }

    # 后端 API 代理
    location /api/ {
        proxy_pass http://127.0.0.1:3000/api/;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
    }

    # WebSocket 代理
    location /ws/ {
        proxy_pass http://127.0.0.1:3000/ws/;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

---

## 常见问题

### 1. trunk serve 报错 "target not found"

**问题**: Trunk.toml 中配置错误

**解决**:
```toml
# 错误配置
target = "wasm32-unknown-unknown"

# 正确配置
[build]
dist = "dist"
```

### 2. WASM 模块加载后页面空白

**原因**: leptos_router 缺少 CSR 特性

**解决**: 修改 `frontend/Cargo.toml`:
```toml
leptos = { version = "0.6", features = ["csr"] }
leptos_router = { version = "0.6", features = ["csr"] }
leptos_meta = { version = "0.6", features = ["csr"] }
```

### 3. "unreachable executed" 错误

**原因**: Router 在 CSR 模式下需要 base 配置

**解决**:
1. HTML 中添加: `<base href="/">`
2. Router 组件添加: `<Router base="/">`

### 4. API 请求 403 错误

**原因**: 用户权限不足

**解决**:
```sql
-- 更新用户角色为 editor
UPDATE users SET role_id = '00000000-0000-0000-0000-000000000002'
WHERE email = 'your@email.com';
```

### 5. 数据库连接失败

**检查**:
```bash
# 测试数据库连接
psql -h localhost -U entangle -d postgres -c "SELECT 1;"

# 检查 PostgreSQL 服务状态
systemctl status postgresql
```

### 6. WASM 编译目标未安装

**解决**:
```bash
rustup target add wasm32-unknown-unknown
```

---

## 下一步

- [调试指南](DEBUG_GUIDE.md) - 详细的调试方法
- [API 文档](API_REFERENCE.md) - 后端 API 参考
- [前端开发指南](FRONTEND_GUIDE.md) - Leptos 开发说明
