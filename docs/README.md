# Entangle 文档中心

> 版本: 1.0.0 | 最后更新: 2026-01-01

欢迎来到 Entangle 项目文档中心。本目录包含项目的所有技术文档。

---

## 快速导航

### 入门指南

| 文档 | 说明 |
|------|------|
| [开发者指南](DEVELOPMENT.md) | 环境配置、编译启动、调试部署 |
| [快速开始](QUICK_START.md) | 项目概述和快速上手 |
| [测试指南](TESTING.md) | 测试用例和脚本使用 |

### 功能模块

| 文档 | 说明 |
|------|------|
| [认证系统](AUTH_README.md) | JWT 认证、RBAC 权限模型 |
| [文件夹系统](FOLDER_SYSTEM.md) | 层级文件夹、文档组织 |
| [标签系统](TAG_SYSTEM.md) | 标签管理、文档筛选 |
| [前端开发](FRONTEND.md) | Leptos/WASM 前端开发 |

### 📚 代码讲解（新增 - 用于答辩）

| 文档 | 说明 | 用途 |
|------|------|------|
| [**完整技术架构**](ARCHITECTURE.md) | 系统架构、技术栈、核心流程、常见问题 | 全面理解项目，准备答辩 |
| [**快速参考指南**](QUICK_REFERENCE.md) | 速查手册、演示流程、关键代码位置 | 演示时快速查阅 |
| [**核心代码导读**](CODE_WALKTHROUGH.md) | 带详细注释的核心代码讲解 | 代码走查，理解实现 |

### 项目管理

| 文档 | 说明 |
|------|------|
| [开发进度](PROGRESS.md) | 阶段进度、任务状态 |
| [项目计划](PROJECT_PLAN.md) | 总体规划、技术架构 |
| [课程报告](COURSE_REPORT.md) | 大作业报告 |

---

## 文档结构

```
docs/
├── README.md              # 本文档 - 文档索引
│
├── 入门指南
│   ├── DEVELOPMENT.md     # 开发者指南（编译、调试、部署）
│   ├── QUICK_START.md     # 快速开始
│   └── TESTING.md         # 测试指南
│
├── 功能模块
│   ├── AUTH_README.md     # 认证授权系统
│   ├── FOLDER_SYSTEM.md   # 文件夹系统
│   ├── TAG_SYSTEM.md      # 标签系统
│   └── FRONTEND.md        # 前端开发文档
│
├── 📚 代码讲解（用于答辩）
│   ├── ARCHITECTURE.md     # 完整技术架构文档（800 行）
│   ├── QUICK_REFERENCE.md  # 快速参考指南（400 行）
│   └── CODE_WALKTHROUGH.md # 核心代码导读（600 行）
│
└── 项目管理
    ├── PROGRESS.md        # 开发进度追踪
    ├── PROJECT_PLAN.md    # 项目总体计划
    └── COURSE_REPORT.md   # 课程大作业报告
```

---

## 技术栈概览

### 后端

| 组件 | 技术 |
|------|------|
| Web 框架 | Axum 0.7 |
| 数据库 | PostgreSQL (openGauss) |
| ORM | SQLx |
| 认证 | JWT + Argon2 |
| 实时协作 | WebSocket + Yrs (CRDT) |

### 前端

| 组件 | 技术 |
|------|------|
| 框架 | Leptos 0.6 |
| 运行时 | WebAssembly |
| 路由 | Leptos Router |
| HTTP | gloo-net |

### 工具链

| 工具 | 用途 |
|------|------|
| Nix Flakes | 开发环境管理 |
| Trunk | WASM 构建 |
| SQLx | 数据库迁移 |
| Just | 任务运行器 |

---

## 快速启动

```bash
# 1. 进入开发环境
nix develop

# 2. 启动数据库并运行迁移
sqlx migrate run

# 3. 启动后端 (终端1)
cargo run --bin entangle-api

# 4. 启动前端 (终端2)
cd frontend && trunk serve

# 5. 访问应用
open http://localhost:8080
```

---

## API 端点概览

### 认证

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/auth/register` | 用户注册 |
| POST | `/api/auth/login` | 用户登录 |
| GET | `/api/auth/profile` | 获取个人资料 |

### 文档

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/documents` | 创建文档 |
| GET | `/api/documents/:id` | 获取文档 |
| PUT | `/api/documents/:id` | 更新文档 |
| DELETE | `/api/documents/:id` | 删除文档 |
| GET | `/api/documents/my` | 我的文档 |
| PUT | `/api/documents/:id/move` | 移动文档 |

### 文件夹

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/folders` | 创建文件夹 |
| GET | `/api/folders/tree` | 获取文件夹树 |
| GET | `/api/folders/:id` | 获取文件夹详情 |
| PUT | `/api/folders/:id` | 更新文件夹 |
| DELETE | `/api/folders/:id` | 删除文件夹 |

### 标签

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/tags` | 创建标签 |
| GET | `/api/tags` | 获取所有标签 |
| PUT | `/api/tags/:id` | 更新标签 |
| DELETE | `/api/tags/:id` | 删除标签 |
| GET | `/api/documents/by-tags` | 按标签筛选 |

### WebSocket

| 路径 | 说明 |
|------|------|
| `ws://host/ws/documents/:id` | 文档实时协作 |

---

## 项目进度

| 阶段 | 状态 | 进度 |
|------|------|------|
| 项目基础 | ✅ 完成 | 100% |
| 认证系统 | ✅ 完成 | 100% |
| 文档核心 | ✅ 完成 | 100% |
| 文档管理扩展 | ✅ 完成 | 100% |
| 实时协作 | ✅ 完成 | 100% |
| 前端开发 | ✅ 完成 | 100% |
| 评论通知 | ⏳ 待开始 | 0% |
| 测试收尾 | ⏳ 待开始 | 0% |

**总体进度: 75%**

---

## 🎯 答辩准备建议

### 快速通道（1-2 小时准备）

1. **快速理解**（30 分钟）
   - 阅读 [QUICK_REFERENCE.md](QUICK_REFERENCE.md)
   - 记住核心技术栈、数据流程

2. **深入学习**（40 分钟）
   - 阅读 [ARCHITECTURE.md](ARCHITECTURE.md) 的：
     - 项目概述
     - 整体架构
     - 实时协作系统详解
     - 回答老师可能的问题

3. **代码准备**（20 分钟）
   - 浏览 [CODE_WALKTHROUGH.md](CODE_WALKTHROUGH.md)
   - 理解 WebSocket 主循环

### 关键问题准备

| 问题 | 查看 |
|------|------|
| 为什么用 Rust？ | [ARCHITECTURE.md](ARCHITECTURE.md) - 技术栈详解 |
| CRDT 是什么？ | [ARCHITECTURE.md](ARCHITECTURE.md) - Q2 |
| WebSocket 如何工作？ | [CODE_WALKTHROUGH.md](CODE_WALKTHROUGH.md) |
| 如何保证并发安全？ | [ARCHITECTURE.md](ARCHITECTURE.md) - Q5 |

---

## 贡献指南

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/xxx`)
3. 提交更改 (`git commit -m 'feat: xxx'`)
4. 推送分支 (`git push origin feature/xxx`)
5. 创建 Pull Request

---

## 相关链接

- [Leptos 文档](https://leptos.dev/)
- [Axum 文档](https://docs.rs/axum)
- [SQLx 文档](https://docs.rs/sqlx)
- [Yrs 文档](https://docs.rs/yrs)

---

*文档由 Entangle 团队维护*
