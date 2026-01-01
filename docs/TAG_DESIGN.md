# 标签系统设计文档

## 1. 概述

标签系统允许用户为文档添加自定义标签，实现灵活的文档分类和快速检索。

### 核心特性

- ✅ 创建、读取、更新、删除标签
- ✅ 为文档添加/移除标签
- ✅ 按标签筛选文档
- ✅ 标签使用统计
- ✅ 标签颜色自定义
- ✅ 用户级别的标签隔离

---

## 2. 数据库设计

### 2.1 tags 表

存储标签基本信息。

```sql
CREATE TABLE tags (
    id UUID PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    color VARCHAR(7) NOT NULL DEFAULT '#3B82F6',  -- 十六进制颜色
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT tags_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT tags_color_format CHECK (color ~ '^#[0-9A-Fa-f]{6}$'),
    CONSTRAINT tags_name_owner_unique UNIQUE (name, owner_id)
);

CREATE INDEX idx_tags_owner ON tags(owner_id);
CREATE INDEX idx_tags_name ON tags(name);
```

**字段说明：**
- `id`: 标签唯一标识
- `name`: 标签名称（最长 50 字符）
- `color`: 标签颜色（#RRGGBB 格式）
- `owner_id`: 标签所有者
- `created_at/updated_at`: 时间戳

**约束：**
- 标签名称不能为空
- 颜色必须是有效的十六进制格式
- 同一用户的标签名称不能重复

### 2.2 document_tags 表

存储文档与标签的多对多关系。

```sql
CREATE TABLE document_tags (
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    tag_id UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (document_id, tag_id)
);

CREATE INDEX idx_document_tags_document ON document_tags(document_id);
CREATE INDEX idx_document_tags_tag ON document_tags(tag_id);
```

**字段说明：**
- `document_id`: 文档 ID
- `tag_id`: 标签 ID
- `created_at`: 关联创建时间

**特性：**
- 级联删除：删除文档或标签时自动清理关联
- 复合主键：防止重复关联

---

## 3. API 设计

### 3.1 标签 CRUD

#### 创建标签

```http
POST /api/tags
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "前端开发",
  "color": "#3B82F6"  // 可选，默认蓝色
}
```

**响应 (201):**
```json
{
  "id": "tag-uuid",
  "name": "前端开发",
  "color": "#3B82F6",
  "owner_id": "user-uuid",
  "document_count": 0,
  "created_at": "2026-01-01T00:00:00Z",
  "updated_at": "2026-01-01T00:00:00Z"
}
```

#### 获取我的所有标签

```http
GET /api/tags
Authorization: Bearer <token>
```

**响应 (200):**
```json
[
  {
    "id": "tag-uuid",
    "name": "前端开发",
    "color": "#3B82F6",
    "owner_id": "user-uuid",
    "document_count": 5,
    "created_at": "2026-01-01T00:00:00Z",
    "updated_at": "2026-01-01T00:00:00Z"
  }
]
```

#### 更新标签

```http
PUT /api/tags/:id
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "前端",       // 可选
  "color": "#10B981"    // 可选
}
```

#### 删除标签

```http
DELETE /api/tags/:id
Authorization: Bearer <token>
```

**响应 (204):** 无内容

**注意：** 删除标签会自动移除所有文档的该标签关联。

---

### 3.2 文档标签管理

#### 为文档添加标签

```http
POST /api/documents/:id/tags
Authorization: Bearer <token>
Content-Type: application/json

{
  "tag_id": "tag-uuid"
}
```

**响应 (201):** 无内容

#### 从文档移除标签

```http
DELETE /api/documents/:id/tags/:tag_id
Authorization: Bearer <token>
```

**响应 (204):** 无内容

#### 获取文档的所有标签

```http
GET /api/documents/:id/tags
Authorization: Bearer <token>
```

**响应 (200):**
```json
[
  {
    "id": "tag-uuid",
    "name": "前端开发",
    "color": "#3B82F6"
  }
]
```

#### 批量设置文档标签

```http
PUT /api/documents/:id/tags
Authorization: Bearer <token>
Content-Type: application/json

{
  "tag_ids": ["tag-uuid-1", "tag-uuid-2"]
}
```

**响应 (200):** 更新后的标签列表

---

### 3.3 按标签筛选

#### 获取带标签的文档列表

```http
GET /api/documents/by-tags?tag_ids=uuid1,uuid2&match=all
Authorization: Bearer <token>
```

**查询参数：**
- `tag_ids`: 逗号分隔的标签 ID 列表
- `match`: 匹配模式
  - `all`: 文档必须包含所有指定标签（AND）
  - `any`: 文档包含任一标签即可（OR）

**响应 (200):**
```json
[
  {
    "id": "doc-uuid",
    "title": "文档标题",
    "owner": {...},
    "tags": [
      {"id": "tag-uuid", "name": "前端", "color": "#3B82F6"}
    ],
    "is_public": false,
    "created_at": "2026-01-01T00:00:00Z",
    "updated_at": "2026-01-01T00:00:00Z"
  }
]
```

---

## 4. Rust 数据模型

### 4.1 Tag 模型

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTag {
    pub name: String,
    #[serde(default = "default_color")]
    pub color: String,
}

fn default_color() -> String {
    "#3B82F6".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTag {
    pub name: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagWithCount {
    #[serde(flatten)]
    pub tag: Tag,
    pub document_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TagSummary {
    pub id: Uuid,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddTagToDocument {
    pub tag_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetDocumentTags {
    pub tag_ids: Vec<Uuid>,
}
```

### 4.2 文档标签关联

```rust
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DocumentTag {
    pub document_id: Uuid,
    pub tag_id: Uuid,
    pub created_at: DateTime<Utc>,
}
```

---

## 5. Repository 层

### 5.1 TagRepository

```rust
pub struct TagRepository;

impl TagRepository {
    // 创建标签
    pub async fn create(
        pool: &PgPool,
        tag_data: &CreateTag,
        owner_id: Uuid,
    ) -> Result<Tag, sqlx::Error>;

    // 查找标签
    pub async fn find_by_id(
        pool: &PgPool,
        tag_id: Uuid,
    ) -> Result<Option<Tag>, sqlx::Error>;

    // 列出用户的所有标签（带文档计数）
    pub async fn list_by_owner(
        pool: &PgPool,
        owner_id: Uuid,
    ) -> Result<Vec<TagWithCount>, sqlx::Error>;

    // 更新标签
    pub async fn update(
        pool: &PgPool,
        tag_id: Uuid,
        update_data: &UpdateTag,
    ) -> Result<Tag, sqlx::Error>;

    // 删除标签
    pub async fn delete(
        pool: &PgPool,
        tag_id: Uuid,
    ) -> Result<(), sqlx::Error>;

    // 检查标签所有权
    pub async fn is_owner(
        pool: &PgPool,
        tag_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, sqlx::Error>;

    // 为文档添加标签
    pub async fn add_to_document(
        pool: &PgPool,
        document_id: Uuid,
        tag_id: Uuid,
    ) -> Result<(), sqlx::Error>;

    // 从文档移除标签
    pub async fn remove_from_document(
        pool: &PgPool,
        document_id: Uuid,
        tag_id: Uuid,
    ) -> Result<(), sqlx::Error>;

    // 获取文档的所有标签
    pub async fn get_document_tags(
        pool: &PgPool,
        document_id: Uuid,
    ) -> Result<Vec<TagSummary>, sqlx::Error>;

    // 批量设置文档标签
    pub async fn set_document_tags(
        pool: &PgPool,
        document_id: Uuid,
        tag_ids: &[Uuid],
    ) -> Result<(), sqlx::Error>;

    // 按标签筛选文档
    pub async fn get_documents_by_tags(
        pool: &PgPool,
        user_id: Uuid,
        tag_ids: &[Uuid],
        match_all: bool,
    ) -> Result<Vec<DocumentWithTags>, sqlx::Error>;
}
```

---

## 6. 权限控制

### 6.1 标签权限

| 操作 | 权限要求 |
|------|---------|
| 创建标签 | 任何已登录用户 |
| 查看标签 | 标签所有者 |
| 更新标签 | 标签所有者 |
| 删除标签 | 标签所有者 |
| 为文档添加标签 | 文档所有者 或 协作者(write/admin) |
| 从文档移除标签 | 文档所有者 或 协作者(write/admin) |

### 6.2 权限验证逻辑

```rust
// 检查是否可以为文档添加标签
async fn can_tag_document(
    pool: &PgPool,
    user_id: Uuid,
    document_id: Uuid,
    tag_id: Uuid,
) -> Result<bool, AppError> {
    // 1. 检查标签所有权
    let is_tag_owner = TagRepository::is_owner(pool, tag_id, user_id).await?;
    if !is_tag_owner {
        return Ok(false);
    }

    // 2. 检查文档写权限
    let can_write = DocumentPermissionService::can_write(pool, user_id, document_id).await?;

    Ok(can_write)
}
```

---

## 7. 使用场景

### 7.1 基本使用流程

```
用户登录
  ↓
创建标签（"前端"、"后端"、"文档"）
  ↓
创建文档
  ↓
为文档添加标签
  ↓
通过标签筛选查看文档
```

### 7.2 高级场景

**场景 1: 多标签筛选**
```
需求：查找同时包含"前端"和"React"标签的文档
API: GET /api/documents/by-tags?tag_ids=uuid1,uuid2&match=all
```

**场景 2: 标签统计**
```
需求：显示每个标签下有多少文档
API: GET /api/tags
响应包含 document_count 字段
```

**场景 3: 批量标签管理**
```
需求：一次性设置文档的所有标签
API: PUT /api/documents/:id/tags
Body: {"tag_ids": ["uuid1", "uuid2", "uuid3"]}
```

---

## 8. 性能优化

### 8.1 索引策略

```sql
-- 已创建的索引
CREATE INDEX idx_tags_owner ON tags(owner_id);           -- 查询用户标签
CREATE INDEX idx_tags_name ON tags(name);                -- 标签名称搜索
CREATE INDEX idx_document_tags_document ON document_tags(document_id);  -- 查询文档标签
CREATE INDEX idx_document_tags_tag ON document_tags(tag_id);            -- 反向查询
```

### 8.2 查询优化

**优化 1: 标签列表带计数**
```sql
SELECT
    t.*,
    COUNT(dt.document_id) as document_count
FROM tags t
LEFT JOIN document_tags dt ON t.id = dt.tag_id
WHERE t.owner_id = $1
GROUP BY t.id
ORDER BY t.name;
```

**优化 2: 按标签筛选（AND 模式）**
```sql
SELECT d.*, ...
FROM documents d
WHERE d.id IN (
    SELECT document_id
    FROM document_tags
    WHERE tag_id = ANY($1)
    GROUP BY document_id
    HAVING COUNT(DISTINCT tag_id) = $2  -- 标签数量
)
```

**优化 3: 按标签筛选（OR 模式）**
```sql
SELECT DISTINCT d.*, ...
FROM documents d
INNER JOIN document_tags dt ON d.id = dt.document_id
WHERE dt.tag_id = ANY($1)
```

---

## 9. 数据验证

### 9.1 标签名称验证

- 长度：1-50 字符
- 不能为空或只包含空格
- 同一用户不能有重名标签

### 9.2 颜色验证

- 格式：`#RRGGBB`
- 正则表达式：`^#[0-9A-Fa-f]{6}$`
- 默认值：`#3B82F6`（蓝色）

### 9.3 关联验证

- 标签必须存在
- 文档必须存在
- 用户必须有权限操作标签和文档
- 防止重复关联

---

## 10. 错误处理

### 10.1 错误码

| 错误码 | 场景 | 说明 |
|--------|------|-----|
| 400 | 标签名称为空 | 标签名称必须非空 |
| 400 | 颜色格式错误 | 颜色必须是 #RRGGBB 格式 |
| 403 | 无权限操作标签 | 只有标签所有者可以操作 |
| 404 | 标签不存在 | 指定的标签 ID 不存在 |
| 409 | 标签名称重复 | 该用户已有同名标签 |
| 409 | 标签已关联 | 文档已经有该标签 |

### 10.2 错误响应格式

```json
{
  "error": "标签名称不能为空"
}
```

---

## 11. 测试计划

### 11.1 单元测试

- ✅ 创建标签
- ✅ 更新标签
- ✅ 删除标签
- ✅ 为文档添加标签
- ✅ 从文档移除标签
- ✅ 批量设置标签
- ✅ 按标签筛选文档

### 11.2 集成测试

```bash
# scripts/test_tags.sh
1. 创建多个标签
2. 为文档添加标签
3. 获取文档的标签列表
4. 按标签筛选文档（AND 模式）
5. 按标签筛选文档（OR 模式）
6. 批量设置标签
7. 更新标签
8. 删除标签
9. 验证级联删除
```

---

## 12. 后续扩展

### 12.1 预留功能

- **标签分组**: 为标签添加分类
- **标签建议**: 根据文档内容自动推荐标签
- **热门标签**: 统计最常用的标签
- **标签合并**: 合并重复或相似的标签
- **标签共享**: 允许多个用户使用相同的标签系统

### 12.2 API 扩展

- `GET /api/tags/popular` - 热门标签
- `POST /api/tags/merge` - 合并标签
- `GET /api/tags/suggestions` - 标签建议

---

## 13. 技术要点

### 13.1 多对多关系

标签和文档是多对多关系，通过 `document_tags` 中间表实现。

### 13.2 级联删除

- 删除用户 → 删除其所有标签 → 删除标签关联
- 删除文档 → 删除其标签关联
- 删除标签 → 删除其文档关联

### 13.3 唯一性约束

```sql
CONSTRAINT tags_name_owner_unique UNIQUE (name, owner_id)
```

确保同一用户的标签名称唯一。

---

## 14. 实现优先级

### Phase 1: 基础功能 ✅
- 标签 CRUD
- 文档标签关联
- 基础筛选

### Phase 2: 高级功能 ⏳
- 批量操作
- 多标签筛选（AND/OR）
- 标签统计

### Phase 3: 优化 ⏳
- 性能优化
- 缓存策略
- 批量更新

---

**最后更新**: 2026-01-01
**版本**: 1.0.0
**状态**: 设计完成，待实现
