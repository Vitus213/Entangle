# Entangle 数据库设计文档

> openGauss/PostgreSQL 数据库表结构和关系说明

---

## 目录

1. [数据库概览](#1-数据库概览)
2. [表结构详解](#2-表结构详解)
3. [ER 图](#3-er-图)
4. [索引设计](#4-索引设计)
5. [迁移管理](#5-迁移管理)

---

## 1. 数据库概览

### 1.1 技术选型

- **数据库**: openGauss 5.1.0 (PostgreSQL 兼容)
- **连接池**: SQLx (异步)
- **迁移工具**: SQLx CLI

### 1.2 数据库连接

```bash
# 环境变量
DATABASE_URL=postgres://omm:password@localhost:5432/postgres
```

### 1.3 表列表

| 表名 | 说明 | 主要字段 |
|------|------|----------|
| users | 用户表 | id, email, password_hash, nickname |
| roles | 角色表 | id, name, description, is_system |
| documents | 文档表 | id, title, content, owner_id, crdt_state |
| folders | 文件夹表 | id, name, parent_id, owner_id |
| tags | 标签表 | id, name, color, owner_id |
| document_tags | 文档-标签关联 | document_id, tag_id |
| document_collaborators | 协作者表 | document_id, user_id, permission |
| comments | 评论表 | id, doc_id, user_id, content, position |
| notifications | 通知表 | id, user_id, type, title, content |
| tasks | 任务表 | id, doc_id, title, assignee_id, status |
| document_versions | 版本历史 | id, doc_id, version_number, content |

---

## 2. 表结构详解

### 2.1 users (用户表)

存储系统用户信息。

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    nickname VARCHAR(100) NOT NULL,
    avatar_url VARCHAR(500),
    phone VARCHAR(20),
    role_id UUID REFERENCES roles(id),
    email_verified BOOLEAN DEFAULT FALSE,
    status VARCHAR(20) DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 索引
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_role ON users(role_id);
```

**字段说明**：

| 字段 | 类型 | 说明 |
|------|------|------|
| id | UUID | 主键 |
| email | VARCHAR(255) | 邮箱，唯一 |
| password_hash | VARCHAR(255) | Argon2 哈希密码 |
| nickname | VARCHAR(100) | 显示昵称 |
| avatar_url | VARCHAR(500) | 头像 URL |
| phone | VARCHAR(20) | 手机号（可选） |
| role_id | UUID | 外键 → roles |
| email_verified | BOOLEAN | 邮箱是否验证 |
| status | VARCHAR(20) | 状态: active/banned |

---

### 2.2 roles (角色表)

存储用户角色定义。

```sql
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) NOT NULL UNIQUE,
    description VARCHAR(255),
    is_system BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 默认数据
INSERT INTO roles (id, name, description, is_system) VALUES
    ('00000000-0000-0000-0000-000000000001', 'user', '普通用户', TRUE),
    ('00000000-0000-0000-0000-000000000002', 'admin', '管理员', TRUE);
```

**字段说明**：

| 字段 | 类型 | 说明 |
|------|------|------|
| id | UUID | 主键 |
| name | VARCHAR(50) | 角色名称: user/admin |
| description | VARCHAR(255) | 角色描述 |
| is_system | BOOLEAN | 是否系统角色 |

---

### 2.3 documents (文档表)

存储协作文档。

```sql
CREATE TABLE documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(500) NOT NULL,
    content TEXT DEFAULT '',
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    is_public BOOLEAN DEFAULT FALSE,
    folder_id UUID REFERENCES folders(id) ON DELETE SET NULL,
    crdt_state BYTEA,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 索引
CREATE INDEX idx_documents_owner ON documents(owner_id);
CREATE INDEX idx_documents_folder ON documents(folder_id);
CREATE INDEX idx_documents_updated_at ON documents(updated_at DESC);
```

**字段说明**：

| 字段 | 类型 | 说明 |
|------|------|------|
| id | UUID | 主键 |
| title | VARCHAR(500) | 文档标题 |
| content | TEXT | 文本内容（备份） |
| owner_id | UUID | 所有者 ID → users |
| is_public | BOOLEAN | 是否公开 |
| folder_id | UUID | 所属文件夹 → folders |
| crdt_state | BYTEA | CRDT 二进制状态 |

**为什么有 content 和 crdt_state 两个字段？**

- `content`: 纯文本备份，便于搜索和显示
- `crdt_state`: CRDT 完整状态，用于协作同步

---

### 2.4 folders (文件夹表)

存储文档文件夹结构。

```sql
CREATE TABLE folders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    parent_id UUID REFERENCES folders(id) ON DELETE CASCADE,
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 索引
CREATE INDEX idx_folders_parent ON folders(parent_id);
CREATE INDEX idx_folders_owner ON folders(owner_id);
```

**字段说明**：

| 字段 | 类型 | 说明 |
|------|------|------|
| id | UUID | 主键 |
| name | VARCHAR(255) | 文件夹名称 |
| parent_id | UUID | 父文件夹 → folders (自引用) |
| owner_id | UUID | 所有者 → users |

**树形结构**：

```
根文件夹 (parent_id = NULL)
├── 工作 (id=1, parent_id=NULL)
│   ├── 项目A (id=2, parent_id=1)
│   └── 项目B (id=3, parent_id=1)
└── 个人 (id=4, parent_id=NULL)
    └── 日记 (id=5, parent_id=4)
```

---

### 2.5 tags (标签表)

存储文档标签。

```sql
CREATE TABLE tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) NOT NULL,
    color VARCHAR(7) NOT NULL DEFAULT '#3B82F6',
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 唯一约束：每个用户标签名唯一
CREATE UNIQUE INDEX idx_tags_user_name ON tags(owner_id, name);
```

**字段说明**：

| 字段 | 类型 | 说明 |
|------|------|------|
| id | UUID | 主键 |
| name | VARCHAR(50) | 标签名称 |
| color | VARCHAR(7) | 颜色，如 #FF0000 |
| owner_id | UUID | 所有者 → users |

---

### 2.6 document_tags (文档标签关联表)

多对多关系：一个文档可以有多个标签，一个标签可用于多个文档。

```sql
CREATE TABLE document_tags (
    document_id UUID REFERENCES documents(id) ON DELETE CASCADE,
    tag_id UUID REFERENCES tags(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (document_id, tag_id)
);
```

---

### 2.7 document_collaborators (协作者表)

存储文档协作者和权限。

```sql
CREATE TABLE document_collaborators (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permission VARCHAR(50) NOT NULL,  -- 'read', 'write', 'admin'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(document_id, user_id)
);

-- 索引
CREATE INDEX idx_collab_document ON document_collaborators(document_id);
CREATE INDEX idx_collab_user ON document_collaborators(user_id);
```

**字段说明**：

| 字段 | 类型 | 说明 |
|------|------|------|
| id | UUID | 主键 |
| document_id | UUID | 文档 ID → documents |
| user_id | UUID | 用户 ID → users |
| permission | VARCHAR(50) | read/write/admin |

**权限层级**：

```
read (只读) < write (编辑) < admin (管理)
```

---

### 2.8 comments (评论表)

存储文档评论。

```sql
CREATE TABLE comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    doc_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES comments(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    position JSONB,
    is_resolved BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 索引
CREATE INDEX idx_comments_doc ON comments(doc_id);
CREATE INDEX idx_comments_parent ON comments(parent_id);
```

**字段说明**：

| 字段 | 类型 | 说明 |
|------|------|------|
| id | UUID | 主键 |
| doc_id | UUID | 文档 ID → documents |
| user_id | UUID | 评论者 → users |
| parent_id | UUID | 父评论 → comments (自引用，用于回复) |
| content | TEXT | 评论内容 |
| position | JSONB | 位置: `{"start": 0, "end": 10}` |
| is_resolved | BOOLEAN | 是否已解决 |

**嵌套评论结构**：

```
主评论 (parent_id = NULL)
├── 回复1 (parent_id = 主评论ID)
└── 回复2 (parent_id = 主评论ID)
```

---

### 2.9 notifications (通知表)

存储用户通知。

```sql
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    notification_type VARCHAR(50) NOT NULL,  -- 'comment', 'task', 'share', 'system'
    title VARCHAR(255) NOT NULL,
    content TEXT,
    resource_type VARCHAR(50),
    resource_id UUID,
    sender_id UUID REFERENCES users(id) ON DELETE SET NULL,
    is_read BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 索引
CREATE INDEX idx_notifications_user ON notifications(user_id, created_at DESC);
CREATE INDEX idx_notifications_unread ON notifications(user_id) WHERE is_read = FALSE;
```

**字段说明**：

| 字段 | 类型 | 说明 |
|------|------|------|
| id | UUID | 主键 |
| user_id | UUID | 接收者 → users |
| notification_type | VARCHAR(50) | comment/task/share/system |
| title | VARCHAR(255) | 通知标题 |
| content | TEXT | 通知内容 |
| resource_type | VARCHAR(50) | 资源类型: document/comment |
| resource_id | UUID | 资源 ID |
| sender_id | UUID | 发送者 → users |
| is_read | BOOLEAN | 是否已读 |

**通知类型**：

- `comment`: 新评论
- `task`: 任务分配/更新
- `share`: 协作邀请
- `system`: 系统通知

---

### 2.10 tasks (任务表)

存储任务信息。

```sql
CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    doc_id UUID REFERENCES documents(id) ON DELETE SET NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    assignee_id UUID REFERENCES users(id) ON DELETE SET NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    priority VARCHAR(50) NOT NULL DEFAULT 'medium',
    due_date TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 索引
CREATE INDEX idx_tasks_assignee ON tasks(assignee_id);
CREATE INDEX idx_tasks_created_by ON tasks(created_by);
CREATE INDEX idx_tasks_doc ON tasks(doc_id);
CREATE INDEX idx_tasks_status ON tasks(status);
```

**字段说明**：

| 字段 | 类型 | 说明 |
|------|------|------|
| id | UUID | 主键 |
| doc_id | UUID | 关联文档 → documents |
| title | VARCHAR(255) | 任务标题 |
| description | TEXT | 任务描述 |
| created_by | UUID | 创建者 → users |
| assignee_id | UUID | 被分配者 → users |
| status | VARCHAR(50) | pending/in_progress/completed/cancelled |
| priority | VARCHAR(50) | low/medium/high/urgent |
| due_date | TIMESTAMPTZ | 截止日期 |

---

### 2.11 document_versions (版本历史表)

存储文档版本快照。

```sql
CREATE TABLE document_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    doc_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL,
    title VARCHAR(500) NOT NULL,
    content TEXT,
    crdt_state BYTEA,
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    description VARCHAR(500),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_version UNIQUE(doc_id, version_number)
);

-- 索引
CREATE INDEX idx_versions_doc ON document_versions(doc_id, version_number);
```

**字段说明**：

| 字段 | 类型 | 说明 |
|------|------|------|
| id | UUID | 主键 |
| doc_id | UUID | 文档 ID → documents |
| version_number | INTEGER | 版本号（递增） |
| title | VARCHAR(500) | 版本时的标题 |
| content | TEXT | 版本时的内容 |
| crdt_state | BYTEA | CRDT 状态快照 |
| created_by | UUID | 创建者 → users |
| description | VARCHAR(500) | 版本描述 |

---

## 3. ER 图

```
┌─────────────┐
│    users    │
├─────────────┤
│ id (PK)     │──┐
│ email       │  │
│ nickname    │  │
│ role_id     │──┼─────────────────┐
└─────────────┘  │                 │
                  │                 │
                  ▼                 │
         ┌─────────────────┐        │
         │     roles       │        │
         ├─────────────────┤        │
         │ id (PK)         │        │
         │ name            │        │
         └─────────────────┘        │
                                    │
┌─────────────┐              ┌──────┴──────────────────────┐
│  documents  │              │                             │
├─────────────┤              │                             │
│ id (PK)     │◄─────────────┤                             │
│ title       │              │                             │
│ content     │              ▼                             ▼
│ owner_id    │──┐     ┌───────────────┐          ┌──────────────┐
│ folder_id   │──┼────►│   folders     │          │   tags       │
│ crdt_state  │  │     ├───────────────┤          ├──────────────┤
└─────────────┘  │     │ id (PK)       │          │ id (PK)      │
                  │     │ parent_id     │          │ name         │
                  │     │ owner_id      │          └──────────────┘
                  │     └───────────────│                  │
                  │                       │                  │
                  │                       │                  │
┌─────────────────────────────────────────┼──────────────────┼─────────┐
│                                         │                  │         │
│         ┌───────────────┐              │                  │         │
│         │  comments     │              │                  │         │
│         ├───────────────┤              │                  │         │
│         │ id (PK)       │              │                  │         │
│         │ doc_id (FK)   │──────────────┘                  │         │
│         │ user_id (FK)  │                                │         │
│         │ parent_id     │◄────(self-ref)                  │         │
│         └───────────────┘                                │         │
│                                                            │         │
│         ┌───────────────┐        ┌───────────────┐        │         │
│         │  notifications│        │     tasks     │        │         │
│         ├───────────────┤        ├───────────────┤        │         │
│         │ id (PK)       │        │ id (PK)       │        │         │
│         │ user_id (FK)  │        │ assignee_id   │◄───────┴─────────┤
│         │ sender_id     │        │ created_by    │◄─────────────────┤
│         └───────────────┘        │ doc_id        │◄─────────────────┤
│                                  └───────────────┘                  │
│                                                                     │
│         ┌─────────────────────────────────────────────────────┐    │
│         │          document_collaborators                      │    │
│         ├─────────────────────────────────────────────────────┤    │
│         │ document_id (FK) ──────────────────────────────────┼────┘
│         │ user_id (FK) ───────────────────────────────────────┼────┐
│         └─────────────────────────────────────────────────────┘    │
│                                                                    │
│         ┌─────────────────────────────────────────────────────┐    │
│         │             document_versions                        │    │
│         ├─────────────────────────────────────────────────────┤    │
│         │ doc_id (FK) ────────────────────────────────────────┼────┘
│         │ created_by (FK)                                     │
│         └─────────────────────────────────────────────────────┘
│
└───────────────────────────────────────────────────────────────────┘
```

---

## 4. 索引设计

### 4.1 性能索引

| 索引 | 表 | 字段 | 用途 |
|------|-----|------|------|
| idx_users_email | users | email | 登录查询 |
| idx_documents_owner | documents | owner_id | 查询用户的文档 |
| idx_documents_updated_at | documents | updated_at DESC | 按更新时间排序 |
| idx_collab_document | document_collaborators | document_id | 查询协作者 |
| idx_collab_user | document_collaborators | user_id | 查询用户可访问的文档 |
| idx_comments_doc | comments | doc_id | 查询文档评论 |
| idx_notifications_user | notifications | user_id, created_at | 通知列表 |
| idx_tasks_assignee | tasks | assignee_id | 查询分配的任务 |

### 4.2 唯一约束

| 约束 | 表 | 字段 |
|------|-----|------|
| users_email_key | users | email |
| tags_user_name_key | tags | owner_id, name |
| document_collaborators_key | document_collaborators | document_id, user_id |
| unique_version | document_versions | doc_id, version_number |

---

## 5. 迁移管理

### 5.1 创建迁移

```bash
sqlx migrate add migrate_name
```

### 5.2 运行迁移

```bash
# 开发环境
sqlx migrate run

# 生产环境
sqlx migrate run --database-url $DATABASE_URL
```

### 5.3 回滚迁移

```bash
sqlx migrate revert
```

### 5.4 迁移文件位置

```
migrations/
├── 001_initial_setup.sql
├── 002_add_folders.sql
├── 003_add_tags.sql
├── 004_add_collaborators.sql
├── 005_add_comments.sql
├── 006_add_notifications.sql
├── 007_add_tasks.sql
└── 008_add_versions.sql
```

---

## 6. 数据完整性

### 6.1 外键约束

- `documents.owner_id → users.id` (CASCADE)
- `folders.owner_id → users.id` (CASCADE)
- `comments.user_id → users.id` (CASCADE)
- `document_collaborators.user_id → users.id` (CASCADE)
- `document_collaborators.document_id → documents.id` (CASCADE)

### 6.2 级联操作

- **CASCADE**: 删除用户时，自动删除其创建的内容
- **SET NULL**: 删除父文件夹时，子文件夹的 parent_id 设为 NULL

---

## 7. 查询示例

### 7.1 获取用户可访问的文档

```sql
-- 包括拥有的文档和有协作权限的文档
SELECT DISTINCT d.* FROM documents d
WHERE d.owner_id = $user_id
   OR d.id IN (
       SELECT document_id FROM document_collaborators
       WHERE user_id = $user_id
   )
   OR d.is_public = TRUE
ORDER BY d.updated_at DESC;
```

### 7.2 获取文档树形结构

```sql
-- 递归查询 (PostgreSQL)
WITH RECURSIVE folder_tree AS (
    -- 根文件夹
    SELECT id, name, parent_id, owner_id, 1 as level
    FROM folders
    WHERE parent_id IS NULL AND owner_id = $user_id

    UNION ALL

    -- 子文件夹
    SELECT f.id, f.name, f.parent_id, f.owner_id, ft.level + 1
    FROM folders f
    JOIN folder_tree ft ON f.parent_id = ft.id
)
SELECT * FROM folder_tree ORDER BY level, name;
```

### 7.3 获取嵌套评论

```sql
-- 获取顶层评论
SELECT c.*, u.nickname, u.avatar_url,
       COUNT(rc.id) as reply_count
FROM comments c
JOIN users u ON c.user_id = u.id
LEFT JOIN comments rc ON rc.parent_id = c.id
WHERE c.doc_id = $doc_id AND c.parent_id IS NULL
GROUP BY c.id, u.id
ORDER BY c.created_at DESC;
```

---

文档版本: v1.0
最后更新: 2024-01-05
