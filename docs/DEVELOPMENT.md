# Entangle 开发者指南

> 版本: 1.0.0 | 最后更新: 2026-01-01

---

## 目录

- [环境要求](#环境要求)
- [快速启动](#快速启动)
- [后端开发](#后端开发)
- [前端开发](#前端开发)
- [数据库管理](#数据库管理)
- [调试技巧](#调试技巧)
- [生产部署](#生产部署)
- [常见问题](#常见问题)

---

## 环境要求

### 必需工具

| 工具 | 版本 | 用途 |
|------|------|------|
| Rust | 1.75+ | 后端和前端编译 |
| PostgreSQL | 14+ | 数据库 |
| trunk | 0.18+ | WASM 打包工具 |
| wasm32-unknown-unknown | - | WASM 编译目标 |

### 使用 Nix (推荐)

项目提供完整的 Nix Flakes 配置：

```bash
# 进入开发环境（自动配置所有依赖）
nix develop

# 或使用 direnv 自动加载
direnv allow .
```

### 手动安装

```bash
# 1. 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. 添加 WASM 编译目标
rustup target add wasm32-unknown-unknown

# 3. 安装 trunk
cargo install trunk

# 4. 验证安装
rustc --version && trunk --version
```

---

## 快速启动

### 一键启动

```bash
# 终端 1: 启动后端 API (端口 3000)
cargo run --bin entangle-api

# 终端 2: 启动前端开发服务器 (端口 8080)
cd frontend && trunk serve
```

### 访问地址

| 服务 | 地址 |
|------|------|
| 前端 | http://localhost:8080 |
| 后端 API | http://localhost:3000 |
| WebSocket | ws://localhost:3000/ws |
| 健康检查 | http://localhost:3000/health |

---

## 后端开发

### 环境变量配置

创建 `.env` 文件：

```bash
# 数据库连接
DATABASE_URL=postgres://entangle:your_password@localhost:5432/postgres

# JWT 密钥 (至少32字符)
JWT_SECRET=your-secret-key-at-least-32-characters

# 服务器配置
SERVER_HOST=127.0.0.1
SERVER_PORT=3000

# 日志级别
RUST_LOG=info,entangle_api=debug
```

### 编译命令

```bash
# 开发模式（快速，包含调试信息）
cargo build --bin entangle-api

# 发布模式（优化，适合生产）
cargo build --release --bin entangle-api

# 检查编译错误
cargo check --bin entangle-api

# 编译所有 crate
cargo build --workspace
```

### 启动命令

```bash
# 开发模式
cargo run --bin entangle-api

# 发布模式
cargo run --release --bin entangle-api

# 直接运行二进制
./target/debug/entangle-api
./target/release/entangle-api
```

### 热重载开发

```bash
# 安装 cargo-watch
cargo install cargo-watch

# 代码修改后自动重启
cargo watch -x 'run --bin entangle-api'
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_create_document

# 显示 println! 输出
cargo test -- --nocapture

# 运行特定 crate 的测试
cargo test -p entangle-collab
```

---

## 前端开发

### 启动开发服务器

```bash
cd frontend

# 开发模式（热重载）
trunk serve

# 自定义配置
trunk serve --port 3001           # 自定义端口
trunk serve --address 0.0.0.0     # 监听所有接口
```

### 构建命令

```bash
cd frontend

# 开发构建
trunk build

# 生产构建（优化 WASM 体积）
trunk build --release

# 清理构建缓存
trunk clean && cargo clean
```

### 构建产物

```
frontend/dist/
├── index.html                    # 处理后的 HTML
├── entangle-frontend-{hash}.js   # WASM 绑定
└── entangle-frontend-{hash}_bg.wasm  # WebAssembly 二进制
```

---

## 数据库管理

### 初始化数据库

```bash
# 创建用户和数据库
sudo -u postgres psql << EOF
CREATE USER entangle WITH PASSWORD 'your_password';
CREATE DATABASE postgres OWNER entangle;
GRANT ALL PRIVILEGES ON DATABASE postgres TO entangle;
EOF

# 运行迁移
sqlx migrate run
```

### 常用查询

```sql
-- 查看所有用户
SELECT id, email, nickname, role_id FROM users;

-- 查看用户权限
SELECT u.email, r.name as role
FROM users u
JOIN roles r ON u.role_id = r.id;

-- 查看所有文档
SELECT id, title, owner_id, is_public FROM documents;

-- 升级用户角色为 editor
UPDATE users
SET role_id = '00000000-0000-0000-0000-000000000002'
WHERE email = 'your@email.com';
```

### 查看 SQL 日志

```bash
# 显示执行的 SQL 语句
RUST_LOG=sqlx::query=debug cargo run --bin entangle-api

# 详细追踪
RUST_LOG=sqlx=trace cargo run --bin entangle-api
```

---

## 调试技巧

### 日志配置

通过 `RUST_LOG` 环境变量控制：

```bash
# 基础日志
RUST_LOG=info cargo run --bin entangle-api

# 详细调试
RUST_LOG=debug cargo run --bin entangle-api

# 特定模块
RUST_LOG=entangle_api=debug,entangle_db=trace cargo run

# SQL 查询
RUST_LOG=sqlx=debug cargo run --bin entangle-api
```

| 级别 | 用途 |
|------|------|
| `error` | 仅错误 |
| `warn` | 警告和错误 |
| `info` | 一般信息（默认） |
| `debug` | 调试信息 |
| `trace` | 详细追踪 |

### 后端调试

```rust
// 使用 dbg! 宏
let result = dbg!(some_function());

// 使用 tracing
use tracing::{debug, info, error};
info!("处理请求: {}", request_id);
debug!(user_id = %user.id, "用户登录");
error!(?error, "数据库错误");
```

### 前端调试

```rust
use leptos::logging;

logging::log!("Debug: {:?}", data);
logging::warn!("Warning!");
logging::error!("Error!");
```

浏览器控制台检查：
```javascript
localStorage.getItem('token')    // 检查 Token
window.location.pathname         // 检查路由
```

### API 测试

```bash
# 健康检查
curl http://localhost:3000/health

# 用户登录
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"demo@example.com","password":"demo123"}'

# 带 Token 请求
TOKEN="your_jwt_token"
curl http://localhost:3000/api/documents/my \
  -H "Authorization: Bearer $TOKEN"

# 创建文档
curl -X POST http://localhost:3000/api/documents \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title":"测试","content":"内容","is_public":false}'
```

### VS Code 调试配置

创建 `.vscode/launch.json`:

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug entangle-api",
            "cargo": {
                "args": ["build", "--bin=entangle-api"],
                "filter": {"name": "entangle-api", "kind": "bin"}
            },
            "env": {
                "RUST_LOG": "debug",
                "DATABASE_URL": "postgres://entangle:password@localhost:5432/postgres"
            }
        }
    ]
}
```

---

## 生产部署

### 后端部署

```bash
# 1. 编译发布版本
cargo build --release --bin entangle-api

# 2. 复制二进制和配置
cp target/release/entangle-api /opt/entangle/
cp .env.production /opt/entangle/.env

# 3. 使用 systemd 管理
sudo systemctl enable entangle-api
sudo systemctl start entangle-api
```

### 前端部署

```bash
# 1. 编译发布版本
cd frontend && trunk build --release

# 2. 部署静态文件
cp -r dist/* /var/www/entangle/
```

### Nginx 配置

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

### 数据库连接失败

```bash
# 检查 PostgreSQL 服务
systemctl status postgresql

# 测试连接
psql -h localhost -U entangle -d postgres -c "SELECT 1;"

# 检查端口
nc -zv localhost 5432
```

### trunk serve 报错

```bash
# 检查端口占用
lsof -i :8080

# 使用其他端口
trunk serve --port 3001
```

### WASM 加载失败

检查 Nginx MIME 类型配置：
```nginx
types {
    application/wasm wasm;
}
```

### 403 权限错误

```sql
-- 升级用户角色
UPDATE users
SET role_id = '00000000-0000-0000-0000-000000000002'
WHERE email = 'your@email.com';
```

### "unreachable executed" 错误

确保 Cargo.toml 配置正确：
```toml
leptos_router = { version = "0.6", features = ["csr"] }
```

确保 Router 有 base 属性：
```rust
<Router base="/">
```

### WASM target 未安装

```bash
rustup target add wasm32-unknown-unknown
# 或使用 Nix
nix develop
```

### API 响应结构不匹配

使用 curl 查看实际响应，对比前端结构体定义：
```bash
curl http://localhost:3000/api/documents/my \
  -H "Authorization: Bearer $TOKEN" | jq
```

---

## 调试工具推荐

| 工具 | 用途 |
|------|------|
| rust-analyzer | IDE Rust 支持 |
| CodeLLDB | VS Code 调试器 |
| cargo-watch | 代码热重载 |
| cargo-expand | 查看宏展开 |
| sqlx-cli | 数据库迁移工具 |
| httpie | 友好的 HTTP 客户端 |
| pgcli | PostgreSQL 交互式客户端 |

---

## 相关文档

- [前端开发文档](FRONTEND.md) - Leptos/WASM 详细开发指南
- [认证系统文档](AUTH_README.md) - JWT/RBAC 认证说明
- [测试文档](TESTING.md) - 测试用例和脚本
- [项目进度](PROGRESS.md) - 开发进度追踪

---

*最后更新: 2026-01-01*
