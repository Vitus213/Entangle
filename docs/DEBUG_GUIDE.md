# Entangle 调试指南

> 本文档介绍如何调试 Entangle 项目的前端和后端

## 目录

- [日志配置](#日志配置)
- [后端调试](#后端调试)
- [前端调试](#前端调试)
- [数据库调试](#数据库调试)
- [网络请求调试](#网络请求调试)
- [常见错误诊断](#常见错误诊断)

---

## 日志配置

### 后端日志级别

通过环境变量 `RUST_LOG` 控制：

```bash
# 基础日志
RUST_LOG=info cargo run --bin entangle-api

# 详细调试日志
RUST_LOG=debug cargo run --bin entangle-api

# 特定模块日志
RUST_LOG=entangle_api=debug,entangle_db=trace cargo run --bin entangle-api

# SQL 查询日志
RUST_LOG=sqlx=debug cargo run --bin entangle-api

# 全部追踪
RUST_LOG=trace cargo run --bin entangle-api
```

### 日志级别说明

| 级别 | 用途 |
|------|------|
| `error` | 仅显示错误 |
| `warn` | 警告和错误 |
| `info` | 一般信息 (默认) |
| `debug` | 调试信息 |
| `trace` | 详细追踪 |

### 前端日志

前端使用 `console_error_panic_hook` 捕获 panic，错误会显示在浏览器控制台：

```rust
// main.rs 中已配置
console_error_panic_hook::set_once();
```

---

## 后端调试

### 使用 println! 调试

```rust
// 快速调试
println!("DEBUG: user_id = {:?}", user_id);
println!("DEBUG: request body = {:#?}", request_body);

// 带文件和行号
println!("[{}:{}] value = {:?}", file!(), line!(), value);
```

### 使用 dbg! 宏

```rust
// 自动打印变量名、值和位置
let result = dbg!(some_function());

// 链式调试
let value = dbg!(dbg!(x) + dbg!(y));
```

### 使用 tracing 日志

```rust
use tracing::{debug, info, warn, error, instrument};

// 基本日志
info!("处理请求: {}", request_id);
debug!(user_id = %user.id, "用户登录");
error!(?error, "数据库错误");

// 函数追踪
#[instrument(skip(pool))]
async fn create_document(pool: &PgPool, doc: CreateDocument) -> Result<Document> {
    debug!("创建文档: {:?}", doc.title);
    // ...
}
```

### IDE 调试 (VS Code)

1. 安装 `rust-analyzer` 和 `CodeLLDB` 扩展

2. 创建 `.vscode/launch.json`:

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug entangle-api",
            "cargo": {
                "args": ["build", "--bin=entangle-api", "--package=entangle-api"],
                "filter": {
                    "name": "entangle-api",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}",
            "env": {
                "RUST_LOG": "debug",
                "DATABASE_URL": "postgres://entangle:password@localhost:5432/postgres"
            }
        }
    ]
}
```

3. 设置断点并按 F5 启动调试

### 单元测试调试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_create_document

# 显示 println! 输出
cargo test -- --nocapture

# 单线程运行 (便于调试)
cargo test -- --test-threads=1

# 运行特定 crate 的测试
cargo test -p entangle-db
```

---

## 前端调试

### 浏览器开发者工具

1. **打开控制台**: `F12` 或 `Ctrl+Shift+I`
2. **查看 Console 标签**: WASM panic 和 log 输出
3. **查看 Network 标签**: API 请求和响应
4. **查看 Application 标签**: LocalStorage (token 存储)

### Leptos 调试日志

```rust
use leptos::logging;

// 在组件中添加日志
#[component]
fn MyComponent() -> impl IntoView {
    logging::log!("组件渲染");

    create_effect(move |_| {
        logging::log!("Effect 执行");
    });

    view! { <div>"Hello"</div> }
}
```

### 使用 web_sys::console

```rust
use web_sys::console;

// 控制台日志
console::log_1(&"Hello from WASM".into());
console::log_2(&"Key:".into(), &value.into());

// 格式化对象
console::log_1(&format!("{:?}", my_struct).into());
```

### 创建调试页面

创建 `frontend/debug.html` 测试 WASM 加载：

```html
<!DOCTYPE html>
<html>
<head>
    <title>WASM Debug</title>
</head>
<body>
    <h1>WASM 调试页面</h1>
    <div id="status">加载中...</div>

    <script type="module">
        const status = document.getElementById('status');

        try {
            status.textContent = '✓ JavaScript 正常';

            // 动态加载 WASM
            const wasmModule = await import('/entangle-frontend.js');
            await wasmModule.default();

            status.textContent = '✓ WASM 加载成功';
        } catch (error) {
            status.textContent = '✗ 错误: ' + error.message;
            console.error('WASM 加载失败:', error);
        }
    </script>
</body>
</html>
```

### 常见前端问题排查

#### 1. 检查 WASM 是否加载

```javascript
// 在浏览器控制台执行
console.log(window.wasmBindings);  // 应该显示 WASM 导出对象
```

#### 2. 检查 Token 是否保存

```javascript
// 在浏览器控制台执行
console.log(localStorage.getItem('token'));
```

#### 3. 检查路由状态

```javascript
// 查看当前 URL
console.log(window.location.href);
console.log(window.location.pathname);
```

---

## 数据库调试

### 查看表结构

```sql
-- 列出所有表
\dt

-- 查看表结构
\d users
\d documents

-- 查看索引
\di
```

### 常用查询

```sql
-- 查看所有用户
SELECT id, email, nickname, role_id FROM users;

-- 查看用户权限
SELECT u.email, r.name as role, p.name as permission
FROM users u
JOIN roles r ON u.role_id = r.id
JOIN role_permissions rp ON r.id = rp.role_id
JOIN permissions p ON rp.permission_id = p.id
WHERE u.email = 'demo@example.com';

-- 查看所有文档
SELECT id, title, owner_id, is_public, created_at FROM documents;

-- 查看文档协作者
SELECT d.title, u.email, dc.permission_level
FROM document_collaborators dc
JOIN documents d ON dc.document_id = d.id
JOIN users u ON dc.user_id = u.id;
```

### 检查数据完整性

```sql
-- 检查孤儿文档 (所有者不存在)
SELECT d.* FROM documents d
LEFT JOIN users u ON d.owner_id = u.id
WHERE u.id IS NULL;

-- 检查无效的角色引用
SELECT u.* FROM users u
LEFT JOIN roles r ON u.role_id = r.id
WHERE r.id IS NULL;
```

### 使用 sqlx 日志

```bash
# 显示执行的 SQL 语句
RUST_LOG=sqlx::query=debug cargo run --bin entangle-api

# 显示查询计划
RUST_LOG=sqlx=trace cargo run --bin entangle-api
```

---

## 网络请求调试

### 使用 curl 测试 API

```bash
# 健康检查
curl http://localhost:3000/health

# 用户登录
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"demo@example.com","password":"demo123"}'

# 携带 Token 请求
TOKEN="your_jwt_token"
curl http://localhost:3000/api/documents/my \
  -H "Authorization: Bearer $TOKEN"

# 创建文档
curl -X POST http://localhost:3000/api/documents \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title":"测试文档","content":"内容","is_public":false}'

# 查看详细响应
curl -v http://localhost:3000/api/documents/my \
  -H "Authorization: Bearer $TOKEN"
```

### 使用 HTTPie (更友好的 curl)

```bash
# 安装 HTTPie
pip install httpie

# 登录
http POST localhost:3000/api/auth/login \
  email=demo@example.com password=demo123

# 携带 Token
http localhost:3000/api/documents/my \
  "Authorization: Bearer $TOKEN"
```

### 检查 CORS 问题

```bash
# 测试预检请求
curl -X OPTIONS http://localhost:3000/api/documents \
  -H "Origin: http://localhost:8080" \
  -H "Access-Control-Request-Method: POST" \
  -v
```

---

## 常见错误诊断

### 错误: "unreachable executed"

**原因**: WASM panic，通常是 leptos_router 配置问题

**诊断步骤**:
1. 打开浏览器控制台查看完整错误栈
2. 检查 Cargo.toml 中 leptos_router 是否有 `csr` 特性
3. 检查 Router 组件是否有 `base` 属性

**解决**:
```toml
# Cargo.toml
leptos_router = { version = "0.6", features = ["csr"] }
```

```rust
// lib.rs
<Router base="/">
```

### 错误: "missing field `xxx`"

**原因**: 前端结构体与后端 API 响应不匹配

**诊断步骤**:
1. 使用 curl 查看实际 API 响应
2. 对比前端 struct 定义

**解决**: 更新前端结构体匹配后端响应

### 错误: 403 Forbidden

**原因**: 权限不足

**诊断步骤**:
```sql
-- 查看用户角色
SELECT u.email, r.name FROM users u
JOIN roles r ON u.role_id = r.id
WHERE u.email = 'your@email.com';

-- 查看角色权限
SELECT p.name FROM role_permissions rp
JOIN permissions p ON rp.permission_id = p.id
WHERE rp.role_id = 'role_uuid';
```

**解决**:
```sql
-- 升级用户角色为 editor
UPDATE users SET role_id = '00000000-0000-0000-0000-000000000002'
WHERE email = 'your@email.com';
```

### 错误: 数据库连接失败

**诊断步骤**:
```bash
# 1. 检查 PostgreSQL 服务
systemctl status postgresql

# 2. 检查连接字符串
echo $DATABASE_URL

# 3. 测试连接
psql "$DATABASE_URL" -c "SELECT 1;"

# 4. 检查网络
nc -zv localhost 5432
```

### 错误: WASM 加载失败

**诊断步骤**:
1. 检查 Network 标签，确认 .wasm 文件返回 200
2. 检查 MIME 类型是否为 `application/wasm`
3. 检查是否有 CORS 错误

**解决**:
```nginx
# Nginx 配置
types {
    application/wasm wasm;
}
```

---

## 调试工具推荐

| 工具 | 用途 |
|------|------|
| `rust-analyzer` | IDE Rust 支持 |
| `CodeLLDB` | VS Code 调试器 |
| `cargo-watch` | 代码热重载 |
| `cargo-expand` | 查看宏展开 |
| `sqlx-cli` | 数据库迁移工具 |
| `httpie` | HTTP 客户端 |
| `pgcli` | PostgreSQL 交互式客户端 |

---

## 相关文档

- [编译与启动指南](BUILD_AND_RUN.md)
- [API 参考文档](API_REFERENCE.md)
- [前端开发指南](FRONTEND_GUIDE.md)
