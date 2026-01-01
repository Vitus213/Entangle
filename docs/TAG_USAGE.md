# 标签系统使用文档

## 1. 概述

标签系统允许用户通过自定义标签对文档进行分类和快速检索，支持颜色自定义和灵活的筛选方式。

### 核心特性

- ✅ 创建、更新、删除标签
- ✅ 标签颜色自定义
- ✅ 为文档添加/移除标签
- ✅ 按标签筛选文档（AND/OR 模式）
- ✅ 批量设置文档标签
- ✅ 标签使用统计
- ✅ 用户级别隔离

---

## 2. API 端点

### 2.1 标签 CRUD

#### 创建标签

```http
POST /api/tags
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "前端开发",
  "color": "#3B82F6"  // 可选，默认 #3B82F6（蓝色）
}
```

**响应 (200):**
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

**特性：** 按标签名称排序，包含文档计数

#### 更新标签

```http
PUT /api/tags/:id
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "前端框架",     // 可选
  "color": "#10B981"      // 可选
}
```

#### 删除标签

```http
DELETE /api/tags/:id
Authorization: Bearer <token>
```

**响应 (204):** 无内容

**注意：** 删除标签会自动移除所有文档的该标签关联

---

### 2.2 文档标签管理

#### 为文档添加标签

```http
POST /api/documents/:id/tags
Authorization: Bearer <token>
Content-Type: application/json

{
  "tag_id": "tag-uuid"
}
```

**响应 (201):** 成功

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
  "tag_ids": ["tag-uuid-1", "tag-uuid-2", "tag-uuid-3"]
}
```

**响应 (200):** 更新后的标签列表

**注意：** 会先清空文档的所有标签，然后添加新标签

#### 从文档移除标签

```http
DELETE /api/documents/:id/tags/:tag_id
Authorization: Bearer <token>
```

**响应 (204):** 无内容

---

### 2.3 按标签筛选文档

```http
GET /api/documents/by-tags?tag_ids=uuid1,uuid2&match_mode=all
Authorization: Bearer <token>
```

**查询参数：**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| tag_ids | string | 是 | 逗号分隔的标签 ID 列表 |
| match_mode | string | 否 | 匹配模式：`any`(默认) 或 `all` |

**匹配模式说明：**
- `any`: OR 模式 - 文档包含任一标签即可
- `all`: AND 模式 - 文档必须包含所有指定标签

**响应 (200):**
```json
[
  {
    "id": "doc-uuid",
    "title": "文档标题",
    "owner": {
      "id": "user-uuid",
      "nickname": "用户名",
      "email": "user@example.com"
    },
    "is_public": false,
    "created_at": "2026-01-01T00:00:00Z",
    "updated_at": "2026-01-01T00:00:00Z"
  }
]
```

---

## 3. 使用场景

### 场景 1: 基础标签管理

```bash
# 1. 创建标签
TOKEN="your-jwt-token"
curl -X POST http://localhost:3000/api/tags \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"前端","color":"#3B82F6"}'

# 2. 列出所有标签
curl http://localhost:3000/api/tags \
  -H "Authorization: Bearer $TOKEN"

# 3. 更新标签
curl -X PUT http://localhost:3000/api/tags/<tag-id> \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"React 开发","color":"#6366F1"}'
```

### 场景 2: 为文档添加标签

```bash
# 1. 为文档添加单个标签
curl -X POST http://localhost:3000/api/documents/<doc-id>/tags \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"tag_id":"<tag-id>"}'

# 2. 查看文档的所有标签
curl http://localhost:3000/api/documents/<doc-id>/tags \
  -H "Authorization: Bearer $TOKEN"

# 3. 批量设置文档标签
curl -X PUT http://localhost:3000/api/documents/<doc-id>/tags \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"tag_ids":["<tag-id-1>","<tag-id-2>"]}'
```

### 场景 3: 按标签筛选文档

```bash
# OR 模式：查找带"前端"或"后端"标签的文档
curl "http://localhost:3000/api/documents/by-tags?tag_ids=<tag1>,<tag2>&match_mode=any" \
  -H "Authorization: Bearer $TOKEN"

# AND 模式：查找同时带"前端"和"React"标签的文档
curl "http://localhost:3000/api/documents/by-tags?tag_ids=<tag1>,<tag2>&match_mode=all" \
  -H "Authorization: Bearer $TOKEN"
```

### 场景 4: 标签组织策略

**技术栈标签：**
```
前端: #3B82F6 (蓝色)
后端: #10B981 (绿色)
数据库: #F59E0B (黄色)
DevOps: #8B5CF6 (紫色)
```

**项目阶段标签：**
```
规划中: #6B7280 (灰色)
开发中: #3B82F6 (蓝色)
测试中: #F59E0B (黄色)
已完成: #10B981 (绿色)
```

**优先级标签：**
```
低优先级: #6B7280 (灰色)
中优先级: #F59E0B (黄色)
高优先级: #EF4444 (红色)
紧急: #DC2626 (深红)
```

---

## 4. 权限控制

| 操作 | 权限要求 |
|------|---------|
| 创建标签 | 任何已登录用户 |
| 查看标签 | 标签所有者 |
| 更新标签 | 标签所有者 |
| 删除标签 | 标签所有者 |
| 为文档添加标签 | 文档所有者 或 协作者(write/admin) + 标签所有者 |
| 从文档移除标签 | 文档所有者 或 协作者(write/admin) |
| 查看文档标签 | 文档读权限 |
| 按标签筛选文档 | 只能使用自己的标签筛选自己的文档 |

---

## 5. 数据验证

### 标签名称

- **长度**: 1-50 个字符
- **不能为空**: 不能只包含空格
- **唯一性**: 同一用户不能有重名标签

### 标签颜色

- **格式**: `#RRGGBB` (十六进制)
- **示例**: `#3B82F6`, `#10B981`, `#F59E0B`
- **默认值**: `#3B82F6` (蓝色)

### 常用颜色推荐

```
蓝色系: #3B82F6, #2563EB, #1D4ED8, #6366F1
绿色系: #10B981, #059669, #047857, #22C55E
黄色系: #F59E0B, #D97706, #B45309, #FBBF24
红色系: #EF4444, #DC2626, #B91C1C, #F87171
紫色系: #8B5CF6, #7C3AED, #6D28D9, #A78BFA
灰色系: #6B7280, #4B5563, #374151, #9CA3AF
```

---

## 6. 错误处理

### 常见错误

| 错误码 | 场景 | 说明 |
|--------|------|-----|
| 400 | 标签名称为空 | 标签名称必须非空 |
| 400 | 颜色格式错误 | 颜色必须是 #RRGGBB 格式 |
| 400 | 无效的标签 ID | tag_ids 参数格式错误 |
| 403 | 无权限操作标签 | 只有标签所有者可以操作 |
| 403 | 无权编辑文档 | 需要文档写权限才能添加标签 |
| 404 | 标签不存在 | 指定的标签 ID 不存在 |
| 409 | 标签名称重复 | 该用户已有同名标签 |

### 错误响应格式

```json
{
  "error": "错误描述信息"
}
```

---

## 7. 最佳实践

### 7.1 标签命名

**推荐：**
- 简短明了：`React`, `API`, `Bug`
- 统一风格：全部使用中文或英文
- 有意义：`前端开发` 而不是 `标签1`

**不推荐：**
- 过长：`这是一个非常非常长的标签名称...`
- 特殊字符：`#前端@`, `后端***`
- 重复信息：`标签-前端-开发-React`

### 7.2 颜色使用

- **技术类**: 使用蓝色系
- **状态类**: 使用红黄绿表示状态
- **分类类**: 使用不同色系区分
- **保持一致**: 同类标签使用相近颜色

### 7.3 标签数量

- **建议**: 每个文档 2-5 个标签
- **最多**: 不超过 10 个标签
- **避免**: 标签过多导致分类混乱

### 7.4 标签整理

定期整理标签：
1. 删除不再使用的标签
2. 合并相似标签
3. 重命名不清晰的标签
4. 统一标签颜色方案

---

## 8. 性能优化

### 8.1 索引

系统已创建以下索引：
```sql
CREATE INDEX idx_tags_owner ON tags(owner_id);
CREATE INDEX idx_tags_name ON tags(name);
CREATE INDEX idx_document_tags_document ON document_tags(document_id);
CREATE INDEX idx_document_tags_tag ON document_tags(tag_id);
```

### 8.2 查询优化

- 标签列表自动包含文档计数（一次查询）
- 按标签筛选使用优化的 SQL（AND/OR 模式）
- 批量设置标签使用事务保证一致性

---

## 9. 集成示例

### JavaScript/TypeScript

```typescript
// 标签服务类
class TagService {
  private baseUrl = 'http://localhost:3000/api';
  private token: string;

  constructor(token: string) {
    this.token = token;
  }

  // 创建标签
  async createTag(name: string, color: string = '#3B82F6') {
    const response = await fetch(`${this.baseUrl}/tags`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.token}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ name, color })
    });
    return response.json();
  }

  // 获取所有标签
  async getTags() {
    const response = await fetch(`${this.baseUrl}/tags`, {
      headers: { 'Authorization': `Bearer ${this.token}` }
    });
    return response.json();
  }

  // 为文档添加标签
  async addTagToDocument(docId: string, tagId: string) {
    const response = await fetch(`${this.baseUrl}/documents/${docId}/tags`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.token}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ tag_id: tagId })
    });
    return response.status === 201;
  }

  // 按标签筛选文档
  async filterDocumentsByTags(tagIds: string[], matchAll: boolean = false) {
    const mode = matchAll ? 'all' : 'any';
    const response = await fetch(
      `${this.baseUrl}/documents/by-tags?tag_ids=${tagIds.join(',')}&match_mode=${mode}`,
      { headers: { 'Authorization': `Bearer ${this.token}` } }
    );
    return response.json();
  }
}
```

### Python

```python
import requests

class TagService:
    def __init__(self, token: str):
        self.base_url = 'http://localhost:3000/api'
        self.headers = {
            'Authorization': f'Bearer {token}',
            'Content-Type': 'application/json'
        }

    def create_tag(self, name: str, color: str = '#3B82F6'):
        response = requests.post(
            f'{self.base_url}/tags',
            headers=self.headers,
            json={'name': name, 'color': color}
        )
        return response.json()

    def get_tags(self):
        response = requests.get(
            f'{self.base_url}/tags',
            headers=self.headers
        )
        return response.json()

    def filter_documents_by_tags(self, tag_ids: list, match_all: bool = False):
        mode = 'all' if match_all else 'any'
        response = requests.get(
            f'{self.base_url}/documents/by-tags',
            headers=self.headers,
            params={'tag_ids': ','.join(tag_ids), 'match_mode': mode}
        )
        return response.json()
```

---

## 10. 测试

### 运行测试脚本

```bash
# 确保 API 服务器正在运行
./scripts/test_tags.sh
```

### 测试覆盖

测试脚本包含以下场景：
- ✅ 创建多个标签
- ✅ 列出所有标签
- ✅ 更新标签
- ✅ 为文档添加标签
- ✅ 获取文档标签列表
- ✅ 按标签筛选（OR 模式）
- ✅ 按标签筛选（AND 模式）
- ✅ 批量设置标签
- ✅ 移除标签
- ✅ 删除标签
- ✅ 验证文档计数

---

## 11. 常见问题

### Q1: 为什么我不能使用别人的标签？

**A:** 标签是用户私有的，每个用户只能使用自己创建的标签。这样设计是为了避免标签命名冲突和权限混乱。

### Q2: 删除标签后，文档会被删除吗？

**A:** 不会。删除标签只会移除标签与文档的关联关系，文档本身不受影响。

### Q3: 可以创建多少个标签？

**A:** 没有硬性限制，但建议保持在 20-50 个标签之间，便于管理。

### Q4: 如何批量为多个文档添加标签？

**A:** 目前需要逐个调用 API。未来版本可能会支持批量操作。

### Q5: 标签颜色可以自定义吗？

**A:** 可以。支持任何有效的十六进制颜色值（#RRGGBB 格式）。

---

## 12. 相关文档

- [标签系统设计文档](./TAG_DESIGN.md) - 详细的技术设计
- [测试脚本](../scripts/test_tags.sh) - 自动化测试
- [文档系统使用文档](./DOCUMENT_USAGE.md) - 文档功能
- [文件夹系统使用文档](./FOLDER_USAGE.md) - 文件夹功能

---

**最后更新**: 2026-01-01
**版本**: 1.0.0
**状态**: 生产就绪 ✅
