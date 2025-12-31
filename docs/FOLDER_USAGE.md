# 文件夹功能使用文档

## 概述

文件夹系统允许用户以层级结构组织文档，支持创建、更新、删除文件夹，以及在文件夹间移动文档。

## 功能特性

- ✅ 创建多级文件夹层次结构
- ✅ 重命名和移动文件夹
- ✅ 在文件夹间移动文档
- ✅ 查看文件夹树形结构
- ✅ 浏览文件夹内容
- ✅ 级联删除（删除文件夹时会删除其子文件夹）
- ✅ 权限控制（只有所有者可以操作文件夹）

## API 端点

### 1. 创建文件夹

**请求:**
```http
POST /api/folders
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "My Folder",
  "parent_id": null  // null 表示根文件夹，否则为父文件夹 ID
}
```

**响应:**
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "name": "My Folder",
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

### 2. 获取文件夹详情

**请求:**
```http
GET /api/folders/:id
Authorization: Bearer <token>
```

**响应:**
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "name": "My Folder",
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

### 3. 更新文件夹

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

**响应:**
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "name": "Renamed Folder",
  "parent_id": "new-parent-id",
  "owner": {
    "id": "user-id",
    "nickname": "DemoUser",
    "email": "demo@example.com"
  },
  "created_at": "2026-01-01T00:00:00Z",
  "updated_at": "2026-01-01T00:01:00Z"
}
```

### 4. 删除文件夹

**请求:**
```http
DELETE /api/folders/:id
Authorization: Bearer <token>
```

**响应:**
```http
204 No Content
```

**注意:** 删除文件夹会级联删除所有子文件夹。文件夹内的文档不会被删除，但会失去文件夹关联。

### 5. 获取文件夹树

**请求:**
```http
GET /api/folders/tree
Authorization: Bearer <token>
```

**响应:**
```json
[
  {
    "id": "root-folder-id",
    "name": "Projects",
    "parent_id": null,
    "owner_id": "user-id",
    "created_at": "2026-01-01T00:00:00Z",
    "updated_at": "2026-01-01T00:00:00Z",
    "document_count": 5,
    "children": [
      {
        "id": "sub-folder-id",
        "name": "Frontend",
        "parent_id": "root-folder-id",
        "owner_id": "user-id",
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-01T00:00:00Z",
        "document_count": 3,
        "children": []
      }
    ]
  }
]
```

### 6. 获取文件夹内容

**请求:**
```http
GET /api/folders/:id/contents
Authorization: Bearer <token>
```

**响应:**
```json
{
  "folder": {
    "id": "folder-id",
    "name": "My Folder",
    "path": ["Projects", "My Folder"]  // 从根到当前的完整路径
  },
  "subfolders": [
    {
      "id": "subfolder-id",
      "name": "Subfolder",
      "document_count": 2
    }
  ],
  "documents": [
    {
      "id": "doc-id",
      "title": "Document Title",
      "owner": {
        "id": "owner-id",
        "nickname": "Owner Name",
        "email": "owner@example.com"
      },
      "is_public": false,
      "created_at": "2026-01-01T00:00:00Z",
      "updated_at": "2026-01-01T00:00:00Z"
    }
  ]
}
```

### 7. 移动文档到文件夹

**请求:**
```http
PUT /api/documents/:id/move
Authorization: Bearer <token>
Content-Type: application/json

{
  "folder_id": "target-folder-id"  // null 表示移动到根目录
}
```

**响应:**
```http
204 No Content
```

## 权限控制

| 操作 | 权限要求 |
|------|---------|
| 创建文件夹 | 任何已登录用户 |
| 查看文件夹 | 文件夹所有者 |
| 更新文件夹 | 文件夹所有者 |
| 删除文件夹 | 文件夹所有者 |
| 获取文件夹树 | 任何已登录用户（仅显示自己的文件夹） |
| 移动文档 | 文档所有者 + 目标文件夹所有者 |

## 使用示例

### 示例 1: 创建层级文件夹结构

```bash
# 1. 创建根文件夹
curl -X POST http://localhost:3000/api/folders \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"Projects","parent_id":null}'

# 响应中获取 folder_id

# 2. 创建子文件夹
curl -X POST http://localhost:3000/api/folders \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"Frontend","parent_id":"<root-folder-id>"}'
```

### 示例 2: 移动文档到文件夹

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

### 示例 3: 浏览文件夹内容

```bash
# 获取文件夹树
curl http://localhost:3000/api/folders/tree \
  -H "Authorization: Bearer $TOKEN"

# 获取特定文件夹的内容
curl http://localhost:3000/api/folders/<folder-id>/contents \
  -H "Authorization: Bearer $TOKEN"
```

## 错误处理

### 常见错误码

| 错误码 | 说明 |
|--------|-----|
| 401 Unauthorized | 未提供有效的认证令牌 |
| 403 Forbidden | 无权访问该文件夹 |
| 404 Not Found | 文件夹不存在 |
| 500 Internal Server Error | 服务器内部错误 |

### 错误响应格式

```json
{
  "error": "错误描述信息"
}
```

## 最佳实践

1. **文件夹命名**
   - 使用清晰描述性的名称
   - 避免使用特殊字符
   - 保持名称简短（建议不超过 50 个字符）

2. **层级深度**
   - 建议不超过 5 层深度
   - 过深的层级会影响查询性能

3. **批量操作**
   - 移动多个文档时，逐个调用移动API
   - 考虑在客户端实现批量操作的进度提示

4. **删除操作**
   - 删除文件夹前确认不再需要
   - 删除是级联的，会同时删除所有子文件夹
   - 文件夹内的文档不会被删除，但会失去关联

## 性能优化

1. **文件夹树查询**
   - 使用 PostgreSQL 递归 CTE 优化查询
   - 文档数量统计已预先计算

2. **路径查询**
   - 使用递归查询一次性获取完整路径
   - 避免多次查询数据库

3. **缓存建议**
   - 客户端可缓存文件夹树结构
   - 文件夹变更后刷新缓存

## 数据库模式

```sql
CREATE TABLE folders (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    parent_id UUID REFERENCES folders(id) ON DELETE CASCADE,
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE documents
  ADD COLUMN folder_id UUID REFERENCES folders(id) ON DELETE SET NULL;
```

## 技术实现

- **递归 CTE**: 用于高效查询文件夹树和路径
- **级联删除**: 使用数据库外键约束实现
- **权限检查**: 在 API 层面进行所有者验证
- **openGauss 兼容**: 使用标准 SQL 语法，确保跨数据库兼容性

## 相关文档

- [架构设计文档](./FOLDER_DESIGN.md) - 详细的技术设计
- [实现状态文档](./FOLDER_IMPLEMENTATION_STATUS.md) - 实现进度跟踪
- [测试脚本](../scripts/test_folders.sh) - 自动化测试脚本

## 更新日志

### v1.0.0 (2026-01-01)
- ✅ 初始版本发布
- ✅ 基本 CRUD 操作
- ✅ 文件夹树和内容浏览
- ✅ 文档移动功能
- ✅ 级联删除支持
- ✅ 完整的权限控制

---

**最后更新**: 2026-01-01
**版本**: 1.0.0
