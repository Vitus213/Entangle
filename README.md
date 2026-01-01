# Entangle - 多人协作文档编辑系统

> 基于 Rust + openGauss 的实时协作文档编辑平台

## 快速开始

### 使用 Nix 开发环境（推荐）

本项目使用 Nix Flakes 管理开发环境和依赖。

#### 1. 安装 Nix

```bash
# 官方安装脚本
sh <(curl -L https://nixos.org/nix/install) --daemon

# 或使用 Determinate Systems 的安装器（推荐）
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

#### 2. 启用 Nix Flakes

如果使用官方安装脚本，需要手动启用 flakes：

```bash
mkdir -p ~/.config/nix
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
```

#### 3. 进入开发环境

```bash
# 克隆项目
git clone <repository-url>
cd Entangle

# 运行自动设置脚本
./scripts/setup.sh

# 进入 Nix 开发环境
nix develop
```

#### 4. 使用 direnv（可选，强烈推荐）

自动加载环境变量：

```bash
# 安装 direnv
nix profile install nixpkgs#direnv

# 配置 shell（bash）
echo 'eval "$(direnv hook bash)"' >> ~/.bashrc

# 或者 zsh
echo 'eval "$(direnv hook zsh)"' >> ~/.zshrc

# 允许 direnv
direnv allow .
```

之后每次进入项目目录，环境会自动加载！

### 环境变量管理

项目的环境变量分为两部分：

1. **公共配置**（`flake.nix`）：非敏感的配置，如端口号、日志级别等
2. **私密配置**（`.env`）：敏感信息，如密钥、密码等

生成随机密钥：

```bash
just secrets
# 或
openssl rand -hex 32
```

### 开发工作流

#### 后端开发 (使用 Just 命令)

```bash
# 查看所有可用命令
just

# 启动数据库
just db-up

# 运行迁移
just migrate

# 运行开发服务器
just run

# 自动重载开发
just watch

# 完整开发流程（启动数据库 + 迁移 + 运行）
just dev

# 代码检查
just check
```

#### 前端开发 (Leptos + WASM)

```bash
# 进入前端目录
cd frontend

# 启动开发服务器（热重载，推荐）
trunk serve

# 自定义端口
trunk serve --port 3001

# 生产构建
trunk build --release
```

#### 全栈开发（推荐）

```bash
# Terminal 1: 启动后端（端口 3000）
just dev

# Terminal 2: 启动前端（端口 8080）
cd frontend && trunk serve

# 然后访问: http://localhost:8080
```

#### 手动命令

```bash
# 启动数据库（openGauss + Redis）
docker-compose -f docker-compose.dev.yml up -d

# 运行数据库迁移
sqlx migrate run

# 运行 API 服务器
cargo run --bin entangle-api

# 运行测试
cargo test

# 代码格式化
cargo fmt

# 代码检查
cargo clippy -- -D warnings
```

## 项目结构

```
Entangle/
├── flake.nix              # Nix Flakes 配置（环境 + 依赖）
├── shell.nix              # 传统 nix-shell 支持
├── .envrc                 # direnv 配置
├── .env                   # 私密环境变量（不提交）
├── .env.example           # 环境变量示例
├── justfile               # 开发命令
├── Cargo.toml             # Workspace 配置
├── docker-compose.dev.yml # 开发数据库
├── crates/                # Rust 代码
│   ├── api/              # HTTP/WebSocket 接口
│   ├── core/             # 业务逻辑
│   ├── db/               # 数据访问
│   ├── auth/             # 认证授权
│   └── collab/           # 实时协作 (CRDT)
├── migrations/            # 数据库迁移
├── scripts/               # 辅助脚本
├── uploads/               # 文件上传目录
└── docs/                  # 项目文档
```

## Nix 环境特性

### 自动提供的工具

- Rust 稳定版工具链（rustc, cargo, rust-analyzer）
- PostgreSQL 客户端工具
- Redis 客户端工具
- sqlx-cli（数据库迁移）
- Docker 和 Docker Compose
- Just（命令运行器）
- watchexec（文件监视）
- Node.js 20 + pnpm（前端开发）

### 环境变量自动注入

进入 Nix 环境后，以下变量自动设置：

- `APP_*`: 应用配置
- `DATABASE_URL`: 数据库连接
- `REDIS_URL`: Redis 连接
- `LOG_LEVEL`: 日志级别
- 等等...

### 开发体验优化

```bash
# 一次性设置
./scripts/setup.sh

# 之后只需
cd Entangle  # direnv 自动加载环境
just dev     # 启动开发
```

## 常见问题

### Q: 如何添加新的环境变量？

**公共变量**：编辑 `flake.nix` 中的 `env` 部分
**私密变量**：添加到 `.env` 文件

### Q: 数据库连接失败？

```bash
# 检查容器状态
docker-compose -f docker-compose.dev.yml ps

# 查看日志
just db-logs

# 重启
just db-down
just db-up
```

### Q: 如何更新 Nix 依赖？

```bash
nix flake update
```

### Q: 不想用 Nix 怎么办？

传统方式仍然支持：

```bash
# 手动安装依赖
# - Rust (rustup)
# - PostgreSQL
# - Redis
# - Docker

# 复制环境变量
cp .env.example .env

# 启动
docker-compose -f docker-compose.dev.yml up -d
cargo run
```

## 技术栈

### 后端
- **框架**: Rust + Axum
- **数据库**: openGauss (PostgreSQL 兼容)
- **ORM**: SeaORM
- **缓存**: Redis
- **实时协作**: Yrs (CRDT)
- **认证**: JWT + Argon2

### 前端
- **框架**: Leptos 0.6 (Rust 全栈)
- **运行时**: WebAssembly (WASM)
- **路由**: Leptos Router
- **HTTP 客户端**: gloo-net
- **构建工具**: Trunk

### 开发工具
- **包管理**: Nix Flakes
- **任务运行**: Just
- **容器化**: Docker + Docker Compose

## 文档

### 项目规划
- [项目计划](docs/PROJECT_PLAN.md) - 完整的系统设计和开发计划
- [开发进度](docs/PROGRESS.md) - 当前开发状态

### 后端文档
- [认证系统](docs/AUTH_README.md) - RBAC 权限管理
- [文件夹设计](docs/FOLDER_DESIGN.md) - 文件夹系统架构
- [标签设计](docs/TAG_DESIGN.md) - 标签系统架构
- [测试文档](docs/TESTING.md) - 测试指南

### 前端文档
- [前端开发指南](docs/FRONTEND_GUIDE.md) - Leptos 开发详细指南
- [前端环境配置](docs/FRONTEND_SETUP.md) - Nix 构建环境说明

### 使用指南
- [快速开始](docs/QUICK_START.md) - 快速上手指南
- [文件夹使用](docs/FOLDER_USAGE.md) - 文件夹功能说明
- [标签使用](docs/TAG_USAGE.md) - 标签功能说明

## 许可证

MIT
