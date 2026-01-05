# Entangle 首次开发指南

> 第一次运行 `just dev` 的完整步骤

---

## 前提条件检查

### 1. 系统要求

- **操作系统**: NixOS (或其他 Linux 发行版)
- **内存**: 至少 4GB RAM
- **磁盘**: 至少 10GB 可用空间
- **网络**: 能访问 Docker Hub

### 2. 安装必要工具

```bash
# 检查 Nix 版本
nix --version

# 检查 Docker
docker --version
docker-compose --version

# 检查 just (如果没有，用 nix-shell 会自动安装)
just --version
```

---

## 第一步：进入开发环境

### 1. 进入 Nix 开发环境

```bash
cd /home/vitus/Documents/Entangle
nix-shell
```

进入环境后，你会看到：

```
🚀 Entangle Development Environment
==================================
Rust: rustc 1.xx.x
Cargo: cargo 1.xx.x
Trunk: trunk.x.x.x

📦 WASM Target: wasm32-unknown-unknown
   ✅ Already installed

...
```

### 2. 检查环境变量

```bash
# 检查数据库连接字符串
echo $DATABASE_URL
# 应该输出: postgres://entangle:Entangle@2024@localhost:5432/entangle_db

# 检查应用端口
echo $APP_PORT
# 应该输出: 3000
```

---

## 第二步：配置数据库

### 1. 启动数据库容器

```bash
# 启动 OpenGauss 和 Redis
just db-up
# 或
docker-compose -f docker-compose.dev.yml up -d
```

**重要**: OpenGauss 首次启动需要 30-40 秒来初始化！

### 2. 检查容器状态

```bash
docker ps | grep entangle
```

应该看到两个容器正在运行：

```
entangle-db      status: up (healthy)   ports: 0.0.0.0:5432->5432/tcp
entangle-redis   status: up (healthy)   ports: 0.0.0.0:6379->6379/tcp
```

### 3. 等待数据库完全启动

```bash
# 查看日志，确认数据库已就绪
just db-logs
# 或
docker-compose -f docker-compose.dev.yml logs -f opengauss
```

看到以下日志表示数据库已就绪：

```
[DEBUG] state [Starting]: start success database successfully
```

### 4. 创建数据库用户（首次运行必需）

OpenGauss 禁止初始用户远程连接，需要创建应用用户：

**方法一：使用 SQL 脚本（推荐）**

```bash
# 等待数据库完全启动后（约30秒）
docker exec -i entangle-db gsql -U gaussdb postgres << 'EOF'
-- 创建应用用户
CREATE USER entangle WITH PASSWORD 'Entangle@2024' SYSID 600;
ALTER USER entangle WITH CREATEDB CREATEROLE;

-- 创建数据库
CREATE DATABASE entangle_db OWNER entangle;

-- 授予权限
GRANT ALL PRIVILEGES ON DATABASE entangle_db TO entangle;

-- 显示创建的用户
\du entangle
EOF
```

**方法二：交互式创建**

```bash
# 进入数据库容器
docker exec -it entangle-db gsql -U gaussdb postgres

# 然后在 gsql 命令行中执行：
CREATE USER entangle WITH PASSWORD 'Entangle@2024' SYSID 600;
ALTER USER entangle WITH CREATEDB CREATEROLE;
CREATE DATABASE entangle_db OWNER entangle;
GRANT ALL PRIVILEGES ON DATABASE entangle_db TO entangle;
\du entangle
\q
```

**验证创建成功**：

```bash
docker exec -it entangle-db gsql -U entangle -d entangle_db -c "SELECT version();"
```

如果显示数据库版本信息，说明创建成功！

---

## 第三步：初始化项目配置

### 1. 创建 .env 文件

```bash
# 复制示例配置
cp .env.example .env

# 生成随机密钥（可选）
just secrets
```

如果运行 `just secrets`，会生成如下密钥：

```
APP_SECRET_KEY=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
JWT_SECRET=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

将这些密钥复制到 `.env` 文件中。

### 2. 检查 .env 配置

```bash
cat .env
```

确保以下配置正确：

```env
# 数据库连接
DATABASE_URL=postgres://entangle:Entangle@2024@localhost:5432/entangle_db

# JWT 密钥（必须设置）
JWT_SECRET=your-secret-key-here

# 应用配置
APP_PORT=3000
APP_HOST=127.0.0.1
```

---

## 第四步：运行数据库迁移

```bash
# 运行迁移，创建表结构
just migrate
# 或
sqlx migrate run
```

**成功输出示例**：

```
Applying migration: 001_init_permissions.sql
Applying migration: 002_create_users.sql
...
Finished! Migrations ran successfully.
```

---

## 第五步：启动后端服务

### 1. 编译并运行后端

```bash
# 在项目根目录
cargo run --bin entangle-api
```

首次编译需要 2-5 分钟，看到以下输出表示启动成功：

```
2025-01-05T10:00:00.000Z  INFO entangle_api: Starting Entangle API server...
2025-01-05T10:00:00.100Z  INFO entangle_api: Server listening on http://127.0.0.1:3000
2025-01-05T10:00:00.200Z  INFO entangle_api: Database connected successfully
2025-01-05T10:00:00.300Z  INFO entangle_api: Redis connected successfully
```

### 2. 测试 API

打开新终端，测试 API 是否正常：

```bash
# 健康检查
curl http://localhost:3000/health

# 应该返回
{"status":"ok"}
```

---

## 第六步：启动前端服务

### 1. 打开新终端

**重要**: 前端需要在新终端中运行！

```bash
# 进入项目目录
cd /home/vitus/Documents/Entangle

# 进入 nix-shell
nix-shell

# 进入前端目录
cd frontend

# 启动前端开发服务器
trunk serve
```

看到以下输出表示启动成功：

```
2025-01-05T10:00:00.000Z  INFO Building entangle-frontend
2025-01-05T10:00:30.000Z  INFO Finished
2025-01-05T10:00:30.100Z  INFO Serving at http://127.0.0.1:8080
```

### 2. 访问应用

打开浏览器访问：http://localhost:8080

---

## 一键启动（推荐）

上面的步骤可以简化为：

```bash
# 终端 1：启动数据库并运行后端
just dev
```

这个命令会自动：
1. 启动数据库容器
2. 等待数据库初始化（35秒）
3. 运行数据库迁移
4. 启动后端服务

```bash
# 终端 2：启动前端
cd frontend
trunk serve
```

---

## 常见问题排查

### 问题 1: "connection to server at "localhost" (127.0.0.1), port 5432 failed"

**原因**: 数据库未启动或未创建用户

**解决**:
```bash
# 检查容器状态
docker ps | grep entangle

# 如果没有运行，启动容器
just db-up

# 等待 30-40 秒后创建用户（见第二步）
```

---

### 问题 2: "password authentication failed for user 'entangle'"

**原因**: 未创建 entangle 用户或密码错误

**解决**:
```bash
# 重新创建用户
docker exec -i entangle-db gsql -U gaussdb postgres << 'EOF'
DROP USER IF EXISTS entangle;
CREATE USER entangle WITH PASSWORD 'Entangle@2024' SYSID 600;
ALTER USER entangle WITH CREATEDB CREATEROLE;
CREATE DATABASE entangle_db OWNER entangle;
GRANT ALL PRIVILEGES ON DATABASE entangle_db TO entangle;
EOF
```

---

### 问题 3: "error: linker `cc` not found"

**原因**: Nix 环境缺少 C 编译器

**解决**:
```bash
# 退出当前 nix-shell
exit

# 重新进入（会加载最新的 flake.nix 配置）
nix-shell
```

flake.nix 已更新为包含 gcc 和 gnumake。

---

### 问题 4: "database "entangle_db" does not exist"

**原因**: 数据库未创建

**解决**:
```bash
# 连接到数据库并创建
docker exec -it entangle-db gsql -U gaussdb postgres

CREATE DATABASE entangle_db OWNER entangle;
\q
```

---

### 问题 5: OpenGauss 启动太慢

**现象**: 容器启动了，但连接失败

**原因**: OpenGauss 首次启动需要初始化，需要 30-40 秒

**解决**:
```bash
# 耐心等待，查看日志
docker logs -f entangle-db

# 看到 "start success database successfully" 后再继续
```

---

### 问题 6: 前端页面空白或报错

**原因**: 后端未启动或 CORS 配置错误

**解决**:
```bash
# 检查后端是否运行
curl http://localhost:3000/health

# 检查浏览器控制台错误
# 如果是 CORS 错误，检查 .env 中的 CORS_ALLOWED_ORIGINS
```

---

## 验证安装

运行以下命令验证所有组件正常：

```bash
# 1. 检查数据库
docker exec -it entangle-db gsql -U entangle -d entangle_db -c "SELECT 1;"

# 2. 检查 Redis
docker exec -it entangle-redis redis-cli ping
# 应该返回 PONG

# 3. 检查后端 API
curl http://localhost:3000/health

# 4. 检查前端
curl http://localhost:8080
# 应该返回 HTML 内容
```

---

## 下一步

安装成功后，你可以：

1. **注册账号**: http://localhost:8080/register
2. **登录系统**: 使用注册的账号登录
3. **创建文档**: 点击"新建文档"
4. **测试协作**: 打开两个浏览器窗口，编辑同一文档

---

## 项目结构速览

```
Entangle/
├── crates/
│   ├── entangle-api/       # 后端 API
│   ├── entangle-frontend/  # 前端代码
│   └── entangle-crdt/      # CRDT 协作逻辑
├── frontend/               # 前端构建输出
├── migrations/             # 数据库迁移脚本
├── docs/                   # 项目文档
├── justfile                # 命令快捷方式
├── flake.nix               # Nix 环境配置
└── docker-compose.dev.yml  # 开发环境容器配置
```

---

## 常用命令

```bash
# 启动开发环境（数据库 + 后端）
just dev

# 仅启动数据库
just db-up

# 停止数据库
just db-down

# 运行迁移
just migrate

# 创建新迁移
just migrate-create <name>

# 查看数据库日志
just db-logs

# 运行后端
cargo run --bin entangle-api

# 运行测试
cargo test

# 代码检查
just lint

# 格式化代码
just fmt
```

---

## 技术支持

如果遇到问题：

1. 查看日志: `just db-logs`
2. 检查容器: `docker ps -a`
3. 查看迁移: `ls migrations/`
4. 阅读文档: `docs/QUICK_START.md`

**祝开发顺利！** 🚀
