# Entangle 项目开发进度

> 最后更新: 2026-01-04

## 📊 总体进度

- **已完成**: 8/8 阶段 (100%)
- **进行中**: 0/8 阶段
- **待开始**: 0/8 阶段

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

### 阶段 4: 实时协作 (CRDT) (100%)

**完成时间**: 2026-01-01

**核心成果**:
- ✅ CRDT 文档管理 (Yrs)
- ✅ 用户感知系统 (光标、选区、在线状态)
- ✅ 文档房间管理
- ✅ WebSocket 端点
- ✅ **消息广播机制** (tokio::sync::broadcast)
- ✅ **心跳检测** (30秒 Ping/Pong)
- ✅ **断线处理** (自动清理用户状态)
- ✅ **CRDT 状态持久化** (自动保存到数据库)
- ✅ **优雅关闭** (保存所有脏文档)

**技术栈**:
- `entangle-collab` crate
- Yrs 0.18 (CRDT)
- WebSocket (Axum)
- tokio::sync::broadcast (消息广播)

**数据库更新**:
- 新增 `crdt_state` 字段 (BYTEA 类型)
- 自动同步 CRDT 状态和文本内容

**API 端点**:
- `ws://host/ws/documents/:id` - WebSocket 连接

**消息类型**:
```json
{"type": "sync", "update": "<hex_encoded_update>"}
{"type": "awareness", "state": {...}}
{"type": "user_joined", "user_id": "...", "nickname": "..."}
{"type": "user_left", "user_id": "..."}
{"type": "error", "message": "..."}
```

**测试**:
- ✅ Collab crate 单元测试 (8/8 通过)
- ✅ 广播机制测试
- ✅ `scripts/test_websocket.sh` - WebSocket 端点测试

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

### 阶段 5: 评论与通知 (100%) ✅

**完成时间**: 2026-01-04

**核心成果**:
- ✅ 评论系统 (创建/回复/解决)
- ✅ 通知系统 (创建/列表/已读管理)
- ✅ 任务系统 (CRUD/状态更新/分配)

**数据库**:
- `comments` 表
- `notifications` 表
- `tasks` 表

**API 端点**:
- `POST /api/comments` - 创建评论
- `GET /api/comments/:id` - 获取评论详情
- `PUT /api/comments/:id` - 更新评论
- `DELETE /api/comments/:id` - 删除评论
- `GET /api/comments/:id/replies` - 获取回复
- `PUT /api/comments/:id/resolve` - 标记已解决
- `GET /api/documents/:id/comments` - 获取文档评论
- `GET /api/notifications` - 获取通知列表
- `GET /api/notifications/unread-count` - 未读数量
- `PUT /api/notifications/:id/read` - 标记已读
- `PUT /api/notifications/read-all` - 全部已读
- `DELETE /api/notifications/:id` - 删除通知
- `POST /api/tasks` - 创建任务
- `GET /api/tasks` - 获取任务列表
- `GET /api/tasks/:id` - 获取任务详情
- `PUT /api/tasks/:id` - 更新任务
- `DELETE /api/tasks/:id` - 删除任务
- `PUT /api/tasks/:id/status` - 更新状态
- `PUT /api/tasks/:id/assign` - 分配任务
- `GET /api/documents/:id/tasks` - 获取文档任务

---

### 阶段 6: 版本控制 (100%) ✅

**完成时间**: 2026-01-04

**核心成果**:
- ✅ 版本快照 (创建/列表/详情)
- ✅ 版本对比 (逐行差异对比)
- ✅ 版本回滚 (自动备份+恢复)

**数据库**:
- `document_versions` 表

**API 端点**:
- `POST /api/versions` - 创建版本快照
- `GET /api/versions/:id` - 获取版本详情
- `DELETE /api/versions/:id` - 删除版本
- `POST /api/versions/:id/rollback` - 回滚到指定版本
- `GET /api/versions/:a/compare/:b` - 对比两个版本
- `GET /api/documents/:id/versions` - 获取文档版本列表

---

## ⏳ 待开始阶段

### 阶段 8: 测试与收尾 (0%)

**功能列表**:
- 集成测试
- API 文档 (OpenAPI)
- 部署文档
- 课程报告

---

## 📈 技术债务

### 高优先级
- [x] ~~实现 WebSocket 消息广播机制~~
- [x] ~~添加心跳检测避免连接超时~~
- [x] ~~CRDT 状态持久化到数据库~~
- [x] ~~**前端 CRDT 集成到编辑器**~~ (已完成 100%)

### 中优先级
- [ ] 前端实时协作功能测试
- [ ] 完成 Awareness 状态管理（光标同步）
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
1. **测试前端 CRDT 实时协作**
   - [ ] 启动后端和前端服务器
   - [ ] 执行单用户测试
   - [ ] 执行多用户实时协作测试
   - [ ] 验证冲突自动解决
2. 开始 Awareness 状态管理（光标同步）

### 本周目标
- 完成前端实时协作功能测试
- 修复测试中发现的问题
- 实现多用户光标显示

### 本月目标
- ✅ 完成阶段 5: 评论与通知系统
- ✅ 完成阶段 6: 版本控制系统
- 开始集成测试和文档编写

---

## 📝 最新提交

- `feat: 实现版本控制系统` (2026-01-04)
  - 版本快照 (创建/列表/详情)
  - 版本对比 (逐行差异对比)
  - 版本回滚 (自动备份+恢复)
  - 创建 document_versions 表
  - 6 个 API 端点
- `feat: 实现评论、通知和任务系统` (2026-01-04)
  - 完成评论系统 API (创建/回复/解决)
  - 完成通知系统 API (列表/已读管理)
  - 完成任务系统 API (CRUD/状态/分配)
  - 创建数据库迁移脚本
  - 集成到主路由
- `feat: 完整集成前端 CRDT 实时协作` (2026-01-03)
  - 修改后端 DocumentResponse 包含 crdt_state
  - 实现 CrdtManager 集成到 EditorPage
  - 本地编辑 -> CRDT 更新 -> WebSocket 发送
  - WebSocket 接收 -> CRDT 应用 -> UI 更新
  - 完整的 CRDT 双向同步
  - 创建测试指南文档
- `feat: 添加前端 CRDT 管理器模块` (2026-01-03)
  - 实现 CrdtManager 封装 yrs 库
  - 支持文档状态初始化和更新
  - 添加防循环更新机制
  - 完整的单元测试覆盖
- `feat: 完成实时协作系统` (2026-01-01)
  - 实现消息广播机制
  - 添加心跳检测
  - CRDT 状态持久化
  - 优雅关闭处理
- `feat: 完成 Leptos 前端基础功能` (2026-01-01)
- `feat: 配置前端构建环境` (2026-01-01)
- `afdffb5: feat: 实现完整的文件夹管理系统` (2025-12-31)
- `a8a75bf: feat: 实现完整的标签系统` (2025-12-31)

---

## 📚 相关文档

### 文档索引
- [文档中心](README.md) - 完整文档导航

### 入门指南
- [开发者指南](DEVELOPMENT.md) - 编译、调试、部署
- [快速开始](QUICK_START.md) - 项目概述
- [测试指南](TESTING.md) - 测试用例

### 功能模块
- [认证系统](AUTH_README.md) - JWT/RBAC 认证
- [文件夹系统](FOLDER_SYSTEM.md) - 层级文件夹管理
- [标签系统](TAG_SYSTEM.md) - 标签管理和筛选
- [前端开发](FRONTEND.md) - Leptos/WASM 前端
- [实时协作（前端）](REALTIME_COLLAB_FRONTEND.md) - CRDT/WebSocket 前端实现
- [CRDT 测试指南](TESTING_CRDT.md) - 实时协作功能测试

### 项目规划
- [项目计划](PROJECT_PLAN.md) - 总体规划
- [课程报告](COURSE_REPORT.md) - 大作业报告
