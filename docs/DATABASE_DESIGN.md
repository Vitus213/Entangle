# Entangle 数据库设计文档

> openGauss/PostgreSQL 数据库表结构和关系说明
>
> **答辩重点章节** - 本文档详细说明系统的数据库架构设计

---

## 目录

1. [数据库架构概览](#1-数据库架构概览)
2. [建表流程与迁移顺序](#2-建表流程与迁移顺序)
3. [表结构详解](#3-表结构详解)
4. [表关系与ER图](#4-表关系与er图)
5. [索引与约束设计](#5-索引与约束设计)
6. [Rust模型映射](#6-rust模型映射)
7. [初始化数据](#7-初始化数据)
8. [查询示例](#8-查询示例)

---

## 1. 数据库架构概览

### 1.1 技术选型

| 组件 | 技术 | 说明 |
|------|------|------|
| **数据库** | openGauss 5.1.0 | PostgreSQL 兼容，国产数据库 |
| **连接池** | SQLx (异步) | Rust 异步 SQL 工具包 |
| **迁移工具** | SQLx CLI | 数据库版本管理 |
| **主键类型** | UUID | 应用层生成，确保分布式兼容 |

### 1.2 数据库连接

```bash
# 环境变量配置
DATABASE_URL=postgres://omm:password@localhost:5432/postgres
```

### 1.3 数据库分层架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        应用层 (Rust)                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │   Auth      │  │  Document   │  │  Realtime   │             │
│  │   Service   │  │   Service   │  │   Service   │             │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘             │
└─────────┼────────────────┼────────────────┼────────────────────┘
          │                │                │
          ▼                ▼                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      数据访问层 (SQLx)                          │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              模型层 (models/)                            │   │
│  │  User │ Document │ Folder │ Tag │ Comment │ Task...     │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────────────┐
│                    openGauss 数据库                             │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐       │
│  │  users │ │documents│ │folders │ │  tags  │ │comments│       │
│  └────────┘ └────────┘ └────────┘ └────────┘ └────────┘       │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐                 │
│  │ roles  │ │permissions│tasks │ │versions │ │notifications│ │
│  └────────┘ └────────┘ └────────┘ └────────┘                 │
└─────────────────────────────────────────────────────────────────┘
```

### 1.4 表分类

| 分类 | 表名 | 功能 |
|------|------|------|
| **用户与权限** | users, roles, permissions, role_permissions | 用户认证与RBAC权限控制 |
| **文档核心** | documents, document_collaborators | 文档存储与协作 |
| **组织管理** | folders, tags, document_tags | 文件夹与标签分类 |
| **交互功能** | comments, notifications, tasks | 评论、通知、任务 |
| **版本控制** | document_versions | 文档版本历史 |

---

## 2. 建表流程与迁移顺序

### 2.1 迁移文件列表

数据库通过 SQLx 迁移文件按顺序创建，文件命名格式：`{timestamp}_{description}.sql`

```
migrations/
├── 20251231160721_enable_extensions.sql              # ① 启用扩展
├── 20251231160724_create_users_table.sql             # ② 创建用户表
├── 20251231160722_create_roles_table.sql             # ③ 创建角色表
├── 20251231160723_create_permissions_table.sql       # ④ 创建权限表
├── 20251231160725_create_role_permissions_table.sql  # ⑤ 角色权限关联
├── 20251231160726_create_documents_table.sql         # ⑥ 创建文档表
├── 20260101024042_create_folders_table.sql           # ⑦ 创建文件夹表
├── 20260101081500_create_tags_tables.sql             # ⑧ 创建标签表
├── 20251231160727_create_document_collaborators_table.sql  # ⑨ 协作者表
├── 20260101120000_add_crdt_state_to_documents.sql   # ⑩ 添加CRDT状态
├── 20260104121734_create_comments_table.sql         # ⑪ 创建评论表
├── 20260104121735_create_notifications_table.sql    # ⑫ 创建通知表
├── 20260104121736_create_tasks_table.sql            # ⑬ 创建任务表
├── 20260104150000_create_versions_table.sql         # ⑭ 创建版本表
└── 20260105120000_create_super_admin.sql            # ⑮ 创建超级管理员
```

### 2.2 建表依赖关系

```
          ┌─────────────┐
          │   users     │ ◄─── 基础表，最先创建
          └──────┬──────┘
                 │
      ┌──────────┼──────────┐
      ▼          ▼          ▼
┌─────────┐ ┌────────┐ ┌──────────┐
│ folders │ │ documents│ │   tags   │
└─────────┘ └────┬───┘ └──────────┘
                 │
      ┌──────────┼──────────┬──────────┐
      ▼          ▼          ▼          ▼
┌─────────┐ ┌────────┐ ┌──────────┐ ┌──────────┐
│comments │ │ tasks  │ │versions  │ │collaborators│
└─────────┘ └────────┘ └──────────┘ └──────────┘
```

### 2.3 执行迁移命令

```bash
# 运行所有迁移
sqlx migrate run

# 查看迁移状态
sqlx migrate info

# 回滚最后一次迁移
sqlx migrate revert
```

---

## 3. 表结构详解

> 本章节按照建表顺序详细说明每个表的SQL定义

### 3.1 users (用户表) 【基础表】

**文件**: `migrations/20251231160724_create_users_table.sql`

```sql
CREATE TABLE users (
    id              UUID         PRIMARY KEY,           -- 主键，应用层生成UUID
    email           VARCHAR(255) UNIQUE NOT NULL,       -- 邮箱，唯一
    phone           VARCHAR(20)  UNIQUE,                -- 手机号（可选）
    password_hash   VARCHAR(255) NOT NULL,              -- Argon2id 哈希密码
    nickname        VARCHAR(100) NOT NULL,              -- 显示昵称
    avatar_url      VARCHAR(500),                       -- 头像 URL
    role_id         UUID         REFERENCES roles(id),  -- 外键 → roles
    email_verified  BOOLEAN      DEFAULT FALSE,         -- 邮箱是否验证
    status          VARCHAR(20)  DEFAULT 'active'       -- 状态：active/disabled/deleted
        CHECK (status IN ('active', 'disabled', 'deleted')),
    last_login_at   TIMESTAMPTZ,                        -- 最后登录时间
    created_at      TIMESTAMPTZ  DEFAULT NOW(),         -- 创建时间
    updated_at      TIMESTAMPTZ  DEFAULT NOW()          -- 更新时间
);

-- 性能索引
CREATE INDEX idx_users_email   ON users(email);
CREATE INDEX idx_users_phone   ON users(phone);
CREATE INDEX idx_users_role    ON users(role_id);
CREATE INDEX idx_users_status  ON users(status);
```

**字段说明**

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | UUID | PRIMARY KEY | 主键，应用层使用 `uuid::Uuid::new_v4()` 生成 |
| email | VARCHAR(255) | UNIQUE NOT NULL | 登录用邮箱，必须唯一 |
| phone | VARCHAR(20) | UNIQUE | 手机号，可选 |
| password_hash | VARCHAR(255) | NOT NULL | Argon2id 哈希，非明文存储 |
| nickname | VARCHAR(100) | NOT NULL | 用户显示名称 |
| avatar_url | VARCHAR(500) | | 头像链接 |
| role_id | UUID | FOREIGN KEY | 关联角色表 |
| email_verified | BOOLEAN | DEFAULT FALSE | 邮箱验证状态 |
| status | VARCHAR(20) | CHECK | active/disabled/deleted |
| last_login_at | TIMESTAMPTZ | | 最后登录时间戳 |
| created_at | TIMESTAMPTZ | DEFAULT NOW() | 创建时间 |
| updated_at | TIMESTAMPTZ | DEFAULT NOW() | 更新时间 |

---

### 3.2 roles (角色表) 【RBAC基础】

**文件**: `migrations/20251231160722_create_roles_table.sql`

```sql
CREATE TABLE roles (
    id          UUID         PRIMARY KEY,
    name        VARCHAR(50)  UNIQUE NOT NULL,   -- 角色名称：admin/editor/viewer
    description TEXT,                            -- 角色描述
    is_system   BOOLEAN      DEFAULT FALSE,      -- 是否系统内置角色
    created_at  TIMESTAMPTZ  DEFAULT NOW()
);

-- 插入默认系统角色（使用固定UUID便于开发）
INSERT INTO roles (id, name, description, is_system) VALUES
    ('00000000-0000-0000-0000-000000000001'::UUID, 'admin',  '系统管理员，拥有所有权限', TRUE),
    ('00000000-0000-0000-0000-000000000002'::UUID, 'editor', '编辑者，可以创建和编辑文档', TRUE),
    ('00000000-0000-0000-0000-000000000003'::UUID, 'viewer', '查看者，只能查看文档', TRUE);

CREATE INDEX idx_roles_name ON roles(name);
```

**预置角色说明**

| 角色名 | UUID | 权限级别 | 说明 |
|--------|------|----------|------|
| admin | `00000000-0000-0000-0000-000000000001` | 最高 | 系统管理员，全部权限 |
| editor | `00000000-0000-0000-0000-000000000002` | 中等 | 可创建和编辑文档 |
| viewer | `00000000-0000-0000-0000-000000000003` | 最低 | 只读权限 |

---

### 3.3 permissions (权限表) 【RBAC基础】

**文件**: `migrations/20251231160723_create_permissions_table.sql`

```sql
CREATE TABLE permissions (
    id          UUID         PRIMARY KEY,
    name        VARCHAR(100) UNIQUE NOT NULL,   -- 权限名称：document:create
    resource    VARCHAR(50)  NOT NULL,          -- 资源类型：document/user/system
    action      VARCHAR(50)  NOT NULL,          -- 操作类型：create/read/update/delete
    description TEXT                            -- 权限描述
);

-- 插入默认权限
INSERT INTO permissions (id, name, resource, action, description) VALUES
    ('10000000-0000-0000-0000-000000000001'::UUID, 'document:create', 'document', 'create', '创建文档'),
    ('10000000-0000-0000-0000-000000000002'::UUID, 'document:read',   'document', 'read',    '查看文档'),
    ('10000000-0000-0000-0000-000000000003'::UUID, 'document:update', 'document', 'update',  '编辑文档'),
    ('10000000-0000-0000-0000-000000000004'::UUID, 'document:delete', 'document', 'delete',  '删除文档'),
    ('10000000-0000-0000-0000-000000000005'::UUID, 'user:manage',     'user',     'manage',  '管理用户'),
    ('10000000-0000-0000-0000-000000000006'::UUID, 'system:config',   'system',   'config',  '系统配置');

CREATE INDEX idx_permissions_resource ON permissions(resource);
```

**权限设计**

| 权限名 | 资源 | 操作 | 说明 |
|--------|------|------|------|
| document:create | document | create | 创建新文档 |
| document:read | document | read | 查看文档内容 |
| document:update | document | update | 编辑文档 |
| document:delete | document | delete | 删除文档 |
| user:manage | user | manage | 管理用户（管理员专用） |
| system:config | system | config | 系统配置（管理员专用） |

---

### 3.4 role_permissions (角色权限关联表) 【RBAC关联】

**文件**: `migrations/20251231160725_create_role_permissions_table.sql`

```sql
-- 多对多关联表：一个角色有多个权限，一个权限可属于多个角色
CREATE TABLE role_permissions (
    role_id       UUID REFERENCES roles(id)       ON DELETE CASCADE,
    permission_id UUID REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

-- 为 admin 角色分配所有权限
INSERT INTO role_permissions (role_id, permission_id)
SELECT '00000000-0000-0000-0000-000000000001'::UUID, id FROM permissions;

-- 为 editor 角色分配文档操作权限
INSERT INTO role_permissions (role_id, permission_id)
SELECT '00000000-0000-0000-0000-000000000002'::UUID, id
FROM permissions WHERE resource = 'document';

-- 为 viewer 角色只分配读权限
INSERT INTO role_permissions (role_id, permission_id)
SELECT '00000000-0000-0000-0000-000000000003'::UUID, id
FROM permissions WHERE resource = 'document' AND action = 'read';

CREATE INDEX idx_role_permissions_role ON role_permissions(role_id);
CREATE INDEX idx_role_permissions_perm ON role_permissions(permission_id);
```

**RBAC权限模型**

```
┌─────────────────────────────────────────────────────────────┐
│                      RBAC 权限模型                          │
│                                                             │
│   用户 ──► 角色 ──► 角色权限关联 ──► 权限                   │
│   User     Role    Role_Permissions   Permission            │
│                                                             │
│   ┌─────┐   ┌─────┐   ┌─────────────┐   ┌──────────────┐  │
│   │张三 │──►│admin│──►│所有权限     │──►│document:*    │  │
│   └─────┘   └─────┘   │             │   │user:manage   │  │
│                        └─────────────┘   └──────────────┘  │
│                                                             │
│   ┌─────┐   ┌─────┐                                     │  │
│   │李四 │──►│viewer│                                   │  │
│   └─────┘   └─────┘                                     │  │
└─────────────────────────────────────────────────────────────┘
```

---

### 3.5 documents (文档表) 【核心表】

**文件**: `migrations/20251231160726_create_documents_table.sql`

```sql
CREATE TABLE documents (
    id          UUID          PRIMARY KEY,
    title       VARCHAR(255)  NOT NULL,                      -- 文档标题
    content     TEXT          NOT NULL DEFAULT '',           -- 文档内容（纯文本备份）
    owner_id    UUID          NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    is_public   BOOLEAN       NOT NULL DEFAULT FALSE,        -- 是否公开
    folder_id   UUID          REFERENCES folders(id) ON DELETE SET NULL,  -- 所属文件夹
    created_at  TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ   NOT NULL DEFAULT NOW()
);

-- 性能索引
CREATE INDEX idx_documents_owner     ON documents(owner_id);
CREATE INDEX idx_documents_created_at ON documents(created_at DESC);
CREATE INDEX idx_documents_updated_at ON documents(updated_at DESC);
CREATE INDEX idx_documents_public    ON documents(is_public) WHERE is_public = TRUE;
```

**后续添加字段** (`20260101120000_add_crdt_state_to_documents.sql`)：

```sql
-- 添加 CRDT 协同状态字段
ALTER TABLE documents ADD COLUMN crdt_state BYTEA;

-- 为有 CRDT 状态的文档创建索引
CREATE INDEX idx_documents_has_crdt ON documents(id) WHERE crdt_state IS NOT NULL;
```

**字段说明**

| 字段 | 类型 | 说明 |
|------|------|------|
| id | UUID | 主键 |
| title | VARCHAR(255) | 文档标题 |
| content | TEXT | 纯文本内容（用于搜索、显示） |
| owner_id | UUID | 所有者 ID，级联删除 |
| is_public | BOOLEAN | 是否公开访问 |
| folder_id | UUID | 所属文件夹，删除文件夹时设为 NULL |
| crdt_state | BYTEA | CRDT 二进制状态，用于协同编辑 |

**为什么同时存储 content 和 crdt_state？**

- `content`: 纯文本格式，便于全文搜索、快速预览、版本对比
- `crdt_state`: CRDT 完整状态，支持多人协同编辑的冲突解决

---

### 3.6 folders (文件夹表) 【树形结构】

**文件**: `migrations/20260101024042_create_folders_table.sql`

```sql
CREATE TABLE folders (
    id          UUID          PRIMARY KEY,
    name        VARCHAR(255)  NOT NULL,
    parent_id   UUID,                                      -- 父文件夹ID（NULL表示根目录）
    owner_id    UUID          NOT NULL,
    created_at  TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ   NOT NULL DEFAULT NOW(),

    -- 约束
    CONSTRAINT folders_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT folders_parent_fk  FOREIGN KEY (parent_id) REFERENCES folders(id) ON DELETE CASCADE,
    CONSTRAINT folders_owner_fk   FOREIGN KEY (owner_id)  REFERENCES users(id)  ON DELETE CASCADE
);

-- 性能优化索引
CREATE INDEX idx_folders_parent        ON folders(parent_id);
CREATE INDEX idx_folders_owner         ON folders(owner_id);
CREATE INDEX idx_folders_owner_parent  ON folders(owner_id, parent_id);

-- 同时为 documents 表添加 folder_id 外键
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'entangle' AND table_name = 'documents' AND column_name = 'folder_id'
    ) THEN
        ALTER TABLE documents ADD COLUMN folder_id UUID;
        ALTER TABLE documents ADD CONSTRAINT documents_folder_fk
            FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE SET NULL;
        CREATE INDEX idx_documents_folder ON documents(folder_id);
    END IF;
END $$;
```

**树形结构示例**

```
根目录 (parent_id = NULL)
├── 工作 (id=1, parent_id=NULL)
│   ├── 项目A (id=2, parent_id=1)
│   │   └── 需求文档 (id=3, parent_id=2)
│   └── 项目B (id=4, parent_id=1)
└── 个人 (id=5, parent_id=NULL)
    └── 日记 (id=6, parent_id=5)
```

---

### 3.7 tags (标签表) + document_tags (关联表)

**文件**: `migrations/20260101081500_create_tags_tables.sql`

```sql
-- 标签表
CREATE TABLE tags (
    id          UUID          PRIMARY KEY,
    name        VARCHAR(50)   NOT NULL,
    color       VARCHAR(7)    NOT NULL DEFAULT '#3B82F6',  -- 十六进制颜色
    owner_id    UUID          NOT NULL,
    created_at  TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ   NOT NULL DEFAULT NOW(),

    -- 约束
    CONSTRAINT tags_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT tags_color_format    CHECK (color ~ '^#[0-9A-Fa-f]{6}$'),
    CONSTRAINT tags_name_owner_unique UNIQUE (name, owner_id),
    CONSTRAINT tags_owner_fk        FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE
);

-- 文档-标签关联表（多对多）
CREATE TABLE document_tags (
    document_id UUID   NOT NULL,
    tag_id      UUID   NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (document_id, tag_id),
    CONSTRAINT document_tags_document_fk FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE,
    CONSTRAINT document_tags_tag_fk      FOREIGN KEY (tag_id)      REFERENCES tags(id)      ON DELETE CASCADE
);

-- 索引
CREATE INDEX idx_tags_owner          ON tags(owner_id);
CREATE INDEX idx_tags_name           ON tags(name);
CREATE INDEX idx_document_tags_document ON document_tags(document_id);
CREATE INDEX idx_document_tags_tag      ON document_tags(tag_id);
```

**多对多关系**

```
┌─────────┐         ┌─────────────────┐         ┌─────┐
│Document │◄───────►│  document_tags  │◄───────►│ Tag │
│         │         │ (关联表)         │         │     │
└─────────┘         └─────────────────┘         └─────┘
```

---

### 3.8 document_collaborators (协作者表) 【权限控制】

**文件**: `migrations/20251231160727_create_document_collaborators_table.sql`

```sql
CREATE TABLE document_collaborators (
    document_id UUID   NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    user_id     UUID   NOT NULL REFERENCES users(id)     ON DELETE CASCADE,
    permission  VARCHAR(20) NOT NULL CHECK (permission IN ('read', 'write', 'admin')),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (document_id, user_id)
);

-- 索引
CREATE INDEX idx_document_collaborators_user     ON document_collaborators(user_id);
CREATE INDEX idx_document_collaborators_document ON document_collaborators(document_id);
```

**协作权限层级**

```
read (只读) < write (编辑) < admin (管理)
    │           │              │
    │           │              └─ 可删除文档、管理协作者
    │           └─ 可编辑内容、添加评论
    └─ 可查看内容、添加评论
```

---

### 3.9 comments (评论表) 【交互功能】

**文件**: `migrations/20260104121734_create_comments_table.sql`

```sql
CREATE TABLE comments (
    id          UUID          PRIMARY KEY,
    doc_id      UUID          NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    user_id     UUID          NOT NULL REFERENCES users(id)     ON DELETE CASCADE,
    parent_id   UUID          REFERENCES comments(id) ON DELETE CASCADE,  -- 父评论（用于回复）
    content     TEXT          NOT NULL,
    position    JSONB,                                                -- 位置信息
    is_resolved BOOLEAN       DEFAULT FALSE,                         -- 是否已解决
    created_at  TIMESTAMPTZ   DEFAULT NOW(),
    updated_at  TIMESTAMPTZ   DEFAULT NOW()
);

-- 索引
CREATE INDEX idx_comments_doc      ON comments(doc_id);
CREATE INDEX idx_comments_user     ON comments(user_id);
CREATE INDEX idx_comments_parent   ON comments(parent_id);
CREATE INDEX idx_comments_resolved ON comments(is_resolved);
```

**嵌套评论结构**

```
主评论 (parent_id = NULL)
├── 回复1 (parent_id = 主评论ID)
│   └── 回复1.1 (parent_id = 回复1ID)
└── 回复2 (parent_id = 主评论ID)
```

**position 字段格式** (JSONB):

```json
{
  "start": 0,
  "end": 10,
  "text": "选中的文本"
}
```

---

### 3.10 notifications (通知表) 【交互功能】

**文件**: `migrations/20260104121735_create_notifications_table.sql`

```sql
CREATE TABLE notifications (
    id             UUID          PRIMARY KEY,
    user_id        UUID          NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type           VARCHAR(50)   NOT NULL,  -- comment, mention, task, share, system
    title          VARCHAR(255)  NOT NULL,
    content        TEXT,
    resource_type  VARCHAR(50),             -- document, comment, task
    resource_id    UUID,
    sender_id      UUID          REFERENCES users(id) ON DELETE SET NULL,  -- 发送者
    is_read        BOOLEAN       DEFAULT FALSE,
    created_at     TIMESTAMPTZ   DEFAULT NOW()
);

-- 索引
CREATE INDEX idx_notifications_user   ON notifications(user_id);
CREATE INDEX idx_notifications_unread ON notifications(user_id, is_read) WHERE is_read = FALSE;
CREATE INDEX idx_notifications_created ON notifications(created_at);
```

**通知类型**

| type | 说明 | 场景 |
|------|------|------|
| comment | 新评论 | 有人评论你的文档 |
| mention | @提醒 | 有人在评论中@你 |
| task | 任务通知 | 任务被分配给你 |
| share | 分享通知 | 有人邀请你协作 |
| system | 系统通知 | 系统公告 |

---

### 3.11 tasks (任务表) 【交互功能】

**文件**: `migrations/20260104121736_create_tasks_table.sql`

```sql
CREATE TABLE tasks (
    id          UUID          PRIMARY KEY,
    doc_id      UUID          REFERENCES documents(id) ON DELETE SET NULL,
    title       VARCHAR(255)  NOT NULL,
    description TEXT,
    assignee_id UUID          REFERENCES users(id) ON DELETE SET NULL,     -- 被分配人
    created_by  UUID          NOT NULL REFERENCES users(id) ON DELETE CASCADE,  -- 创建人
    status      VARCHAR(20)   DEFAULT 'pending'
        CHECK (status IN ('pending', 'in_progress', 'completed', 'cancelled')),
    priority    VARCHAR(20)   DEFAULT 'medium'
        CHECK (priority IN ('low', 'medium', 'high', 'urgent')),
    due_date    TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at  TIMESTAMPTZ   DEFAULT NOW(),
    updated_at  TIMESTAMPTZ   DEFAULT NOW()
);

-- 索引
CREATE INDEX idx_tasks_assignee ON tasks(assignee_id);
CREATE INDEX idx_tasks_doc      ON tasks(doc_id);
CREATE INDEX idx_tasks_status   ON tasks(status);
CREATE INDEX idx_tasks_created_by ON tasks(created_by);
CREATE INDEX idx_tasks_due_date ON tasks(due_date);
```

**任务状态流转**

```
pending (待处理)
    │
    ├─► in_progress (进行中)
    │       │
    │       └─► completed (已完成)
    │
    └─► cancelled (已取消)
```

---

### 3.12 document_versions (版本历史表) 【版本控制】

**文件**: `migrations/20260104150000_create_versions_table.sql`

```sql
CREATE TABLE document_versions (
    id              UUID          PRIMARY KEY,
    doc_id          UUID          NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    version_number  INTEGER       NOT NULL,
    title           VARCHAR(255)  NOT NULL,
    content         TEXT          NOT NULL,
    crdt_state      BYTEA,
    created_by      UUID          NOT NULL REFERENCES users(id) ON DELETE SET NULL,
    description     VARCHAR(500),                     -- 版本描述/备注
    created_at      TIMESTAMPTZ   DEFAULT NOW(),

    -- 确保同一文档的版本号唯一
    UNIQUE(doc_id, version_number)
);

-- 索引
CREATE INDEX idx_versions_doc      ON document_versions(doc_id);
CREATE INDEX idx_versions_created_by ON document_versions(created_by);
CREATE INDEX idx_versions_created_at  ON document_versions(created_at);
```

**版本控制设计**

```
Document (当前版本)
    │
    ├─── Version 3 (最新快照)
    ├─── Version 2
    └─── Version 1 (初始版本)
```

---

## 4. 表关系与ER图

### 4.1 表关系总结

| 表名 | 关联表 | 关系类型 | 说明 |
|------|--------|----------|------|
| **users** | roles | 多对一 | 一个用户属于一个角色 |
| **users** | documents (owner_id) | 一对多 | 一个用户拥有多个文档 |
| **users** | folders (owner_id) | 一对多 | 一个用户拥有多个文件夹 |
| **users** | tags (owner_id) | 一对多 | 一个用户创建多个标签 |
| **users** | comments (user_id) | 一对多 | 一个用户可发表多条评论 |
| **users** | notifications (user_id) | 一对多 | 一个用户可接收多条通知 |
| **users** | tasks (assignee_id) | 一对多 | 一个用户可被分配多个任务 |
| **documents** | users (owner_id) | 多对一 | 一个文档属于一个用户 |
| **documents** | folders (folder_id) | 多对一 | 一个文档属于一个文件夹 |
| **documents** | comments (doc_id) | 一对多 | 一个文档可有多条评论 |
| **documents** | document_collaborators | 一对多 | 一个文档可有多个协作者 |
| **documents** | document_versions | 一对多 | 一个文档可有多个版本 |
| **documents** | document_tags | 一对多 | 一个文档可有多个标签 |
| **documents** | tasks (doc_id) | 一对多 | 一个文档可关联多个任务 |
| **folders** | folders (parent_id) | 自引用 | 树形结构 |
| **comments** | comments (parent_id) | 自引用 | 嵌套回复 |
| **roles** | permissions | 多对多 | 通过 role_permissions 关联 |
| **tags** | documents | 多对多 | 通过 document_tags 关联 |

### 4.2 完整ER图

```
                           ┌─────────────────────────────────────────────────────────────┐
                           │                        openGauss 数据库                      │
                           │                                                             │
                           │  ┌──────────────────┐      ┌──────────────────┐             │
                           │  │     roles        │◄─────┤ role_permissions │◄────┐       │
                           │  ├──────────────────┤      └──────────────────┘     │       │
                           │  │ id (PK)          │                               │       │
                           │  │ name             │                               │       │
                           │  │ is_system        │                               │       │
                           │  └──────────────────┘                               │       │
                           │            ▲                                         │       │
                           │            │                                         │       │
                           │            │ role_id                                 │       │
                           │            │                                         │       │
                           │  ┌─────────┴─────────────────────────────────────┐   │       │
                           │  │                    users                      │   │       │
                           │  │  ┌──────────────────────────────────────────┐ │   │       │
                           │  │  │ id (PK)                                   │ │   │       │
                           │  │  │ email (UNIQUE)                            │ │   │       │
                           │  │  │ password_hash                             │ │   │       │
                           │  │  │ nickname                                  │ │   │       │
                           │  │  │ role_id ─────────────────────────────────┼─┼───┘       │
                           │  │  │ status                                    │ │             │
                           │  │  └──────────────────────────────────────────┘ │             │
                           │  └────────────────────────────────────────────────┘             │
                           │           ▲           ▲           ▲           ▲                │
                           │           │           │           │           │                │
                           │  ┌────────┘           │           │           │                │
                           │  │ owner_id           │           │           │                │
                           │  │                    │           │           │                │
                           │  │ ┌──────────────────┴─┐ ┌───────┴───────────┴───┐            │
                           │  │ │                    │ │                      │            │
                           │  ▼ ▼                    ▼ ▼                      ▼ ▼           │
                           │  ┌──────────────┐   ┌──────────────┐    ┌──────────────┐       │
                           │  │  folders     │   │  documents   │    │    tags      │       │
                           │  ├──────────────┤   ├──────────────┤    ├──────────────┤       │
                           │  │ id (PK)      │   │ id (PK)      │    │ id (PK)      │       │
                           │  │ name         │   │ title        │    │ name         │       │
                           │  │ parent_id ◄──┼───│ folder_id ───┼───►│ color        │       │
                           │  │ owner_id ────┼───│ owner_id ────┼───►│ owner_id ────┼───────┘
                           │  └──────────────┘   │ content      │    └──────────────┘         │
                           │        │            │ crdt_state   │           │                │
                           │        │            └──────────────┘           │                │
                           │        │ parent_id  │                            │                │
                           │        └────────────┼────────────────────────────┘                │
                           │                      ▼                                             │
                           │           ┌─────────────────────────────────────────────────┐     │
                           │           │              document_tags                      │     │
                           │           ├─────────────────────────────────────────────────┤     │
                           │           │ document_id ◄─────────────────────────────────┼─────┘
                           │           │ tag_id ◄───────────────────────────────────────┼─────┐
                           │           └─────────────────────────────────────────────────┘     │
                           │                                                                  │
                           │  ┌─────────────────────────────────────────────────────────┐   │
                           │  │            document_collaborators                       │   │
                           │  ├─────────────────────────────────────────────────────────┤   │
                           │  │ document_id ──────────────────────────────────────────┼───┼───┐
                           │  │ user_id ───────────────────────────────────────────────┼───┼───┼───┐
                           │  │ permission (read/write/admin)                          │   │   │   │   │
                           │  └─────────────────────────────────────────────────────────┘   │   │   │   │
                           │                                                              │   │   │   │
                           │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │   │   │   │
                           │  │   comments   │  │ notifications│  │    tasks     │       │   │   │   │
                           │  ├──────────────┤  ├──────────────┤  ├──────────────┤       │   │   │   │
                           │  │ id (PK)      │  │ id (PK)      │  │ id (PK)      │       │   │   │   │
                           │  │ doc_id ──────┼──│ user_id ─────┼──│ assignee_id ─┼───────┼───┼───┼───┐
                           │  │ user_id ─────┼──│ sender_id    │  │ created_by ──┼───────┼───┼───┼───┤
                           │  │ parent_id ◄──┼──│ is_read      │  │ doc_id ──────┼───────┼───┘   │   │
                           │  │ position      │  │ type         │  │ status       │       │       │   │
                           │  └──────────────┘  └──────────────┘  └──────────────┘       │       │   │
                           │        ▲                                                            │   │
                           │        └────────────────────────────────────────────────────────────┘   │
                           │                                                                      │
                           │  ┌─────────────────────────────────────────────────────────────┐     │
                           │  │              document_versions                                │     │
                           │  ├─────────────────────────────────────────────────────────────┤     │
                           │  │ id (PK)                                                      │     │
                           │  │ doc_id ──────────────────────────────────────────────────────┼─────┘
                           │  │ version_number                                              │
                           │  │ created_by ──────────────────────────────────────────────────┼─────┐
                           │  └─────────────────────────────────────────────────────────────┘     │
                           └──────────────────────────────────────────────────────────────────────┘
```

### 4.3 关键关系说明

**RBAC权限模型**

```
用户 (users) ──► 角色 (roles) ──► 角色权限 (role_permissions) ──► 权限 (permissions)
    │              │                    │                           │
    └──role_id     └──id                └──role_id, permission_id   └──id
```

**文档协作模型**

```
文档所有者 (owner_id) ──► documents ──┬──► 文档夹 (folder_id)
                                    ├──► 协作者 (document_collaborators)
                                    ├──► 标签 (document_tags)
                                    ├──► 评论 (comments)
                                    ├──► 版本 (document_versions)
                                    └──► 任务 (tasks)
```

---

## 5. 索引与约束设计

### 5.1 索引设计原则

| 索引类型 | 用途 | 示例 |
|----------|------|------|
| **单列索引** | 加速单字段查询 | `idx_users_email` |
| **复合索引** | 加速多字段组合查询 | `idx_folders_owner_parent` |
| **唯一索引** | 保证数据唯一性 | `users.email UNIQUE` |
| **部分索引** | 索引满足条件的行 | `WHERE is_public = TRUE` |
| **降序索引** | 优化排序查询 | `created_at DESC` |

### 5.2 完整索引列表

#### users 表

| 索引名 | 字段 | 类型 | 用途 |
|--------|------|------|------|
| PRIMARY | id | 主键 | 唯一标识 |
| idx_users_email | email | 唯一索引 | 登录查询 |
| idx_users_phone | phone | 唯一索引 | 手机号查询 |
| idx_users_role | role_id | 单列 | 按角色筛选用户 |
| idx_users_status | status | 单列 | 按状态筛选用户 |

#### roles 表

| 索引名 | 字段 | 类型 | 用途 |
|--------|------|------|------|
| PRIMARY | id | 主键 | 唯一标识 |
| idx_roles_name | name | 唯一索引 | 角色名查询 |

#### permissions 表

| 索引名 | 字段 | 类型 | 用途 |
|--------|------|------|------|
| PRIMARY | id | 主键 | 唯一标识 |
| idx_permissions_resource | resource | 单列 | 按资源筛选权限 |

#### role_permissions 表

| 索引名 | 字段 | 类型 | 用途 |
|--------|------|------|------|
| PRIMARY | role_id, permission_id | 复合主键 | 关联唯一性 |
| idx_role_permissions_role | role_id | 单列 | 查询角色的权限 |
| idx_role_permissions_perm | permission_id | 单列 | 查询权限所属角色 |

#### documents 表

| 索引名 | 字段 | 类型 | 用途 |
|--------|------|------|------|
| PRIMARY | id | 主键 | 唯一标识 |
| idx_documents_owner | owner_id | 单列 | 查询用户的文档 |
| idx_documents_folder | folder_id | 单列 | 查询文件夹下的文档 |
| idx_documents_created_at | created_at DESC | 降序 | 按创建时间排序 |
| idx_documents_updated_at | updated_at DESC | 降序 | 按更新时间排序 |
| idx_documents_public | is_public | 部分(仅TRUE) | 查询公开文档 |
| idx_documents_has_crdt | id | 部分(非NULL) | 有CRDT状态的文档 |

#### folders 表

| 索引名 | 字段 | 类型 | 用途 |
|--------|------|------|------|
| PRIMARY | id | 主键 | 唯一标识 |
| idx_folders_parent | parent_id | 单列 | 查询子文件夹 |
| idx_folders_owner | owner_id | 单列 | 查询用户的文件夹 |
| idx_folders_owner_parent | owner_id, parent_id | 复合 | 查询用户在特定父目录下的文件夹 |

#### tags 表

| 索引名 | 字段 | 类型 | 用途 |
|--------|------|------|------|
| PRIMARY | id | 主键 | 唯一标识 |
| idx_tags_owner | owner_id | 单列 | 查询用户的标签 |
| idx_tags_name | name | 单列 | 按名称搜索标签 |

#### document_tags 表

| 索引名 | 字段 | 类型 | 用途 |
|--------|------|------|------|
| PRIMARY | document_id, tag_id | 复合主键 | 关联唯一性 |
| idx_document_tags_document | document_id | 单列 | 查询文档的标签 |
| idx_document_tags_tag | tag_id | 单列 | 查询标签下的文档 |

#### document_collaborators 表

| 索引名 | 字段 | 类型 | 用途 |
|--------|------|------|------|
| PRIMARY | document_id, user_id | 复合主键 | 关联唯一性 |
| idx_document_collaborators_user | user_id | 单列 | 查询用户可访问的文档 |
| idx_document_collaborators_document | document_id | 单列 | 查询文档的协作者 |

#### comments 表

| 索引名 | 字段 | 类型 | 用途 |
|--------|------|------|------|
| PRIMARY | id | 主键 | 唯一标识 |
| idx_comments_doc | doc_id | 单列 | 查询文档的评论 |
| idx_comments_user | user_id | 单列 | 查询用户的评论 |
| idx_comments_parent | parent_id | 单列 | 查询评论的回复 |
| idx_comments_resolved | is_resolved | 单列 | 筛选已解决/未解决评论 |

#### notifications 表

| 索引名 | 字段 | 类型 | 用途 |
|--------|------|------|------|
| PRIMARY | id | 主键 | 唯一标识 |
| idx_notifications_user | user_id | 单列 | 查询用户的通知 |
| idx_notifications_unread | user_id, is_read | 部分(未读) | 查询未读通知 |
| idx_notifications_created | created_at | 单列 | 按时间排序 |

#### tasks 表

| 索引名 | 字段 | 类型 | 用途 |
|--------|------|------|------|
| PRIMARY | id | 主键 | 唯一标识 |
| idx_tasks_assignee | assignee_id | 单列 | 查询分配给用户的任务 |
| idx_tasks_doc | doc_id | 单列 | 查询文档关联的任务 |
| idx_tasks_status | status | 单列 | 按状态筛选任务 |
| idx_tasks_created_by | created_by | 单列 | 查询用户创建的任务 |
| idx_tasks_due_date | due_date | 单列 | 按截止日期排序 |

#### document_versions 表

| 索引名 | 字段 | 类型 | 用途 |
|--------|------|------|------|
| PRIMARY | id | 主键 | 唯一标识 |
| idx_versions_doc | doc_id | 单列 | 查询文档的版本历史 |
| idx_versions_created_by | created_by | 单列 | 查询用户创建的版本 |
| idx_versions_created_at | created_at | 单列 | 按时间排序 |

### 5.3 约束设计

#### CHECK 约束

| 表 | 约束名 | 条件 | 说明 |
|----|--------|------|------|
| users | users_status_check | status IN ('active', 'disabled', 'deleted') | 用户状态枚举 |
| folders | folders_name_not_empty | length(trim(name)) > 0 | 文件夹名非空 |
| tags | tags_name_not_empty | length(trim(name)) > 0 | 标签名非空 |
| tags | tags_color_format | color ~ '^#[0-9A-Fa-f]{6}$' | 颜色格式校验 |
| document_collaborators | document_collaborators_permission_check | permission IN ('read', 'write', 'admin') | 协作权限枚举 |
| tasks | tasks_status_check | status IN ('pending', 'in_progress', 'completed', 'cancelled') | 任务状态枚举 |
| tasks | tasks_priority_check | priority IN ('low', 'medium', 'high', 'urgent') | 任务优先级枚举 |

#### FOREIGN KEY 约束与级联规则

| 外键 | 从表 | 引用表 | 级联规则 | 说明 |
|------|------|--------|----------|------|
| users.role_id | users | roles | ON DELETE SET NULL | 删除角色时用户角色设为NULL |
| documents.owner_id | documents | users | ON DELETE CASCADE | 删除用户时级联删除其文档 |
| documents.folder_id | documents | folders | ON DELETE SET NULL | 删除文件夹时文档文件夹设为NULL |
| folders.parent_id | folders | folders | ON DELETE CASCADE | 删除文件夹时级联删除子文件夹 |
| folders.owner_id | folders | users | ON DELETE CASCADE | 删除用户时级联删除其文件夹 |
| tags.owner_id | tags | users | ON DELETE CASCADE | 删除用户时级联删除其标签 |
| comments.doc_id | comments | documents | ON DELETE CASCADE | 删除文档时级联删除评论 |
| comments.user_id | comments | users | ON DELETE CASCADE | 删除用户时级联删除其评论 |
| comments.parent_id | comments | comments | ON DELETE CASCADE | 删除评论时级联删除回复 |
| notifications.user_id | notifications | users | ON DELETE CASCADE | 删除用户时级联删除通知 |
| notifications.sender_id | notifications | users | ON DELETE SET NULL | 删除发送者时设为NULL |
| tasks.doc_id | tasks | documents | ON DELETE SET NULL | 删除文档时任务文档设为NULL |
| tasks.assignee_id | tasks | users | ON DELETE SET NULL | 删除用户时任务被分配人设为NULL |
| tasks.created_by | tasks | users | ON DELETE CASCADE | 删除用户时级联删除其创建的任务 |
| document_versions.doc_id | document_versions | documents | ON DELETE CASCADE | 删除文档时级联删除版本历史 |
| document_versions.created_by | document_versions | users | ON DELETE SET NULL | 删除用户时版本创建者设为NULL |

---

## 6. Rust模型映射

### 6.1 数据流转架构

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Rust 应用层                                │
│                                                                    │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐       │
│  │   API层      │────►│  Service层   │────►│   模型层     │       │
│  │  (handlers)  │     │  (business)  │     │  (models)    │       │
│  └──────────────┘     └──────────────┘     └──────┬───────┘       │
│                                                   │                │
│                                                   ▼                │
│                                    ┌──────────────────────────┐    │
│                                    │   SQLx (数据库驱动)       │    │
│                                    └──────────┬───────────────┘    │
└───────────────────────────────────────────────┼───────────────────┘
                                                │
                                                ▼
                                   ┌────────────────────────────┐
                                   │    openGauss 数据库        │
                                   │    (表结构)                │
                                   └────────────────────────────┘
```

### 6.2 模型类型说明

每个数据库表对应多种 Rust 结构体：

| 模型类型 | 命名模式 | 说明 | 示例 |
|----------|----------|------|------|
| **实体模型** | 表名 | 与数据库表1:1映射，实现FromRow | `User`, `Document` |
| **创建请求** | Create{表名} | 接收创建请求，不含id/时间戳 | `CreateUser`, `CreateDocument` |
| **更新请求** | Update{表名} | 接收更新请求，字段为Option | `UpdateDocument` |
| **响应模型** | {表名}Response | 返回给API客户端，可能包含关联数据 | `UserResponse`, `DocumentResponse` |
| **列表项** | {表名}ListItem | 用于列表展示，通常不含大字段 | `DocumentListItem` |

### 6.3 User 模型映射

**数据库表**: `users`

```sql
CREATE TABLE users (
    id              UUID         PRIMARY KEY,
    email           VARCHAR(255) UNIQUE NOT NULL,
    phone           VARCHAR(20)  UNIQUE,
    password_hash   VARCHAR(255) NOT NULL,
    nickname        VARCHAR(100) NOT NULL,
    avatar_url      VARCHAR(500),
    role_id         UUID         REFERENCES roles(id),
    email_verified  BOOLEAN      DEFAULT FALSE,
    status          VARCHAR(20)  DEFAULT 'active',
    last_login_at   TIMESTAMPTZ,
    created_at      TIMESTAMPTZ  DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  DEFAULT NOW()
);
```

**Rust 模型** (`crates/db/src/models/user.rs`):

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// 实体模型 - 与数据库表1:1映射
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub phone: Option<String>,
    pub password_hash: String,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub role_id: Option<Uuid>,
    pub email_verified: bool,
    pub status: String,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 创建请求 - 不含 id, created_at, updated_at
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub password: String,  // 明文密码，将在服务层哈希
    pub nickname: String,
    pub phone: Option<String>,
}

// 登录请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

// API响应 - 不含敏感字段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub role: Option<String>,      // 从JOIN查询获取
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
}
```

### 6.4 Document 模型映射

**数据库表**: `documents`

**Rust 模型** (`crates/db/src/models/document.rs`):

```rust
// 实体模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Document {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub owner_id: Uuid,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing)]  // 序列化时跳过二进制数据
    #[sqlx(default)]
    pub crdt_state: Option<Vec<u8>>,  // BYTEA → Vec<u8>
}

// 创建请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDocument {
    pub title: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub is_public: bool,
}

// 更新请求 - 字段为Option，支持部分更新
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDocument {
    pub title: Option<String>,
    pub content: Option<String>,
    pub is_public: Option<bool>,
}

// API响应 - 包含所有者信息（需要JOIN查询）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentResponse {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub owner: DocumentOwner,  // 嵌套关联对象
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crdt_state: Option<String>,  // 十六进制编码
}

// 关联的所有者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentOwner {
    pub id: Uuid,
    pub nickname: String,
    pub email: String,
}

// 协作权限枚举
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum CollaboratorPermission {
    #[serde(rename = "read")]
    Read,
    #[serde(rename = "write")]
    Write,
    #[serde(rename = "admin")]
    Admin,
}
```

### 6.5 类型映射表

| SQL类型 | Rust类型 (sqlx) | 说明 |
|---------|-----------------|------|
| UUID | `Uuid` | uuid crate |
| VARCHAR(n) | `String` | 字符串 |
| TEXT | `String` | 长文本 |
| BOOLEAN | `bool` | 布尔值 |
| INTEGER | `i32` / `i64` | 整数 |
| BYTEA | `Vec<u8>` | 二进制数据 |
| TIMESTAMPTZ | `DateTime<Utc>` | 时间戳 |
| JSONB | `serde_json::Value` | JSON数据 |

### 6.6 可空字段处理

| SQL定义 | Rust类型 | 说明 |
|---------|----------|------|
| `field TYPE NOT NULL` | `field: Type` | 非空字段 |
| `field TYPE` | `field: Option<Type>` | 可空字段 |
| `field TYPE DEFAULT NULL` | `field: Option<Type>` | 可空字段 |
| `field TYPE DEFAULT FALSE` | `field: bool` | 有默认值，非空 |

---

## 7. 初始化数据

### 7.1 默认角色

**迁移文件**: `20251231160722_create_roles_table.sql`

```sql
INSERT INTO roles (id, name, description, is_system) VALUES
    ('00000000-0000-0000-0000-000000000001'::UUID, 'admin',  '系统管理员，拥有所有权限', TRUE),
    ('00000000-0000-0000-0000-000000000002'::UUID, 'editor', '编辑者，可以创建和编辑文档', TRUE),
    ('00000000-0000-0000-0000-000000000003'::UUID, 'viewer', '查看者，只能查看文档', TRUE);
```

### 7.2 默认权限

**迁移文件**: `20251231160723_create_permissions_table.sql`

| 权限ID | 权限名 | 资源 | 操作 |
|--------|--------|------|------|
| 1000...001 | document:create | document | create |
| 1000...002 | document:read | document | read |
| 1000...003 | document:update | document | update |
| 1000...004 | document:delete | document | delete |
| 1000...005 | user:manage | user | manage |
| 1000...006 | system:config | system | config |

### 7.3 角色权限分配

**迁移文件**: `20251231160725_create_role_permissions_table.sql`

```
admin  ──► 所有权限 (6个)
editor ──► 文档权限 (4个: create, read, update, delete)
viewer ──► 读权限 (1个: read)
```

### 7.4 超级管理员账户

**迁移文件**: `20260105120000_create_super_admin.sql`

```sql
-- 默认登录凭据
Email:    admin@entangle.local
Password: admin123456
```

**用户信息**:

| 字段 | 值 |
|------|-----|
| id | `00000000-0000-0000-0000-000000000999` |
| email | admin@entangle.local |
| nickname | 超级管理员 |
| role_id | `00000000-0000-0000-0000-000000000001` (admin) |
| password_hash | Argon2id 哈希值 |
| email_verified | TRUE |
| status | active |

**安全提示**: 首次登录后请立即修改默认密码！

---

## 8. 查询示例

### 8.1 获取用户可访问的文档

```sql
-- 包括拥有的文档、有协作权限的文档、公开文档
SELECT DISTINCT d.*
FROM documents d
WHERE d.owner_id = $user_id                    -- 用户拥有的文档
   OR d.id IN (                                 -- 用户有协作权限的文档
       SELECT document_id
       FROM document_collaborators
       WHERE user_id = $user_id
   )
   OR d.is_public = TRUE                         -- 公开文档
ORDER BY d.updated_at DESC;
```

### 8.2 获取文件夹树形结构

```sql
-- 递归查询获取完整的文件夹树
WITH RECURSIVE folder_tree AS (
    -- 基础查询：根文件夹
    SELECT
        id,
        name,
        parent_id,
        owner_id,
        1 as level,
        ARRAY[name] as path                     -- 路径数组
    FROM folders
    WHERE parent_id IS NULL AND owner_id = $user_id

    UNION ALL

    -- 递归查询：子文件夹
    SELECT
        f.id,
        f.name,
        f.parent_id,
        f.owner_id,
        ft.level + 1,
        ft.path || f.name                       -- 追加路径
    FROM folders f
    INNER JOIN folder_tree ft ON f.parent_id = ft.id
)
SELECT * FROM folder_tree
ORDER BY level, name;
```

### 8.3 获取文档及其协作者

```sql
-- 获取文档详情和协作者列表
SELECT
    d.id,
    d.title,
    d.content,
    d.owner_id,
    u_owner.nickname as owner_name,
    u_owner.email as owner_email,
    COALESCE(
        json_agg(
            json_build_object(
                'user_id', dc.user_id,
                'nickname', u_collab.nickname,
                'email', u_collab.email,
                'permission', dc.permission
            ) ORDER BY dc.created_at
        ) FILTER (WHERE dc.user_id IS NOT NULL),
        '[]'::json
    ) as collaborators
FROM documents d
LEFT JOIN users u_owner ON d.owner_id = u_owner.id
LEFT JOIN document_collaborators dc ON d.id = dc.document_id
LEFT JOIN users u_collab ON dc.user_id = u_collab.id
WHERE d.id = $document_id
GROUP BY d.id, u_owner.id;
```

### 8.4 获取嵌套评论结构

```sql
-- 获取顶层评论及其回复数量
SELECT
    c.id,
    c.content,
    c.position,
    c.is_resolved,
    c.created_at,
    u.nickname,
    u.avatar_url,
    COUNT(rc.id) as reply_count                -- 回复数量
FROM comments c
INNER JOIN users u ON c.user_id = u.id
LEFT JOIN comments rc ON rc.parent_id = c.id  -- 自关联
WHERE c.doc_id = $document_id
  AND c.parent_id IS NULL                      -- 仅顶层评论
GROUP BY c.id, u.id
ORDER BY c.created_at DESC;
```

### 8.5 获取未读通知数量

```sql
-- 统计各类型未读通知
SELECT
    type,
    COUNT(*) as count
FROM notifications
WHERE user_id = $user_id
  AND is_read = FALSE
GROUP BY type;
```

### 8.6 检查用户权限

```sql
-- 检查用户是否拥有指定权限
SELECT EXISTS(
    SELECT 1
    FROM users u
    INNER JOIN role_permissions rp ON u.role_id = rp.role_id
    INNER JOIN permissions p ON rp.permission_id = p.id
    WHERE u.id = $user_id
      AND p.name = $permission_name  -- 例如 'document:update'
) as has_permission;
```

---

## 附录

### A. 数据库配置

```bash
# .env 文件配置
DATABASE_URL=postgres://omm:password@localhost:5432/postgres
DB_SCHEMA=entangle
DB_MAX_CONNECTIONS=10
DB_MIN_CONNECTIONS=1
```

### B. 常用SQL命令

```sql
-- 查看所有表
SELECT table_name FROM information_schema.tables
WHERE table_schema = 'entangle';

-- 查看表结构
\d entangle.users
-- 或
SELECT column_name, data_type, is_nullable
FROM information_schema.columns
WHERE table_schema = 'entangle' AND table_name = 'users';

-- 查看索引
SELECT indexname, tablename, indexdef
FROM pg_indexes
WHERE schemaname = 'entangle';

-- 查看外键
SELECT
    tc.table_name,
    kcu.column_name,
    ccu.table_name AS foreign_table_name,
    ccu.column_name AS foreign_column_name
FROM information_schema.table_constraints AS tc
JOIN information_schema.key_column_usage AS kcu
  ON tc.constraint_name = kcu.constraint_name
JOIN information_schema.constraint_column_usage AS ccu
  ON ccu.constraint_name = tc.constraint_name
WHERE tc.constraint_type = 'FOREIGN KEY'
  AND tc.table_schema = 'entangle';
```

---

**文档版本**: v2.0
**最后更新**: 2025-01-05
**作者**: Entangle 开发团队
