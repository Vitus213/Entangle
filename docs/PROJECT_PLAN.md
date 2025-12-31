# Entangle - 多人协作文档编辑系统

> 基于 Rust 的实时协作文档、表格和演示文稿编辑平台

---

## 📋 目录

1. [项目概述](#1-项目概述)
2. [技术架构](#2-技术架构)
3. [功能模块规划](#3-功能模块规划)
4. [数据库设计](#4-数据库设计)
5. [API 接口设计](#5-api-接口设计)
6. [开发任务清单](#6-开发任务清单)
7. [项目结构](#7-项目结构)
8. [部署方案](#8-部署方案)

---

## 1. 项目概述

### 1.1 项目背景

本项目为数据库系统课程大作业，目标是开发一款支持多人同时在线编辑文档、表格和演示文稿的协作软件。

### 1.2 核心目标

- 支持多用户实时协同编辑
- 提供完善的用户权限管理
- 实现文档版本控制与历史记录
- 支持多种文档格式的导入导出

### 1.3 技术选型总览

| 组件 | 技术选择 | 版本 | 说明 |
|------|----------|------|------|
| 后端框架 | Axum | 0.7.x | 高性能异步 Web 框架 |
| 数据库 | openGauss | 5.0+ | 课程要求，兼容 PostgreSQL 协议 |
| ORM | SeaORM | 0.12.x | 异步 ORM，支持编译时检查 |
| 实时协作 | yrs (Yjs Rust) | 0.18.x | CRDT 算法实现 |
| WebSocket | tokio-tungstenite | 0.21.x | 异步 WebSocket |
| 认证 | JWT + argon2 | - | 无状态认证 + 安全密码哈希 |
| 前端框架 | Vue 3 + TypeScript | 3.4.x | 或 Leptos (全栈 Rust) |
| 富文本编辑器 | TipTap / ProseMirror | - | 支持协作编辑 |
| 缓存 | Redis | 7.x | 会话管理、实时状态 |

---

## 2. 技术架构

### 2.1 系统架构图

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              客户端层                                    │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
│  │   Web App   │  │  Mobile H5  │  │   Desktop   │  │   Tablet    │    │
│  │  (Vue 3)    │  │ (响应式)     │  │  (Tauri)    │  │  (响应式)    │    │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘    │
└─────────┼────────────────┼────────────────┼────────────────┼───────────┘
          │                │                │                │
          ▼                ▼                ▼                ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                            API 网关层                                    │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    Nginx / Traefik                               │   │
│  │              (负载均衡 / SSL 终止 / 静态资源)                      │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                            应用服务层                                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │
│  │   HTTP Server   │  │  WebSocket Hub  │  │  Background     │         │
│  │   (Axum)        │  │  (实时协作)      │  │  Workers        │         │
│  │                 │  │                 │  │  (通知/任务)     │         │
│  │  • REST API     │  │  • 文档同步      │  │                 │         │
│  │  • 认证授权      │  │  • 光标位置      │  │  • 邮件发送      │         │
│  │  • 文件上传      │  │  • 在线状态      │  │  • 定时任务      │         │
│  │                 │  │  • 通知推送      │  │  • 数据清理      │         │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘         │
│           │                    │                    │                   │
│           └────────────────────┼────────────────────┘                   │
│                                │                                        │
│                    ┌───────────▼───────────┐                           │
│                    │     Core Services     │                           │
│                    │                       │                           │
│                    │  • UserService        │                           │
│                    │  • DocumentService    │                           │
│                    │  • CollabService      │                           │
│                    │  • NotificationSvc    │                           │
│                    │  • PermissionService  │                           │
│                    └───────────┬───────────┘                           │
│                                │                                        │
└────────────────────────────────┼────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                            数据存储层                                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │
│  │   openGauss     │  │     Redis       │  │   MinIO/本地     │         │
│  │   (主数据库)     │  │   (缓存/会话)    │  │   (文件存储)     │         │
│  │                 │  │                 │  │                 │         │
│  │  • 用户数据      │  │  • JWT 黑名单   │  │  • 头像         │         │
│  │  • 文档元信息    │  │  • 在线用户     │  │  • 附件         │         │
│  │  • 权限配置      │  │  • 协作状态     │  │  • 导出文件      │         │
│  │  • 版本历史      │  │  • 消息队列     │  │                 │         │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘         │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.2 实时协作架构 (CRDT)

```
┌────────────────────────────────────────────────────────────────────┐
│                        CRDT 协作流程                                │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│   Client A                  Server                   Client B      │
│   ┌──────┐                 ┌──────┐                 ┌──────┐      │
│   │ Yjs  │                 │ yrs  │                 │ Yjs  │      │
│   │ Doc  │                 │ Doc  │                 │ Doc  │      │
│   └──┬───┘                 └──┬───┘                 └──┬───┘      │
│      │                        │                        │          │
│      │   1. 本地编辑           │                        │          │
│      │   ──────────►          │                        │          │
│      │                        │                        │          │
│      │   2. 发送更新 (WS)      │                        │          │
│      │ ─────────────────────► │                        │          │
│      │   (Yjs Update Binary)  │                        │          │
│      │                        │                        │          │
│      │                        │   3. 广播更新           │          │
│      │                        │ ─────────────────────► │          │
│      │                        │                        │          │
│      │                        │   4. 应用更新           │          │
│      │                        │                   ◄────┤          │
│      │                        │                        │          │
│      │                        │   5. 持久化到数据库     │          │
│      │                        │ ────────┐              │          │
│      │                        │         │              │          │
│      │                        │ ◄───────┘              │          │
│      │                        │                        │          │
│                                                                    │
│   ┌────────────────────────────────────────────────────────────┐  │
│   │  CRDT 优势：                                                 │  │
│   │  • 无需中央锁，支持并发编辑                                   │  │
│   │  • 自动冲突解决，保证最终一致性                                │  │
│   │  • 支持离线编辑，上线后自动同步                                │  │
│   └────────────────────────────────────────────────────────────┘  │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

### 2.3 认证授权流程

```
┌────────────────────────────────────────────────────────────────────┐
│                        JWT 认证流程                                 │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│  ┌─────────┐          ┌─────────┐          ┌─────────┐            │
│  │  用户    │          │  服务器  │          │ 数据库   │            │
│  └────┬────┘          └────┬────┘          └────┬────┘            │
│       │                    │                    │                  │
│       │  1. POST /login    │                    │                  │
│       │  {email, password} │                    │                  │
│       │ ─────────────────► │                    │                  │
│       │                    │                    │                  │
│       │                    │  2. 查询用户        │                  │
│       │                    │ ─────────────────► │                  │
│       │                    │                    │                  │
│       │                    │  3. 返回用户信息    │                  │
│       │                    │ ◄───────────────── │                  │
│       │                    │                    │                  │
│       │                    │  4. 验证密码 (argon2)                  │
│       │                    │  ┌──────────────┐  │                  │
│       │                    │  │ hash_verify  │  │                  │
│       │                    │  └──────────────┘  │                  │
│       │                    │                    │                  │
│       │  5. 返回 JWT Token  │                    │                  │
│       │ ◄───────────────── │                    │                  │
│       │  {access_token,    │                    │                  │
│       │   refresh_token}   │                    │                  │
│       │                    │                    │                  │
│       │  6. 请求 API        │                    │                  │
│       │  Authorization:    │                    │                  │
│       │  Bearer <token>    │                    │                  │
│       │ ─────────────────► │                    │                  │
│       │                    │                    │                  │
│       │                    │  7. 验证 JWT        │                  │
│       │                    │  ┌──────────────┐  │                  │
│       │                    │  │ jwt_verify   │  │                  │
│       │                    │  └──────────────┘  │                  │
│       │                    │                    │                  │
│       │  8. 返回数据        │                    │                  │
│       │ ◄───────────────── │                    │                  │
│       │                    │                    │                  │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

---

## 3. 功能模块规划

### 3.1 模块优先级矩阵

```
                    重要性
                      ▲
                      │
          ┌──────────┼──────────┐
          │  P1      │  P0      │
  高      │ 选做加分  │ 核心必做  │
          │          │          │
  ────────┼──────────┼──────────┼────► 紧急性
          │  P3      │  P2      │
  低      │ 可放弃   │ 时间允许  │
          │          │          │
          └──────────┴──────────┘
              低         高
```

### 3.2 模块详细规划

#### 🔴 P0 - 核心必做模块

| 模块 | 功能 | 复杂度 | 预估工时 |
|------|------|--------|----------|
| **用户认证** | 注册/登录/密码重置 | ⭐⭐ | 8h |
| **用户信息** | 资料编辑/头像上传 | ⭐⭐ | 6h |
| **权限管理** | 角色定义/权限分配/ACL | ⭐⭐⭐ | 12h |
| **文档 CRUD** | 创建/编辑/删除/列表 | ⭐⭐ | 10h |
| **富文本编辑** | TipTap 集成/自动保存 | ⭐⭐⭐ | 16h |
| **文档分类** | 文件夹/标签/搜索 | ⭐⭐ | 8h |
| **实时协作** | CRDT 同步/冲突处理 | ⭐⭐⭐⭐⭐ | 40h |
| **光标同步** | 多用户光标位置显示 | ⭐⭐⭐ | 8h |
| **评论批注** | 行内评论/回复/@提及 | ⭐⭐⭐ | 12h |
| **基础通知** | 实时推送/通知列表 | ⭐⭐ | 8h |

#### 🟡 P1 - 推荐选做模块

| 模块 | 功能 | 复杂度 | 加分潜力 |
|------|------|--------|----------|
| **Markdown 支持** | 编辑/预览/导出 | ⭐⭐ | ⭐⭐⭐ |
| **版本控制** | 历史记录/对比/回滚 | ⭐⭐⭐ | ⭐⭐⭐⭐ |
| **文档导入导出** | Word/PDF/Markdown | ⭐⭐⭐ | ⭐⭐⭐ |
| **任务分配** | 创建/分配/状态跟踪 | ⭐⭐ | ⭐⭐ |

#### 🟢 P2 - 时间允许模块

| 模块 | 功能 | 复杂度 | 说明 |
|------|------|--------|------|
| **响应式设计** | 移动端适配 | ⭐⭐ | CSS 媒体查询 |
| **系统监控** | 性能监控/日志 | ⭐⭐⭐ | Prometheus |
| **系统配置** | 参数设置/功能开关 | ⭐ | 简单 CRUD |

#### ⚫ P3 - 建议放弃模块

| 模块 | 功能 | 原因 |
|------|------|------|
| **视频会议** | WebRTC 集成 | 复杂度极高，需要 TURN 服务器 |
| **屏幕共享** | 实时共享 | 需要浏览器 API + 复杂编码 |
| **离线编辑** | Service Worker | IndexedDB + 复杂同步逻辑 |

---

### 3.3 功能依赖关系图

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           功能依赖关系                                   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────┐                                                        │
│  │   用户认证   │◄─────────────────────────────────────────────┐        │
│  │ (注册/登录)  │                                               │        │
│  └──────┬──────┘                                               │        │
│         │                                                       │        │
│         ▼                                                       │        │
│  ┌─────────────┐     ┌─────────────┐                           │        │
│  │   用户信息   │     │   权限管理   │◄──────────────────┐      │        │
│  │ (资料/头像)  │     │ (角色/ACL)  │                    │      │        │
│  └──────┬──────┘     └──────┬──────┘                    │      │        │
│         │                   │                            │      │        │
│         └─────────┬─────────┘                            │      │        │
│                   │                                      │      │        │
│                   ▼                                      │      │        │
│  ┌────────────────────────────────┐                     │      │        │
│  │          文档管理               │                     │      │        │
│  │  ┌─────────┐  ┌─────────────┐  │                     │      │        │
│  │  │ 文档CRUD │  │ 分类与搜索   │  │                     │      │        │
│  │  └────┬────┘  └─────────────┘  │                     │      │        │
│  └───────┼────────────────────────┘                     │      │        │
│          │                                               │      │        │
│          ▼                                               │      │        │
│  ┌────────────────────────────────────────────────┐     │      │        │
│  │              实时协作 (核心)                     │     │      │        │
│  │  ┌──────────────┐  ┌──────────────┐            │     │      │        │
│  │  │ CRDT 同步     │  │ 光标位置同步  │            │     │      │        │
│  │  └──────┬───────┘  └──────────────┘            │     │      │        │
│  └─────────┼──────────────────────────────────────┘     │      │        │
│            │                                             │      │        │
│            ▼                                             │      │        │
│  ┌─────────────────┐     ┌─────────────────┐            │      │        │
│  │    评论批注      │     │    版本控制     │ (选做)     │      │        │
│  │ (评论/回复/@)   │     │ (历史/对比)     │────────────┘      │        │
│  └────────┬────────┘     └─────────────────┘                   │        │
│           │                                                     │        │
│           ▼                                                     │        │
│  ┌─────────────────┐     ┌─────────────────┐                   │        │
│  │    通知系统      │────►│   任务分配      │                   │        │
│  │ (实时推送)       │     │ (创建/跟踪)     │───────────────────┘        │
│  └─────────────────┘     └─────────────────┘                            │
│                                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │  图例：                                                           │  │
│  │  ───► 依赖关系    ════► 强依赖    ----► 可选依赖                  │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 4. 数据库设计

### 4.1 ER 图

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                                    ER 图                                             │
├─────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                     │
│    ┌───────────────┐           ┌───────────────┐           ┌───────────────┐       │
│    │     users     │           │     roles     │           │  permissions  │       │
│    ├───────────────┤           ├───────────────┤           ├───────────────┤       │
│    │ PK id         │     ┌────►│ PK id         │◄────┐     │ PK id         │       │
│    │    email      │     │     │    name       │     │     │    name       │       │
│    │    password   │     │     │    desc       │     │     │    resource   │       │
│    │    nickname   │     │     └───────────────┘     │     │    action     │       │
│    │    avatar_url │     │                           │     └───────┬───────┘       │
│    │ FK role_id    │─────┘     ┌───────────────┐     │             │               │
│    │    created_at │           │role_permissions│     │             │               │
│    └───────┬───────┘           ├───────────────┤     │             │               │
│            │                   │ FK role_id    │─────┘             │               │
│            │                   │ FK perm_id    │───────────────────┘               │
│            │                   └───────────────┘                                   │
│            │                                                                        │
│            │ 1:N                                                                    │
│            ▼                                                                        │
│    ┌───────────────┐           ┌───────────────┐           ┌───────────────┐       │
│    │   documents   │ 1:N       │doc_collaborators│          │  doc_versions │       │
│    ├───────────────┤──────────►├───────────────┤           ├───────────────┤       │
│    │ PK id         │◄──────────│ FK doc_id     │           │ PK id         │       │
│    │    title      │           │ FK user_id    │───┐       │ FK doc_id     │───┐   │
│    │    content    │           │    role       │   │       │    content    │   │   │
│    │    type       │           │    created_at │   │       │    version    │   │   │
│    │ FK owner_id   │───────────┴───────────────┘   │       │    created_by │   │   │
│    │ FK folder_id  │───┐                           │       │    created_at │   │   │
│    │    created_at │   │                           │       └───────────────┘   │   │
│    │    updated_at │   │                           │                           │   │
│    └───────┬───────┘   │                           └───────────────────────────┘   │
│            │           │                                                            │
│            │           │       ┌───────────────┐                                   │
│            │           └──────►│    folders    │                                   │
│            │                   ├───────────────┤                                   │
│            │                   │ PK id         │◄───┐                              │
│            │                   │    name       │    │                              │
│            │                   │ FK parent_id  │────┘ (自引用)                      │
│            │                   │ FK owner_id   │                                   │
│            │                   └───────────────┘                                   │
│            │                                                                        │
│            │ 1:N                                                                    │
│            ▼                                                                        │
│    ┌───────────────┐           ┌───────────────┐           ┌───────────────┐       │
│    │   comments    │           │ notifications │           │     tasks     │       │
│    ├───────────────┤           ├───────────────┤           ├───────────────┤       │
│    │ PK id         │           │ PK id         │           │ PK id         │       │
│    │ FK doc_id     │           │ FK user_id    │           │ FK doc_id     │       │
│    │ FK user_id    │           │    type       │           │ FK assignee   │       │
│    │ FK parent_id  │───┐       │    content    │           │ FK created_by │       │
│    │    content    │   │       │    is_read    │           │    title      │       │
│    │    position   │   │       │    created_at │           │    status     │       │
│    │    created_at │   │       └───────────────┘           │    due_date   │       │
│    └───────────────┘   │                                   └───────────────┘       │
│            ▲           │                                                            │
│            └───────────┘ (回复)                                                      │
│                                                                                     │
│    ┌───────────────┐           ┌───────────────┐                                   │
│    │  doc_tags     │ N:M       │     tags      │                                   │
│    ├───────────────┤──────────►├───────────────┤                                   │
│    │ FK doc_id     │           │ PK id         │                                   │
│    │ FK tag_id     │           │    name       │                                   │
│    └───────────────┘           │    color      │                                   │
│                                └───────────────┘                                   │
│                                                                                     │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

### 4.2 数据库表设计

#### 4.2.1 用户表 (users)

| 中文字段名 | 英文字段名 | 类型 | 约束 | 备注 |
|-----------|-----------|------|------|------|
| 用户ID | id | UUID | PRIMARY KEY | 主键，自动生成 |
| 邮箱 | email | VARCHAR(255) | UNIQUE, NOT NULL | 登录凭证 |
| 手机号 | phone | VARCHAR(20) | UNIQUE | 可选登录凭证 |
| 密码哈希 | password_hash | VARCHAR(255) | NOT NULL | argon2 加密 |
| 昵称 | nickname | VARCHAR(100) | NOT NULL | 显示名称 |
| 头像URL | avatar_url | VARCHAR(500) | | 头像存储路径 |
| 角色ID | role_id | UUID | FOREIGN KEY | 关联 roles 表 |
| 邮箱已验证 | email_verified | BOOLEAN | DEFAULT FALSE | 邮箱验证状态 |
| 状态 | status | VARCHAR(20) | DEFAULT 'active' | active/disabled/deleted |
| 最后登录时间 | last_login_at | TIMESTAMP | | 最近登录时间 |
| 创建时间 | created_at | TIMESTAMP | DEFAULT NOW() | 注册时间 |
| 更新时间 | updated_at | TIMESTAMP | DEFAULT NOW() | 最后更新时间 |

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    phone VARCHAR(20) UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    nickname VARCHAR(100) NOT NULL,
    avatar_url VARCHAR(500),
    role_id UUID REFERENCES roles(id),
    email_verified BOOLEAN DEFAULT FALSE,
    status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'disabled', 'deleted')),
    last_login_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_phone ON users(phone);
CREATE INDEX idx_users_role ON users(role_id);
```

#### 4.2.2 角色表 (roles)

| 中文字段名 | 英文字段名 | 类型 | 约束 | 备注 |
|-----------|-----------|------|------|------|
| 角色ID | id | UUID | PRIMARY KEY | 主键 |
| 角色名称 | name | VARCHAR(50) | UNIQUE, NOT NULL | admin/editor/viewer |
| 角色描述 | description | TEXT | | 角色说明 |
| 是否系统角色 | is_system | BOOLEAN | DEFAULT FALSE | 系统内置角色不可删除 |
| 创建时间 | created_at | TIMESTAMP | DEFAULT NOW() | |

```sql
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) UNIQUE NOT NULL,
    description TEXT,
    is_system BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT NOW()
);

-- 初始化系统角色
INSERT INTO roles (name, description, is_system) VALUES
    ('admin', '系统管理员，拥有所有权限', TRUE),
    ('editor', '编辑者，可以创建和编辑文档', TRUE),
    ('viewer', '查看者，只能查看文档', TRUE);
```

#### 4.2.3 权限表 (permissions)

| 中文字段名 | 英文字段名 | 类型 | 约束 | 备注 |
|-----------|-----------|------|------|------|
| 权限ID | id | UUID | PRIMARY KEY | 主键 |
| 权限名称 | name | VARCHAR(100) | UNIQUE, NOT NULL | 如 document:create |
| 资源类型 | resource | VARCHAR(50) | NOT NULL | document/user/system |
| 操作类型 | action | VARCHAR(50) | NOT NULL | create/read/update/delete |
| 权限描述 | description | TEXT | | |

```sql
CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) UNIQUE NOT NULL,
    resource VARCHAR(50) NOT NULL,
    action VARCHAR(50) NOT NULL,
    description TEXT
);

-- 初始化权限
INSERT INTO permissions (name, resource, action, description) VALUES
    ('document:create', 'document', 'create', '创建文档'),
    ('document:read', 'document', 'read', '查看文档'),
    ('document:update', 'document', 'update', '编辑文档'),
    ('document:delete', 'document', 'delete', '删除文档'),
    ('user:manage', 'user', 'manage', '管理用户'),
    ('system:config', 'system', 'config', '系统配置');
```

#### 4.2.4 角色权限关联表 (role_permissions)

| 中文字段名 | 英文字段名 | 类型 | 约束 | 备注 |
|-----------|-----------|------|------|------|
| 角色ID | role_id | UUID | PRIMARY KEY, FK | |
| 权限ID | permission_id | UUID | PRIMARY KEY, FK | |

```sql
CREATE TABLE role_permissions (
    role_id UUID REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);
```

#### 4.2.5 文档表 (documents)

| 中文字段名 | 英文字段名 | 类型 | 约束 | 备注 |
|-----------|-----------|------|------|------|
| 文档ID | id | UUID | PRIMARY KEY | 主键 |
| 文档标题 | title | VARCHAR(500) | NOT NULL | |
| 文档内容 | content | JSONB | | Yjs 文档状态/富文本 JSON |
| 文档类型 | doc_type | VARCHAR(20) | NOT NULL | document/spreadsheet/presentation |
| 所有者ID | owner_id | UUID | FOREIGN KEY, NOT NULL | 创建者 |
| 文件夹ID | folder_id | UUID | FOREIGN KEY | 所属文件夹 |
| 是否公开 | is_public | BOOLEAN | DEFAULT FALSE | 公开可访问 |
| 状态 | status | VARCHAR(20) | DEFAULT 'active' | active/archived/deleted |
| 创建时间 | created_at | TIMESTAMP | DEFAULT NOW() | |
| 更新时间 | updated_at | TIMESTAMP | DEFAULT NOW() | |

```sql
CREATE TABLE documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(500) NOT NULL,
    content JSONB,
    doc_type VARCHAR(20) NOT NULL CHECK (doc_type IN ('document', 'spreadsheet', 'presentation')),
    owner_id UUID NOT NULL REFERENCES users(id),
    folder_id UUID REFERENCES folders(id),
    is_public BOOLEAN DEFAULT FALSE,
    status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'archived', 'deleted')),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_documents_owner ON documents(owner_id);
CREATE INDEX idx_documents_folder ON documents(folder_id);
CREATE INDEX idx_documents_status ON documents(status);
CREATE INDEX idx_documents_title_gin ON documents USING gin(to_tsvector('simple', title));
```

#### 4.2.6 文档协作者表 (doc_collaborators)

| 中文字段名 | 英文字段名 | 类型 | 约束 | 备注 |
|-----------|-----------|------|------|------|
| ID | id | UUID | PRIMARY KEY | 主键 |
| 文档ID | doc_id | UUID | FOREIGN KEY, NOT NULL | |
| 用户ID | user_id | UUID | FOREIGN KEY, NOT NULL | |
| 协作角色 | role | VARCHAR(20) | NOT NULL | owner/editor/commenter/viewer |
| 邀请者ID | invited_by | UUID | FOREIGN KEY | 谁邀请的 |
| 创建时间 | created_at | TIMESTAMP | DEFAULT NOW() | |

```sql
CREATE TABLE doc_collaborators (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    doc_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL CHECK (role IN ('owner', 'editor', 'commenter', 'viewer')),
    invited_by UUID REFERENCES users(id),
    created_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(doc_id, user_id)
);

CREATE INDEX idx_doc_collaborators_doc ON doc_collaborators(doc_id);
CREATE INDEX idx_doc_collaborators_user ON doc_collaborators(user_id);
```

#### 4.2.7 文档版本表 (doc_versions) - 选做

| 中文字段名 | 英文字段名 | 类型 | 约束 | 备注 |
|-----------|-----------|------|------|------|
| 版本ID | id | UUID | PRIMARY KEY | 主键 |
| 文档ID | doc_id | UUID | FOREIGN KEY, NOT NULL | |
| 版本号 | version | INTEGER | NOT NULL | 自增版本号 |
| 内容快照 | content | JSONB | NOT NULL | 该版本的完整内容 |
| 创建者ID | created_by | UUID | FOREIGN KEY | |
| 版本说明 | message | VARCHAR(500) | | 版本描述 |
| 是否锁定 | is_locked | BOOLEAN | DEFAULT FALSE | 锁定后不可删除 |
| 创建时间 | created_at | TIMESTAMP | DEFAULT NOW() | |

```sql
CREATE TABLE doc_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    doc_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    version INTEGER NOT NULL,
    content JSONB NOT NULL,
    created_by UUID REFERENCES users(id),
    message VARCHAR(500),
    is_locked BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(doc_id, version)
);

CREATE INDEX idx_doc_versions_doc ON doc_versions(doc_id);
```

#### 4.2.8 文件夹表 (folders)

| 中文字段名 | 英文字段名 | 类型 | 约束 | 备注 |
|-----------|-----------|------|------|------|
| 文件夹ID | id | UUID | PRIMARY KEY | 主键 |
| 文件夹名称 | name | VARCHAR(255) | NOT NULL | |
| 父文件夹ID | parent_id | UUID | FOREIGN KEY | 自引用，支持嵌套 |
| 所有者ID | owner_id | UUID | FOREIGN KEY, NOT NULL | |
| 创建时间 | created_at | TIMESTAMP | DEFAULT NOW() | |

```sql
CREATE TABLE folders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    parent_id UUID REFERENCES folders(id) ON DELETE CASCADE,
    owner_id UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_folders_parent ON folders(parent_id);
CREATE INDEX idx_folders_owner ON folders(owner_id);
```

#### 4.2.9 标签表 (tags)

| 中文字段名 | 英文字段名 | 类型 | 约束 | 备注 |
|-----------|-----------|------|------|------|
| 标签ID | id | UUID | PRIMARY KEY | 主键 |
| 标签名称 | name | VARCHAR(50) | NOT NULL | |
| 标签颜色 | color | VARCHAR(7) | | HEX 颜色值 |
| 所有者ID | owner_id | UUID | FOREIGN KEY | 用户自定义标签 |
| 创建时间 | created_at | TIMESTAMP | DEFAULT NOW() | |

```sql
CREATE TABLE tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) NOT NULL,
    color VARCHAR(7),
    owner_id UUID REFERENCES users(id),
    created_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(name, owner_id)
);
```

#### 4.2.10 文档标签关联表 (doc_tags)

```sql
CREATE TABLE doc_tags (
    doc_id UUID REFERENCES documents(id) ON DELETE CASCADE,
    tag_id UUID REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (doc_id, tag_id)
);
```

#### 4.2.11 评论表 (comments)

| 中文字段名 | 英文字段名 | 类型 | 约束 | 备注 |
|-----------|-----------|------|------|------|
| 评论ID | id | UUID | PRIMARY KEY | 主键 |
| 文档ID | doc_id | UUID | FOREIGN KEY, NOT NULL | |
| 用户ID | user_id | UUID | FOREIGN KEY, NOT NULL | 评论者 |
| 父评论ID | parent_id | UUID | FOREIGN KEY | 回复的评论 |
| 评论内容 | content | TEXT | NOT NULL | 支持 @提及 |
| 定位信息 | position | JSONB | | 评论在文档中的位置 |
| 是否已解决 | is_resolved | BOOLEAN | DEFAULT FALSE | |
| 创建时间 | created_at | TIMESTAMP | DEFAULT NOW() | |
| 更新时间 | updated_at | TIMESTAMP | DEFAULT NOW() | |

```sql
CREATE TABLE comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    doc_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    parent_id UUID REFERENCES comments(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    position JSONB,
    is_resolved BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_comments_doc ON comments(doc_id);
CREATE INDEX idx_comments_user ON comments(user_id);
CREATE INDEX idx_comments_parent ON comments(parent_id);
```

#### 4.2.12 通知表 (notifications)

| 中文字段名 | 英文字段名 | 类型 | 约束 | 备注 |
|-----------|-----------|------|------|------|
| 通知ID | id | UUID | PRIMARY KEY | 主键 |
| 用户ID | user_id | UUID | FOREIGN KEY, NOT NULL | 接收者 |
| 通知类型 | type | VARCHAR(50) | NOT NULL | comment/mention/task/share |
| 通知标题 | title | VARCHAR(255) | NOT NULL | |
| 通知内容 | content | TEXT | | |
| 关联资源类型 | resource_type | VARCHAR(50) | | document/comment/task |
| 关联资源ID | resource_id | UUID | | |
| 是否已读 | is_read | BOOLEAN | DEFAULT FALSE | |
| 创建时间 | created_at | TIMESTAMP | DEFAULT NOW() | |

```sql
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type VARCHAR(50) NOT NULL,
    title VARCHAR(255) NOT NULL,
    content TEXT,
    resource_type VARCHAR(50),
    resource_id UUID,
    is_read BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_notifications_user ON notifications(user_id);
CREATE INDEX idx_notifications_unread ON notifications(user_id, is_read) WHERE is_read = FALSE;
```

#### 4.2.13 任务表 (tasks)

| 中文字段名 | 英文字段名 | 类型 | 约束 | 备注 |
|-----------|-----------|------|------|------|
| 任务ID | id | UUID | PRIMARY KEY | 主键 |
| 文档ID | doc_id | UUID | FOREIGN KEY | 关联文档 |
| 任务标题 | title | VARCHAR(255) | NOT NULL | |
| 任务描述 | description | TEXT | | |
| 指派人ID | assignee_id | UUID | FOREIGN KEY | 负责人 |
| 创建者ID | created_by | UUID | FOREIGN KEY, NOT NULL | |
| 状态 | status | VARCHAR(20) | DEFAULT 'pending' | pending/in_progress/completed |
| 优先级 | priority | VARCHAR(20) | DEFAULT 'medium' | low/medium/high/urgent |
| 截止日期 | due_date | TIMESTAMP | | |
| 完成时间 | completed_at | TIMESTAMP | | |
| 创建时间 | created_at | TIMESTAMP | DEFAULT NOW() | |

```sql
CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    doc_id UUID REFERENCES documents(id) ON DELETE SET NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    assignee_id UUID REFERENCES users(id),
    created_by UUID NOT NULL REFERENCES users(id),
    status VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending', 'in_progress', 'completed')),
    priority VARCHAR(20) DEFAULT 'medium' CHECK (priority IN ('low', 'medium', 'high', 'urgent')),
    due_date TIMESTAMP,
    completed_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_tasks_assignee ON tasks(assignee_id);
CREATE INDEX idx_tasks_doc ON tasks(doc_id);
CREATE INDEX idx_tasks_status ON tasks(status);
```

#### 4.2.14 操作日志表 (audit_logs)

| 中文字段名 | 英文字段名 | 类型 | 约束 | 备注 |
|-----------|-----------|------|------|------|
| 日志ID | id | UUID | PRIMARY KEY | 主键 |
| 用户ID | user_id | UUID | FOREIGN KEY | 操作者 |
| 操作类型 | action | VARCHAR(50) | NOT NULL | create/update/delete/login |
| 资源类型 | resource_type | VARCHAR(50) | NOT NULL | |
| 资源ID | resource_id | UUID | | |
| 操作详情 | details | JSONB | | 变更前后对比 |
| IP地址 | ip_address | VARCHAR(45) | | IPv6 兼容 |
| 用户代理 | user_agent | VARCHAR(500) | | |
| 创建时间 | created_at | TIMESTAMP | DEFAULT NOW() | |

```sql
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    action VARCHAR(50) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id UUID,
    details JSONB,
    ip_address VARCHAR(45),
    user_agent VARCHAR(500),
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_user ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_resource ON audit_logs(resource_type, resource_id);
CREATE INDEX idx_audit_logs_created ON audit_logs(created_at);
```

---

## 5. API 接口设计

### 5.1 API 设计原则

- RESTful 风格
- 统一响应格式
- JWT Bearer Token 认证
- 版本控制 (v1)

### 5.2 统一响应格式

```json
// 成功响应
{
    "success": true,
    "data": { ... },
    "message": null
}

// 错误响应
{
    "success": false,
    "data": null,
    "message": "错误信息",
    "error_code": "VALIDATION_ERROR"
}

// 分页响应
{
    "success": true,
    "data": {
        "items": [ ... ],
        "total": 100,
        "page": 1,
        "page_size": 20,
        "total_pages": 5
    }
}
```

### 5.3 接口列表

#### 5.3.1 认证模块 (/api/v1/auth)

| 方法 | 路径 | 描述 | 认证 |
|------|------|------|------|
| POST | /register | 用户注册 | ❌ |
| POST | /login | 用户登录 | ❌ |
| POST | /logout | 用户登出 | ✅ |
| POST | /refresh | 刷新 Token | ✅ |
| POST | /forgot-password | 发送密码重置邮件 | ❌ |
| POST | /reset-password | 重置密码 | ❌ |
| POST | /verify-email | 验证邮箱 | ❌ |

**注册请求示例：**
```json
POST /api/v1/auth/register
{
    "email": "user@example.com",
    "password": "SecurePass123!",
    "nickname": "张三"
}
```

**登录响应示例：**
```json
{
    "success": true,
    "data": {
        "access_token": "eyJhbGciOiJIUzI1NiIs...",
        "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
        "token_type": "Bearer",
        "expires_in": 3600,
        "user": {
            "id": "uuid",
            "email": "user@example.com",
            "nickname": "张三",
            "avatar_url": null,
            "role": "editor"
        }
    }
}
```

#### 5.3.2 用户模块 (/api/v1/users)

| 方法 | 路径 | 描述 | 认证 |
|------|------|------|------|
| GET | /me | 获取当前用户信息 | ✅ |
| PUT | /me | 更新当前用户信息 | ✅ |
| POST | /me/avatar | 上传头像 | ✅ |
| PUT | /me/password | 修改密码 | ✅ |
| GET | /{id} | 获取用户公开信息 | ✅ |
| GET | / | 获取用户列表 (管理员) | ✅ Admin |
| PUT | /{id}/role | 修改用户角色 (管理员) | ✅ Admin |
| PUT | /{id}/status | 修改用户状态 (管理员) | ✅ Admin |

#### 5.3.3 文档模块 (/api/v1/documents)

| 方法 | 路径 | 描述 | 认证 |
|------|------|------|------|
| POST | / | 创建文档 | ✅ |
| GET | / | 获取文档列表 | ✅ |
| GET | /{id} | 获取文档详情 | ✅ |
| PUT | /{id} | 更新文档 | ✅ |
| DELETE | /{id} | 删除文档 | ✅ |
| POST | /{id}/duplicate | 复制文档 | ✅ |
| PUT | /{id}/move | 移动文档 | ✅ |
| GET | /{id}/collaborators | 获取协作者列表 | ✅ |
| POST | /{id}/collaborators | 添加协作者 | ✅ |
| PUT | /{id}/collaborators/{userId} | 更新协作者权限 | ✅ |
| DELETE | /{id}/collaborators/{userId} | 移除协作者 | ✅ |
| GET | /{id}/versions | 获取版本历史 | ✅ |
| POST | /{id}/versions | 创建版本快照 | ✅ |
| GET | /{id}/versions/{versionId} | 获取特定版本 | ✅ |
| POST | /{id}/versions/{versionId}/restore | 恢复到特定版本 | ✅ |

**创建文档请求：**
```json
POST /api/v1/documents
{
    "title": "项目计划书",
    "doc_type": "document",
    "folder_id": "uuid-or-null",
    "content": null
}
```

**文档列表响应：**
```json
{
    "success": true,
    "data": {
        "items": [
            {
                "id": "uuid",
                "title": "项目计划书",
                "doc_type": "document",
                "owner": {
                    "id": "uuid",
                    "nickname": "张三",
                    "avatar_url": "..."
                },
                "folder_id": null,
                "is_public": false,
                "collaborator_count": 3,
                "my_role": "editor",
                "created_at": "2024-01-15T10:30:00Z",
                "updated_at": "2024-01-15T14:20:00Z"
            }
        ],
        "total": 25,
        "page": 1,
        "page_size": 20
    }
}
```

#### 5.3.4 文件夹模块 (/api/v1/folders)

| 方法 | 路径 | 描述 | 认证 |
|------|------|------|------|
| POST | / | 创建文件夹 | ✅ |
| GET | / | 获取文件夹树 | ✅ |
| GET | /{id} | 获取文件夹详情 | ✅ |
| PUT | /{id} | 更新文件夹 | ✅ |
| DELETE | /{id} | 删除文件夹 | ✅ |
| GET | /{id}/contents | 获取文件夹内容 | ✅ |

#### 5.3.5 标签模块 (/api/v1/tags)

| 方法 | 路径 | 描述 | 认证 |
|------|------|------|------|
| POST | / | 创建标签 | ✅ |
| GET | / | 获取标签列表 | ✅ |
| PUT | /{id} | 更新标签 | ✅ |
| DELETE | /{id} | 删除标签 | ✅ |
| POST | /documents/{docId}/tags | 给文档添加标签 | ✅ |
| DELETE | /documents/{docId}/tags/{tagId} | 移除文档标签 | ✅ |

#### 5.3.6 评论模块 (/api/v1/comments)

| 方法 | 路径 | 描述 | 认证 |
|------|------|------|------|
| POST | / | 创建评论 | ✅ |
| GET | /documents/{docId} | 获取文档评论 | ✅ |
| PUT | /{id} | 更新评论 | ✅ |
| DELETE | /{id} | 删除评论 | ✅ |
| POST | /{id}/reply | 回复评论 | ✅ |
| PUT | /{id}/resolve | 标记评论已解决 | ✅ |

#### 5.3.7 任务模块 (/api/v1/tasks)

| 方法 | 路径 | 描述 | 认证 |
|------|------|------|------|
| POST | / | 创建任务 | ✅ |
| GET | / | 获取任务列表 | ✅ |
| GET | /{id} | 获取任务详情 | ✅ |
| PUT | /{id} | 更新任务 | ✅ |
| DELETE | /{id} | 删除任务 | ✅ |
| PUT | /{id}/status | 更新任务状态 | ✅ |
| PUT | /{id}/assign | 分配任务 | ✅ |

#### 5.3.8 通知模块 (/api/v1/notifications)

| 方法 | 路径 | 描述 | 认证 |
|------|------|------|------|
| GET | / | 获取通知列表 | ✅ |
| GET | /unread-count | 获取未读数量 | ✅ |
| PUT | /{id}/read | 标记为已读 | ✅ |
| PUT | /read-all | 全部标记已读 | ✅ |
| DELETE | /{id} | 删除通知 | ✅ |

#### 5.3.9 搜索模块 (/api/v1/search)

| 方法 | 路径 | 描述 | 认证 |
|------|------|------|------|
| GET | /documents | 搜索文档 | ✅ |
| GET | /users | 搜索用户 | ✅ |

**搜索请求参数：**
```
GET /api/v1/search/documents?q=关键词&type=document&owner_id=uuid&tag_id=uuid&created_after=2024-01-01&sort=updated_at&order=desc&page=1&page_size=20
```

#### 5.3.10 WebSocket 实时协作 (/ws)

```
WebSocket /ws/documents/{doc_id}
```

**消息类型：**

```typescript
// 客户端 -> 服务器
interface ClientMessage {
    type: 'sync' | 'awareness' | 'cursor';
    payload: any;
}

// 同步更新 (Yjs Update)
{
    type: 'sync',
    payload: {
        update: Uint8Array  // Yjs 更新数据 (base64)
    }
}

// 用户感知 (在线状态)
{
    type: 'awareness',
    payload: {
        user_id: 'uuid',
        user_name: '张三',
        user_color: '#FF5733'
    }
}

// 光标位置
{
    type: 'cursor',
    payload: {
        position: { line: 10, column: 5 },
        selection: { start: 100, end: 150 }
    }
}

// 服务器 -> 客户端
interface ServerMessage {
    type: 'sync' | 'awareness' | 'cursor' | 'user_joined' | 'user_left' | 'error';
    payload: any;
}

// 用户加入
{
    type: 'user_joined',
    payload: {
        user_id: 'uuid',
        user_name: '张三',
        user_color: '#FF5733'
    }
}

// 用户离开
{
    type: 'user_left',
    payload: {
        user_id: 'uuid'
    }
}
```

---

## 6. 开发任务清单

### 6.1 第一阶段：项目基础 (Week 1)

- [ ] **1.1 项目初始化**
  - [ ] 创建 Cargo workspace
  - [ ] 配置 .gitignore
  - [ ] 设置 CI/CD (GitHub Actions)
  - [ ] 配置 rustfmt 和 clippy

- [ ] **1.2 基础设施搭建**
  - [ ] openGauss 数据库安装与配置
  - [ ] Redis 安装与配置
  - [ ] 配置管理 (环境变量/配置文件)
  - [ ] 日志系统 (tracing)

- [ ] **1.3 数据库迁移**
  - [ ] 安装 sqlx-cli
  - [ ] 编写所有表的迁移脚本
  - [ ] 初始化种子数据

### 6.2 第二阶段：认证与用户 (Week 2)

- [ ] **2.1 认证系统**
  - [ ] JWT 生成与验证
  - [ ] 密码加密 (argon2)
  - [ ] 注册接口
  - [ ] 登录接口
  - [ ] 登出与 Token 刷新
  - [ ] 密码重置流程

- [ ] **2.2 用户管理**
  - [ ] 用户信息 CRUD
  - [ ] 头像上传 (本地存储)
  - [ ] 用户搜索

- [ ] **2.3 权限系统**
  - [ ] RBAC 权限模型实现
  - [ ] 权限中间件
  - [ ] 角色管理接口

### 6.3 第三阶段：文档管理 (Week 3)

- [ ] **3.1 文档 CRUD**
  - [ ] 创建文档
  - [ ] 获取文档列表
  - [ ] 获取文档详情
  - [ ] 更新文档
  - [ ] 删除文档
  - [ ] 文档复制

- [ ] **3.2 文件夹系统**
  - [ ] 文件夹 CRUD
  - [ ] 文件夹树结构
  - [ ] 文档移动

- [ ] **3.3 标签系统**
  - [ ] 标签 CRUD
  - [ ] 文档标签关联

- [ ] **3.4 搜索功能**
  - [ ] 标题搜索
  - [ ] 全文搜索 (PostgreSQL tsvector)
  - [ ] 高级筛选

### 6.4 第四阶段：实时协作 (Week 4-5) ⭐ 核心

- [ ] **4.1 WebSocket 服务**
  - [ ] WebSocket 连接管理
  - [ ] 房间 (Room) 概念实现
  - [ ] 连接认证
  - [ ] 心跳检测

- [ ] **4.2 CRDT 集成**
  - [ ] yrs (Yjs Rust) 集成
  - [ ] 文档状态同步
  - [ ] 更新广播
  - [ ] 状态持久化

- [ ] **4.3 协作功能**
  - [ ] 多用户实时编辑
  - [ ] 冲突自动解决
  - [ ] 光标位置同步
  - [ ] 在线用户显示

- [ ] **4.4 协作者管理**
  - [ ] 邀请协作者
  - [ ] 权限设置
  - [ ] 移除协作者

### 6.5 第五阶段：评论与通知 (Week 5-6)

- [ ] **5.1 评论系统**
  - [ ] 创建评论
  - [ ] 评论回复
  - [ ] @提及解析
  - [ ] 评论位置定位
  - [ ] 标记已解决

- [ ] **5.2 通知系统**
  - [ ] 通知生成
  - [ ] 实时推送 (WebSocket)
  - [ ] 通知列表
  - [ ] 标记已读

- [ ] **5.3 任务系统**
  - [ ] 任务 CRUD
  - [ ] 任务分配
  - [ ] 状态跟踪
  - [ ] 到期提醒

### 6.6 第六阶段：选做功能 (Week 6-7)

- [ ] **6.1 版本控制 (选做)**
  - [ ] 自动版本快照
  - [ ] 版本历史列表
  - [ ] 版本对比
  - [ ] 版本回滚
  - [ ] 版本锁定

- [ ] **6.2 Markdown 支持 (选做)**
  - [ ] Markdown 编辑器
  - [ ] 实时预览
  - [ ] Markdown 导出

- [ ] **6.3 导入导出 (选做)**
  - [ ] Word 导出 (docx)
  - [ ] PDF 导出
  - [ ] Markdown 导出
  - [ ] 批量导出

### 6.7 第七阶段：前端开发 (并行进行)

- [ ] **7.1 基础页面**
  - [ ] 登录/注册页
  - [ ] 首页/仪表盘
  - [ ] 用户设置页

- [ ] **7.2 文档页面**
  - [ ] 文档列表页
  - [ ] 文档编辑器
  - [ ] 文件夹视图
  - [ ] 搜索结果页

- [ ] **7.3 协作功能 UI**
  - [ ] 实时编辑器 (TipTap)
  - [ ] 协作者头像
  - [ ] 光标显示
  - [ ] 评论侧边栏

- [ ] **7.4 其他页面**
  - [ ] 通知中心
  - [ ] 任务列表
  - [ ] 版本历史 (选做)

### 6.8 第八阶段：测试与收尾 (Week 8)

- [ ] **8.1 测试**
  - [ ] 单元测试
  - [ ] 集成测试
  - [ ] API 测试
  - [ ] 压力测试

- [ ] **8.2 文档**
  - [ ] API 文档 (OpenAPI)
  - [ ] 用户手册
  - [ ] 部署文档

- [ ] **8.3 课程报告**
  - [ ] 需求分析 (用例图)
  - [ ] 系统设计
  - [ ] 数据库设计 (ER图)
  - [ ] 功能截图
  - [ ] 系统简介

- [ ] **8.4 答辩准备**
  - [ ] 演示视频录制
  - [ ] 代码复习
  - [ ] PPT 制作

---

## 7. 项目结构

```
entangle/
├── Cargo.toml                    # Workspace 配置
├── Cargo.lock
├── .env.example                  # 环境变量示例
├── .gitignore
├── README.md
│
├── crates/
│   ├── api/                      # HTTP/WebSocket 接口层
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs           # 入口点
│   │       ├── lib.rs
│   │       ├── config.rs         # 配置加载
│   │       ├── routes/           # 路由定义
│   │       │   ├── mod.rs
│   │       │   ├── auth.rs
│   │       │   ├── users.rs
│   │       │   ├── documents.rs
│   │       │   ├── folders.rs
│   │       │   ├── comments.rs
│   │       │   ├── tasks.rs
│   │       │   ├── notifications.rs
│   │       │   └── search.rs
│   │       ├── handlers/         # 请求处理器
│   │       │   └── ...
│   │       ├── middleware/       # 中间件
│   │       │   ├── mod.rs
│   │       │   ├── auth.rs
│   │       │   └── logging.rs
│   │       ├── ws/               # WebSocket 处理
│   │       │   ├── mod.rs
│   │       │   ├── hub.rs        # 连接管理
│   │       │   └── handlers.rs
│   │       ├── dto/              # 数据传输对象
│   │       │   └── ...
│   │       └── error.rs          # 错误处理
│   │
│   ├── core/                     # 业务逻辑层
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── services/         # 业务服务
│   │       │   ├── mod.rs
│   │       │   ├── user_service.rs
│   │       │   ├── document_service.rs
│   │       │   ├── collab_service.rs
│   │       │   ├── comment_service.rs
│   │       │   ├── task_service.rs
│   │       │   └── notification_service.rs
│   │       └── domain/           # 领域模型
│   │           └── ...
│   │
│   ├── db/                       # 数据访问层
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── models/           # 数据库模型
│   │       │   ├── mod.rs
│   │       │   ├── user.rs
│   │       │   ├── document.rs
│   │       │   └── ...
│   │       ├── repositories/     # 数据仓库
│   │       │   └── ...
│   │       └── pool.rs           # 连接池
│   │
│   ├── auth/                     # 认证授权
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── jwt.rs
│   │       ├── password.rs
│   │       └── permissions.rs
│   │
│   └── collab/                   # 实时协作 (CRDT)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── document.rs       # Yrs 文档管理
│           ├── awareness.rs      # 用户感知
│           └── sync.rs           # 同步逻辑
│
├── migrations/                   # 数据库迁移
│   ├── 20240101000001_create_users.sql
│   ├── 20240101000002_create_roles.sql
│   └── ...
│
├── frontend/                     # 前端项目
│   ├── package.json
│   ├── vite.config.ts
│   ├── src/
│   │   ├── main.ts
│   │   ├── App.vue
│   │   ├── components/
│   │   ├── views/
│   │   ├── stores/
│   │   ├── api/
│   │   └── utils/
│   └── public/
│
├── docs/                         # 项目文档
│   ├── PROJECT_PLAN.md           # 本文档
│   ├── API.md                    # API 文档
│   ├── DEPLOYMENT.md             # 部署文档
│   └── 课程报告/
│       ├── 需求分析.md
│       ├── 系统设计.md
│       └── ...
│
└── scripts/                      # 脚本
    ├── setup.sh                  # 环境初始化
    ├── migrate.sh                # 数据库迁移
    └── seed.sh                   # 种子数据
```

---

## 8. 部署方案

### 8.1 开发环境

```yaml
# docker-compose.dev.yml
version: '3.8'

services:
  opengauss:
    image: enmotech/opengauss:5.0.0
    container_name: entangle-db
    environment:
      GS_PASSWORD: "YourSecurePassword123!"
    ports:
      - "5432:5432"
    volumes:
      - opengauss_data:/var/lib/opengauss

  redis:
    image: redis:7-alpine
    container_name: entangle-redis
    ports:
      - "6379:6379"

volumes:
  opengauss_data:
```

### 8.2 生产环境架构

```
┌─────────────────────────────────────────────────────────────────────┐
│                         生产环境部署架构                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│   ┌─────────────┐                                                   │
│   │   Nginx     │  SSL 终止 / 静态资源 / 负载均衡                    │
│   │   (反向代理) │                                                   │
│   └──────┬──────┘                                                   │
│          │                                                          │
│          ├─────────────────┬─────────────────┐                      │
│          ▼                 ▼                 ▼                      │
│   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐              │
│   │  API Node 1 │   │  API Node 2 │   │  API Node 3 │              │
│   │   (Axum)    │   │   (Axum)    │   │   (Axum)    │              │
│   └──────┬──────┘   └──────┬──────┘   └──────┬──────┘              │
│          │                 │                 │                      │
│          └─────────────────┼─────────────────┘                      │
│                            │                                        │
│          ┌─────────────────┼─────────────────┐                      │
│          ▼                 ▼                 ▼                      │
│   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐              │
│   │  openGauss  │   │    Redis    │   │   MinIO     │              │
│   │  (主数据库)  │   │  (缓存/会话) │   │  (文件存储)  │              │
│   └─────────────┘   └─────────────┘   └─────────────┘              │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 8.3 环境变量配置

```env
# .env.example

# 应用配置
APP_NAME=Entangle
APP_ENV=development
APP_PORT=3000
APP_SECRET_KEY=your-secret-key-at-least-32-chars

# 数据库配置
DATABASE_URL=postgres://gaussdb:password@localhost:5432/entangle
DATABASE_MAX_CONNECTIONS=10

# Redis 配置
REDIS_URL=redis://localhost:6379

# JWT 配置
JWT_SECRET=your-jwt-secret-key
JWT_ACCESS_EXPIRY=3600
JWT_REFRESH_EXPIRY=604800

# 文件存储
STORAGE_TYPE=local
STORAGE_PATH=./uploads
# STORAGE_TYPE=s3
# S3_BUCKET=entangle-files
# S3_REGION=us-east-1

# 邮件配置 (可选)
SMTP_HOST=smtp.example.com
SMTP_PORT=587
SMTP_USER=noreply@example.com
SMTP_PASS=password

# 日志
LOG_LEVEL=info
LOG_FORMAT=json
```

---

## 附录

### A. 技术栈版本

| 依赖 | 版本 | 用途 |
|------|------|------|
| Rust | 1.75+ | 编程语言 |
| Axum | 0.7.x | Web 框架 |
| SeaORM | 0.12.x | ORM |
| SQLx | 0.7.x | 数据库驱动 |
| tokio | 1.35+ | 异步运行时 |
| yrs | 0.18.x | CRDT |
| argon2 | 0.5.x | 密码哈希 |
| jsonwebtoken | 9.x | JWT |
| tracing | 0.1.x | 日志 |
| serde | 1.x | 序列化 |

### B. 参考资源

- [Axum 文档](https://docs.rs/axum/latest/axum/)
- [SeaORM 文档](https://www.sea-ql.org/SeaORM/)
- [Yjs 文档](https://docs.yjs.dev/)
- [yrs 仓库](https://github.com/y-crdt/y-crdt)
- [openGauss 文档](https://opengauss.org/zh/docs/)

### C. 开发规范

- 代码风格：遵循 rustfmt 默认配置
- 提交规范：Conventional Commits
- 分支策略：Git Flow
- 代码审查：PR 必须经过至少一人审查

---

*文档版本: 1.0.0*
*最后更新: 2024年1月*
