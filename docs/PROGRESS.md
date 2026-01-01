# Entangle 项目开发进度

> 最后更新: 2026-01-01

## 📊 总体进度

- **已完成**: 5/8 阶段 (62.5%)
- **进行中**: 1/8 阶段
- **待开始**: 2/8 阶段

---

## ✅ 已完成阶段

### 阶段 1: 项目基础 (100%)

**完成时间**: 2024-12-31

**核心成果**:
- ✅ Cargo workspace 配置
- ✅ openGauss 数据库集成
- ✅ 日志系统 (tracing)
- ✅ 环境变量配置

**文档**:
- `docs/QUICK_START.md` - 快速开始指南
- `README.md` - 项目说明

---

### 阶段 2: 认证与用户系统 (100%)

**完成时间**: 2024-12-31

**核心成果**:
- ✅ JWT 认证系统
- ✅ Argon2 密码哈希
- ✅ RBAC 权限模型 (3角色 + 6权限)
- ✅ 用户注册/登录接口
- ✅ 权限中间件

**技术栈**:
- `entangle-auth` crate
- JWT + Argon2
- PostgreSQL

**API 端点**:
- `POST /api/auth/register` - 用户注册
- `POST /api/auth/login` - 用户登录
- `GET /api/auth/profile` - 获取个人资料

**测试**:
- ✅ 所有认证功能测试通过

**文档**:
- `docs/AUTH_README.md` - 认证系统文档

---

### 阶段 2.5: 文档核心功能 (100%)

**完成时间**: 2024-12-31

**核心成果**:
- ✅ 文档 CRUD 操作
- ✅ 公开/私有文档
- ✅ 文档协作者管理 (read/write/admin)
- ✅ 双层权限控制 (RBAC + 文档级)

**数据库**:
- `documents` 表
- `document_collaborators` 表

**API 端点**:
- `POST /api/documents` - 创建文档
- `GET /api/documents/:id` - 获取文档
- `PUT /api/documents/:id` - 更新文档
- `DELETE /api/documents/:id` - 删除文档
- `GET /api/documents/my` - 我的文档
- `GET /api/documents/accessible` - 可访问文档
- `GET /api/documents/public` - 公开文档
- `POST /api/documents/:id/collaborators` - 添加协作者
- `DELETE /api/documents/:id/collaborators/:id` - 移除协作者

**测试**:
- ✅ `scripts/test_documents.sh` - 文档 CRUD 测试
- ✅ `scripts/test_collaboration.sh` - 协作功能测试

---

### 阶段 4: 实时协作 (CRDT) (70%)

**完成时间**: 2024-12-31 (部分完成)

**已完成**:
- ✅ CRDT 文档管理 (Yrs)
- ✅ 用户感知系统
- ✅ 文档房间管理
- ✅ WebSocket 基础端点
- ✅ API 集成

**待完成**:
- ⏳ 更新广播机制
- ⏳ 心跳检测
- ⏳ 断线重连
- ⏳ 状态持久化到数据库

**技术栈**:
- `entangle-collab` crate
- Yrs (CRDT)
- WebSocket (Axum)

**API 端点**:
- `ws://host/ws/documents/:id` - WebSocket 连接

**测试**:
- ✅ Collab crate 单元测试 (7/7)
- ✅ `scripts/test_websocket.sh` - WebSocket 端点测试

---

### 阶段 3: 文档管理扩展 (100%)

**完成时间**: 2026-01-01

**已完成功能**:
- ✅ 文件夹系统 (CRUD + 树结构 + 文档移动)
- ✅ 标签系统 (CRUD + 文档关联 + 筛选)
- ✅ 搜索功能 (标题搜索 + 高级筛选)
- ✅ 文档复制功能

**数据库**:
- `folders` 表
- `tags` 表
- `document_tags` 表

**API 端点**:
- `POST /api/folders` - 创建文件夹
- `GET /api/folders/:id` - 获取文件夹
- `PUT /api/folders/:id` - 更新文件夹
- `DELETE /api/folders/:id` - 删除文件夹
- `GET /api/folders/tree` - 获取文件夹树
- `POST /api/tags` - 创建标签
- `GET /api/tags` - 获取标签列表
- `PUT /api/tags/:id` - 更新标签
- `DELETE /api/tags/:id` - 删除标签
- `GET /api/documents/search` - 搜索文档
- `POST /api/documents/:id/duplicate` - 复制文档

**设计文档**:
- ✅ `docs/FOLDER_DESIGN.md` - 文件夹系统设计
- ✅ `docs/FOLDER_USAGE.md` - 文件夹使用文档
- ✅ `docs/TAG_DESIGN.md` - 标签系统设计
- ✅ `docs/TAG_USAGE.md` - 标签使用文档

**测试脚本**:
- ✅ `scripts/test_folders.sh` - 文件夹功能测试
- ✅ `scripts/test_tags.sh` - 标签功能测试

---

### 阶段 7: 前端开发 (Leptos + WASM) (100%)

**完成时间**: 2026-01-01

**核心成果**:
- ✅ Leptos 框架集成
- ✅ WebAssembly 构建配置
- ✅ 用户认证页面 (登录/注册)
- ✅ 文档管理页面 (列表/卡片展示)
- ✅ 文档编辑器页面
- ✅ 文件夹侧边栏 (创建/显示)
- ✅ 标签侧边栏 (创建/显示)
- ✅ 响应式 UI 设计

**技术栈**:
- Leptos 0.6 (Rust 全栈框架)
- WebAssembly (wasm32-unknown-unknown)
- Trunk (WASM 打包工具)
- gloo-net (HTTP 客户端)

**实现的页面**:
- `/` - 登录页面
- `/register` - 注册页面
- `/documents` - 文档列表页面（带侧边栏）
- `/editor/:id` - 文档编辑器页面

**API 集成**:
- ✅ 用户注册/登录
- ✅ 文档 CRUD 操作
- ✅ 文件夹管理
- ✅ 标签管理
- ✅ Token 管理 (LocalStorage)

**构建配置**:
- ✅ Nix Flakes 前端工具链
- ✅ wasm32 target 配置
- ✅ Trunk 构建脚本
- ✅ 生产优化配置

**文档**:
- ✅ `docs/FRONTEND_GUIDE.md` - 前端开发指南
- ✅ `docs/FRONTEND_SETUP.md` - 环境配置说明

---

## 🔄 进行中阶段

### 阶段 4: 实时协作 (CRDT) (70%)

**当前进展**: 基础功能完成，需完善广播和心跳

**已完成**:
- ✅ CRDT 文档管理 (Yrs)
- ✅ 用户感知系统
- ✅ 文档房间管理
- ✅ WebSocket 基础端点
- ✅ API 集成

**待完成**:
- ⏳ 更新广播机制
- ⏳ 心跳检测
- ⏳ 断线重连
- ⏳ 状态持久化到数据库

**技术栈**:
- `entangle-collab` crate
- Yrs (CRDT)
- WebSocket (Axum)

**API 端点**:
- `ws://host/ws/documents/:id` - WebSocket 连接

**测试**:
- ✅ Collab crate 单元测试 (7/7)
- ✅ `scripts/test_websocket.sh` - WebSocket 端点测试

---

## ⏳ 待开始阶段

### 阶段 5: 评论与通知 (0%)

**功能列表**:
- 评论系统 (创建/回复/@提及)
- 通知系统 (实时推送/列表)
- 任务系统 (创建/分配/跟踪)

---

### 阶段 6: 版本控制 (选做) (0%)

**功能列表**:
- 版本快照
- 版本对比
- 版本回滚

---

### 阶段 8: 测试与收尾 (0%)

**功能列表**:
- 集成测试
- API 文档 (OpenAPI)
- 部署文档
- 课程报告

---

## 📈 技术债务

### 高优先级
- [ ] 实现 WebSocket 消息广播机制
- [ ] 添加心跳检测避免连接超时
- [ ] CRDT 状态持久化到数据库

### 中优先级
- [ ] 前端集成 WebSocket 实时协作
- [ ] 完善错误处理和日志
- [ ] 添加 API 速率限制
- [ ] 优化数据库查询性能

### 低优先级
- [ ] 代码覆盖率测试
- [ ] 性能基准测试
- [ ] Docker 容器化

---

## 🎯 下一步行动

### 立即执行
1. 完善 WebSocket 广播机制
2. 添加心跳检测和断线重连
3. 实现 CRDT 状态持久化

### 本周目标
- 完成阶段 4: 实时协作系统 (100%)
- 开始评论与通知系统设计

### 本月目标
- 完成阶段 5: 评论与通知系统
- 开始集成测试和文档编写

---

## 📝 最新提交

- `feat: 实现文档搜索和复制功能` (2026-01-01)
- `feat: 完成 Leptos 前端基础功能` (2026-01-01)
- `feat: 配置前端构建环境` (2026-01-01)
- `afdffb5: feat: 实现完整的文件夹管理系统` (2025-12-31)
- `a8a75bf: feat: 实现完整的标签系统` (2025-12-31)

---

## 📚 相关文档

### 项目规划
- [项目计划](PROJECT_PLAN.md)
- [快速开始](QUICK_START.md)

### 功能文档
- [认证系统](AUTH_README.md)
- [文件夹系统](FOLDER_DESIGN.md)
- [标签系统](TAG_DESIGN.md)
- [测试文档](TESTING.md)

### 前端文档
- [前端开发指南](FRONTEND_GUIDE.md)
- [前端环境配置](FRONTEND_SETUP.md)
