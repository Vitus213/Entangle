# 文件夹系统文档

> 版本: 1.0.0 | 最后更新: 2026-01-01 | 状态: 完成

---

## 目录

- [概述](#概述)
- [数据库设计](#数据库设计)
- [API 参考](#api-参考)
- [使用指南](#使用指南)
- [权限控制](#权限控制)
- [技术实现](#技术实现)
- [性能优化](#性能优化)
- [错误处理](#错误处理)
- [实现状态](#实现状态)

---

## 概述

### 功能目标

实现层级化的文件夹系统，允许用户组织和管理文档：

- 创建/重命名/删除文件夹
- 嵌套文件夹（树形结构）
- 文档移动（在文件夹间移动）
- 文件夹权限继承

### 核心特性

| 特性 | 说明 |
|------|------|
| 树形结构 | 支持无限层级的文件夹嵌套 |
| 权限控制 | 文件夹所有者管理 |
| 高性能 | 使用 CTE 递归查询 |
| 数据安全 | 级联删除保护 |

---

## 数据库设计

### folders 表

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

### documents 表扩展

```sql
ALTER TABLE documents
ADD COLUMN folder_id UUID REFERENCES folders(id) ON DELETE SET NULL;

CREATE INDEX idx_documents_folder ON documents(folder_id);
```

### 数据模型关系

```
User (1) ──── (N) Folder
                    │
                    ├── (1:N) Folder (自引用)
                    └── (1:N) Document
```

---

## API 参考

### 端点列表

| 方法 | 路径 | 描述 | 权限 |
|------|------|------|------|
| POST | `/api/folders` | 创建文件夹 | 登录用户 |
| GET | `/api/folders/tree` | 获取文件夹树 | 登录用户 |
| GET | `/api/folders/:id` | 获取文件夹详情 | 所有者 |
| PUT | `/api/folders/:id` | 更新文件夹 | 所有者 |
| DELETE | `/api/folders/:id` | 删除文件夹 | 所有者 |
| GET | `/api/folders/:id/contents` | 获取文件夹内容 | 所有者 |
| PUT | `/api/documents/:id/move` | 移动文档 | 文档所有者 |

### 创建文件夹

**请求:**
```http
POST /api/folders
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "项目文档",
  "parent_id": null  // null 表示根文件夹
}
```

**响应 (200):**
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "name": "项目文档",
  "parent_id": null,
  "owner": {
    "id": "user-id",
    "nickname": "DemoUser",
    "email": "demo@example.com"
  },
  "created_at": "2026-01-01T00:00:00Z",
  "updated_at": "2026-01-01T00:00:00Z"
}
```

### 获取文件夹树

**请求:**
```http
GET /api/folders/tree
Authorization: Bearer <token>
```

**响应 (200):**
```json
[
  {
    "id": "uuid-1",
    "name": "项目文档",
    "parent_id": null,
    "owner_id": "user-id",
    "created_at": "2026-01-01T00:00:00Z",
    "updated_at": "2026-01-01T00:00:00Z",
    "document_count": 5,
    "children": [
      {
        "id": "uuid-2",
        "name": "设计文档",
        "parent_id": "uuid-1",
        "document_count": 3,
        "children": []
      }
    ]
  }
]
```

### 获取文件夹内容

**请求:**
```http
GET /api/folders/:id/contents
Authorization: Bearer <token>
```

**响应 (200):**
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
      "owner": {...},
      "is_public": false,
      "created_at": "2026-01-01T00:00:00Z",
      "updated_at": "2026-01-01T00:00:00Z"
    }
  ]
}
```

### 更新文件夹

**请求:**
```http
PUT /api/folders/:id
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "Renamed Folder",  // 可选
  "parent_id": "new-parent-id"  // 可选，用于移动文件夹
}
```

### 删除文件夹

**请求:**
```http
DELETE /api/folders/:id
Authorization: Bearer <token>
```

**响应:** `204 No Content`

**注意:** 删除文件夹会级联删除所有子文件夹。文件夹内的文档不会被删除，但会失去文件夹关联。

### 移动文档

**请求:**
```http
PUT /api/documents/:id/move
Authorization: Bearer <token>
Content-Type: application/json

{
  "folder_id": "target-folder-id"  // null 表示移动到根目录
}
```

**响应:** `204 No Content`

---

## 使用指南

### 创建层级文件夹结构

```bash
# 设置 Token
TOKEN="your-jwt-token"

# 1. 创建根文件夹
curl -X POST http://localhost:3000/api/folders \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"Projects","parent_id":null}'

# 2. 创建子文件夹
curl -X POST http://localhost:3000/api/folders \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"Frontend","parent_id":"<root-folder-id>"}'
```

### 移动文档

```bash
# 将文档移动到指定文件夹
curl -X PUT http://localhost:3000/api/documents/<doc-id>/move \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"folder_id":"<folder-id>"}'

# 将文档移回根目录
curl -X PUT http://localhost:3000/api/documents/<doc-id>/move \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"folder_id":null}'
```

### 浏览文件夹

```bash
# 获取文件夹树
curl http://localhost:3000/api/folders/tree \
  -H "Authorization: Bearer $TOKEN"

# 获取特定文件夹的内容
curl http://localhost:3000/api/folders/<folder-id>/contents \
  -H "Authorization: Bearer $TOKEN"
```

### 最佳实践

1. **文件夹命名**
   - 使用清晰描述性的名称
   - 避免使用特殊字符
   - 保持名称简短（建议不超过 50 个字符）

2. **层级深度**
   - 建议不超过 5 层深度
   - 过深的层级会影响查询性能

3. **删除操作**
   - 删除文件夹前确认不再需要
   - 删除是级联的，会同时删除所有子文件夹
   - 文件夹内的文档不会被删除，但会失去关联

---

## 权限控制

| 操作 | 权限要求 |
|------|---------|
| 创建文件夹 | 任何登录用户 |
| 查看文件夹 | 文件夹所有者 |
| 更新文件夹 | 文件夹所有者 |
| 删除文件夹 | 文件夹所有者 |
| 获取文件夹树 | 任何登录用户（仅显示自己的文件夹） |
| 移动文档 | 文档所有者 + 目标文件夹所有者 |

### 权限检查逻辑

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

## 技术实现

### Rust 数据模型

```rust
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
```

### 文件夹树查询 (CTE)

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

### 文件夹路径查询

```sql
WITH RECURSIVE folder_path AS (
    SELECT id, name, parent_id, ARRAY[name::TEXT] as path
    FROM folders
    WHERE id = $1

    UNION ALL

    SELECT f.id, f.name, f.parent_id, f.name::TEXT || fp.path
    FROM folders f
    INNER JOIN folder_path fp ON f.id = fp.parent_id
)
SELECT path FROM folder_path WHERE parent_id IS NULL;
```

### 循环引用检测

```rust
pub async fn can_move_folder(
    pool: &PgPool,
    folder_id: Uuid,
    new_parent_id: Uuid,
) -> Result<bool, sqlx::Error> {
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

### 技术难点解决

**1. 递归 CTE 类型匹配问题**

问题: VARCHAR(255)[] 与 VARCHAR[] 类型不匹配

解决: 将数组元素类型强制转换为 TEXT
```sql
ARRAY[name::TEXT]  -- 非递归部分
f.name::TEXT || fp.path  -- 递归部分
```

**2. sqlx 嵌套结构映射**

使用临时平面结构接收查询结果，然后手动映射到目标结构。

**3. openGauss 兼容性**

`gen_random_uuid()` 函数在 openGauss 中不可用，在 Rust 代码中生成 UUID。

---

## 性能优化

### 索引策略

| 索引 | 用途 |
|------|------|
| `(owner_id, parent_id)` | 加速树查询 |
| `parent_id` | 加速子文件夹查询 |
| `folder_id` on documents | 加速文档查询 |

### 时间复杂度

| 操作 | 复杂度 |
|------|--------|
| 文件夹树查询 | O(n) - n 为文件夹总数 |
| 路径查询 | O(d) - d 为文件夹深度 |
| 文件夹内容 | O(c + d) - c 子文件夹数，d 文档数 |

### 缓存建议

- Redis 缓存文件夹树（TTL: 5分钟）
- 缓存 key: `folder:tree:{user_id}`
- 更新文件夹时清除缓存

---

## 错误处理

### 错误类型

| 错误 | 说明 |
|------|------|
| `FolderNotFound` | 文件夹不存在 |
| `CircularReference` | 移动会造成循环引用 |
| `PermissionDenied` | 权限不足 |
| `InvalidParentFolder` | 父文件夹无效 |

### 错误码

| 错误码 | 说明 |
|--------|-----|
| 401 | 未提供有效的认证令牌 |
| 403 | 无权访问该文件夹 |
| 404 | 文件夹不存在 |
| 500 | 服务器内部错误 |

### 错误响应格式

```json
{
  "error": "错误描述信息"
}
```

---

## 实现状态

### 进度: 100% 完成

| 组件 | 状态 | 文件 |
|------|------|------|
| 数据库迁移 | ✅ | `migrations/20260101024042_create_folders_table.sql` |
| 数据模型 | ✅ | `crates/db/src/models/folder.rs` |
| Repository 层 | ✅ | `crates/db/src/repository/folder.rs` |
| API 路由 | ✅ | `crates/api/src/routes/folders.rs` |
| 测试脚本 | ✅ | `scripts/test_folders.sh` |

### 测试覆盖

- ✅ 创建根文件夹
- ✅ 创建子文件夹（多层级）
- ✅ 获取文件夹详情
- ✅ 更新文件夹名称
- ✅ 获取文件夹树
- ✅ 创建文档
- ✅ 移动文档到文件夹
- ✅ 获取文件夹内容
- ✅ 移动文档出文件夹
- ✅ 删除文件夹
- ✅ 验证文件夹树更新

---

## 后续扩展

- 文件夹共享功能
- 文件夹颜色/图标
- 文件夹模板
- 文件夹统计信息（大小、文档数量等）
- 批量移动文档
- 文件夹名称搜索

---

## 相关文档

- [标签系统文档](./TAG_SYSTEM.md)
- [编译与启动指南](./BUILD_AND_RUN.md)
- [API 参考文档](./API_REFERENCE.md)

---

*最后更新: 2026-01-01*
