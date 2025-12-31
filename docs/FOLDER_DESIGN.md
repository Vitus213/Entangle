# 文件夹系统架构设计

> 版本: 1.0
> 作者: Entangle Team
> 日期: 2024-12-31

---

## 1. 概述

### 1.1 功能目标

实现层级化的文件夹系统，允许用户组织和管理文档，支持：
- 创建/重命名/删除文件夹
- 嵌套文件夹（树形结构）
- 文档移动（在文件夹间移动）
- 文件夹权限继承

### 1.2 核心特性

- 📁 **树形结构**: 支持无限层级的文件夹嵌套
- 🔒 **权限控制**: 文件夹所有者管理
- 🚀 **高性能**: 使用 CTE 递归查询
- 🛡️ **数据安全**: 级联删除保护

---

## 2. 数据库设计

### 2.1 表结构

```sql
CREATE TABLE folders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    parent_id UUID REFERENCES folders(id) ON DELETE CASCADE,
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT folders_name_not_empty CHECK (length(trim(name)) > 0)
);

-- 索引
CREATE INDEX idx_folders_parent ON folders(parent_id);
CREATE INDEX idx_folders_owner ON folders(owner_id);
CREATE INDEX idx_folders_owner_parent ON folders(owner_id, parent_id);
```

### 2.2 documents 表扩展

```sql
ALTER TABLE documents
ADD COLUMN folder_id UUID REFERENCES folders(id) ON DELETE SET NULL;

CREATE INDEX idx_documents_folder ON documents(folder_id);
```

### 2.3 数据模型

```
User (1) ──── (N) Folder
                    │
                    ├── (1:N) Folder (自引用)
                    └── (1:N) Document
```

---

## 3. API 设计

### 3.1 端点列表

| 方法 | 路径 | 描述 | 权限 |
|------|------|------|------|
| POST | `/api/folders` | 创建文件夹 | 登录用户 |
| GET | `/api/folders` | 获取文件夹树 | 登录用户 |
| GET | `/api/folders/:id` | 获取文件夹详情 | 所有者 |
| PUT | `/api/folders/:id` | 更新文件夹 | 所有者 |
| DELETE | `/api/folders/:id` | 删除文件夹 | 所有者 |
| GET | `/api/folders/:id/contents` | 获取文件夹内容 | 所有者 |
| POST | `/api/documents/:id/move` | 移动文档 | 文档所有者 |

### 3.2 请求/响应示例

#### 创建文件夹

**请求**:
```json
POST /api/folders
{
  "name": "项目文档",
  "parent_id": null  // null 表示根文件夹
}
```

**响应**:
```json
{
  "id": "uuid",
  "name": "项目文档",
  "parent_id": null,
  "owner_id": "uuid",
  "created_at": "2024-12-31T10:00:00Z",
  "updated_at": "2024-12-31T10:00:00Z"
}
```

#### 获取文件夹树

**请求**:
```
GET /api/folders
```

**响应**:
```json
[
  {
    "id": "uuid-1",
    "name": "项目文档",
    "parent_id": null,
    "children": [
      {
        "id": "uuid-2",
        "name": "设计文档",
        "parent_id": "uuid-1",
        "children": []
      }
    ],
    "document_count": 5
  }
]
```

#### 获取文件夹内容

**请求**:
```
GET /api/folders/:id/contents
```

**响应**:
```json
{
  "folder": {
    "id": "uuid",
    "name": "项目文档",
    "path": ["根目录", "项目文档"]
  },
  "subfolders": [
    {
      "id": "uuid",
      "name": "设计文档",
      "document_count": 3
    }
  ],
  "documents": [
    {
      "id": "uuid",
      "title": "需求文档",
      "created_at": "2024-12-31T10:00:00Z"
    }
  ]
}
```

---

## 4. 核心功能实现

### 4.1 文件夹树查询

使用 PostgreSQL CTE (Common Table Expression) 递归查询：

```sql
WITH RECURSIVE folder_tree AS (
    -- 根节点
    SELECT
        id, name, parent_id, owner_id,
        0 as depth,
        ARRAY[id] as path
    FROM folders
    WHERE parent_id IS NULL AND owner_id = $1

    UNION ALL

    -- 递归查询子节点
    SELECT
        f.id, f.name, f.parent_id, f.owner_id,
        ft.depth + 1,
        ft.path || f.id
    FROM folders f
    INNER JOIN folder_tree ft ON f.parent_id = ft.id
)
SELECT * FROM folder_tree ORDER BY path;
```

### 4.2 文件夹路径获取

```sql
WITH RECURSIVE folder_path AS (
    -- 起始节点
    SELECT id, name, parent_id, ARRAY[name] as path
    FROM folders
    WHERE id = $1

    UNION ALL

    -- 向上递归到根节点
    SELECT f.id, f.name, f.parent_id, f.name || fp.path
    FROM folders f
    INNER JOIN folder_path fp ON f.id = fp.parent_id
)
SELECT path FROM folder_path WHERE parent_id IS NULL;
```

### 4.3 循环引用检测

在移动文件夹时，需要检测是否会造成循环引用：

```rust
pub async fn can_move_folder(
    pool: &PgPool,
    folder_id: Uuid,
    new_parent_id: Uuid,
) -> Result<bool, sqlx::Error> {
    // 检查 new_parent_id 是否是 folder_id 的后代
    let result = sqlx::query_scalar::<_, bool>(
        r#"
        WITH RECURSIVE descendants AS (
            SELECT id FROM folders WHERE id = $1
            UNION ALL
            SELECT f.id
            FROM folders f
            INNER JOIN descendants d ON f.parent_id = d.id
        )
        SELECT EXISTS(SELECT 1 FROM descendants WHERE id = $2)
        "#
    )
    .bind(folder_id)
    .bind(new_parent_id)
    .fetch_one(pool)
    .await?;

    Ok(!result)
}
```

---

## 5. 数据模型 (Rust)

### 5.1 核心结构

```rust
// models/folder.rs
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Folder {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFolder {
    pub name: String,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateFolder {
    pub name: Option<String>,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderTree {
    #[serde(flatten)]
    pub folder: Folder,
    pub children: Vec<FolderTree>,
    pub document_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FolderContents {
    pub folder: FolderInfo,
    pub subfolders: Vec<FolderSummary>,
    pub documents: Vec<DocumentListItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FolderInfo {
    pub id: Uuid,
    pub name: String,
    pub path: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FolderSummary {
    pub id: Uuid,
    pub name: String,
    pub document_count: i64,
}
```

---

## 6. Repository 层

```rust
// repository/folder.rs
pub struct FolderRepository;

impl FolderRepository {
    /// 创建文件夹
    pub async fn create(
        pool: &PgPool,
        folder: &CreateFolder,
        owner_id: Uuid,
    ) -> Result<Folder, sqlx::Error>;

    /// 获取文件夹树
    pub async fn get_tree(
        pool: &PgPool,
        owner_id: Uuid,
    ) -> Result<Vec<FolderTree>, sqlx::Error>;

    /// 获取文件夹详情
    pub async fn find_by_id(
        pool: &PgPool,
        folder_id: Uuid,
    ) -> Result<Option<Folder>, sqlx::Error>;

    /// 更新文件夹
    pub async fn update(
        pool: &PgPool,
        folder_id: Uuid,
        update: &UpdateFolder,
    ) -> Result<Folder, sqlx::Error>;

    /// 删除文件夹
    pub async fn delete(
        pool: &PgPool,
        folder_id: Uuid,
    ) -> Result<(), sqlx::Error>;

    /// 获取文件夹内容
    pub async fn get_contents(
        pool: &PgPool,
        folder_id: Uuid,
    ) -> Result<FolderContents, sqlx::Error>;

    /// 获取文件夹路径
    pub async fn get_path(
        pool: &PgPool,
        folder_id: Uuid,
    ) -> Result<Vec<String>, sqlx::Error>;
}
```

---

## 7. 权限控制

### 7.1 权限规则

- **创建**: 任何登录用户都可以创建文件夹
- **读取**: 只有所有者可以访问自己的文件夹
- **更新**: 只有所有者可以重命名或移动文件夹
- **删除**: 只有所有者可以删除文件夹（级联删除所有子文件夹和文档）

### 7.2 检查逻辑

```rust
pub async fn check_folder_owner(
    pool: &PgPool,
    folder_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query_scalar::<_, bool>(
        "SELECT owner_id = $1 FROM folders WHERE id = $2"
    )
    .bind(user_id)
    .bind(folder_id)
    .fetch_optional(pool)
    .await?;

    Ok(result.unwrap_or(false))
}
```

---

## 8. 错误处理

### 8.1 错误类型

- `FolderNotFound` - 文件夹不存在
- `CircularReference` - 移动会造成循环引用
- `FolderNotEmpty` - 删除非空文件夹（可选，根据业务需求）
- `PermissionDenied` - 权限不足
- `InvalidParentFolder` - 父文件夹无效

### 8.2 示例

```rust
#[derive(Debug, thiserror::Error)]
pub enum FolderError {
    #[error("Folder not found")]
    NotFound,

    #[error("Cannot move folder: would create circular reference")]
    CircularReference,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
```

---

## 9. 性能优化

### 9.1 索引策略

- `(owner_id, parent_id)` 复合索引 - 加速树查询
- `parent_id` 单独索引 - 加速子文件夹查询
- `folder_id` on documents - 加速文档查询

### 9.2 缓存策略 (可选)

- Redis 缓存文件夹树（TTL: 5分钟）
- 缓存 key: `folder:tree:{user_id}`
- 更新文件夹时清除缓存

---

## 10. 测试计划

### 10.1 单元测试

- ✅ 创建根文件夹
- ✅ 创建子文件夹
- ✅ 重命名文件夹
- ✅ 移动文件夹
- ✅ 删除文件夹（级联）
- ✅ 获取文件夹树
- ✅ 循环引用检测

### 10.2 集成测试

- ✅ API 端点测试
- ✅ 权限验证
- ✅ 并发创建测试

### 10.3 测试脚本

`scripts/test_folders.sh`:
```bash
#!/usr/bin/env bash
# 测试文件夹系统的所有功能
```

---

## 11. 迁移脚本

`migrations/YYYYMMDDHHMMSS_create_folders_table.sql`:
```sql
-- 创建文件夹表
CREATE TABLE folders (...);

-- 修改文档表
ALTER TABLE documents ADD COLUMN folder_id UUID;

-- 创建索引
CREATE INDEX ...;
```

---

## 12. API 文档示例

将在实现后自动生成 OpenAPI 3.0 规范。

---

## 13. 未来扩展

- 文件夹共享功能
- 文件夹颜色/图标
- 文件夹模板
- 文件夹统计信息（大小、文档数量等）

---

## 附录

### 参考资料

- PostgreSQL CTE 文档: https://www.postgresql.org/docs/current/queries-with.html
- 树形结构最佳实践: https://use-the-index-luke.com/sql/trees-and-hierarchies

### 相关文档

- `PROJECT_PLAN.md` - 项目总体规划
- `PROGRESS.md` - 开发进度
- `API.md` - API 完整文档（待生成）
