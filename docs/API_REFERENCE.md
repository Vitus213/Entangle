# Entangle API 接口文档

> RESTful API 和 WebSocket 接口完整说明

---

## 目录

1. [认证说明](#1-认证说明)
2. [RESTful API](#2-restful-api)
3. [WebSocket 协议](#3-websocket-协议)
4. [数据模型](#4-数据模型)

---

## 1. 认证说明

### 1.1 JWT Token

大多数 API 需要在请求头中携带 JWT Token：

```http
Authorization: Bearer <token>
```

**获取 Token**：通过登录或注册接口获取

### 1.2 Token 示例

```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "nickname": "张三"
  }
}
```

---

## 2. RESTful API

### 基础 URL

```
http://localhost:3000/api
```

---

### 2.1 认证模块 (`/api/auth`)

#### 注册

```http
POST /api/auth/register
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "password123",
  "nickname": "张三"
}
```

**响应**：
```json
{
  "token": "jwt_token_here",
  "user": {
    "id": "uuid",
    "email": "user@example.com",
    "nickname": "张三"
  }
}
```

#### 登录

```http
POST /api/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "password123"
}
```

**响应**：同注册

---

### 2.2 文档模块 (`/api/documents`)

#### 获取可访问文档列表

```http
GET /api/documents/accessible
Authorization: Bearer <token>
```

**响应**：
```json
[
  {
    "id": "doc_uuid",
    "title": "项目文档",
    "owner": {
      "id": "user_uuid",
      "nickname": "张三",
      "email": "user@example.com"
    },
    "is_public": false,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-02T00:00:00Z"
  }
]
```

#### 创建文档

```http
POST /api/documents
Authorization: Bearer <token>
Content-Type: application/json

{
  "title": "新文档",
  "content": "初始内容",
  "is_public": false,
  "folder_id": "folder_uuid"  // 可选
}
```

**响应**：
```json
{
  "id": "new_doc_uuid",
  "title": "新文档",
  "content": "初始内容",
  "crdt_state": "hex_encoded_state",
  "owner": { /* ... */ },
  "is_public": false,
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

#### 获取单个文档

```http
GET /api/documents/{doc_id}
Authorization: Bearer <token>
```

**响应**：同创建文档响应

#### 更新文档

```http
PUT /api/documents/{doc_id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "title": "更新后的标题",
  "content": "更新后的内容"
}
```

#### 删除文档

```http
DELETE /api/documents/{doc_id}
Authorization: Bearer <token>
```

#### 搜索文档

```http
GET /api/documents/search?q=关键词&limit=20
Authorization: Bearer <token>
```

---

### 2.3 协作者管理 (`/api/documents/{doc_id}/collaborators`)

#### 获取协作者列表

```http
GET /api/documents/{doc_id}/collaborators
Authorization: Bearer <token>
```

**响应**：
```json
[
  {
    "user_id": "user_uuid",
    "nickname": "李四",
    "email": "lisi@example.com",
    "permission": "write",
    "created_at": "2024-01-01T00:00:00Z"
  }
]
```

#### 添加协作者

```http
POST /api/documents/{doc_id}/collaborators
Authorization: Bearer <token>
Content-Type: application/json

{
  "email": "lisi@example.com",
  "permission": "write"  // "read" | "write" | "admin"
}
```

#### 移除协作者

```http
DELETE /api/documents/{doc_id}/collaborators/{user_id}
Authorization: Bearer <token>
```

---

### 2.4 文件夹模块 (`/api/folders`)

#### 获取文件夹树

```http
GET /api/folders/tree
Authorization: Bearer <token>
```

**响应**：
```json
[
  {
    "id": "folder_uuid",
    "name": "工作文档",
    "parent_id": null,
    "children": [
      {
        "id": "sub_folder_uuid",
        "name": "项目 A",
        "parent_id": "folder_uuid",
        "children": []
      }
    ]
  }
]
```

#### 创建文件夹

```http
POST /api/folders
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "新文件夹",
  "parent_id": "parent_uuid"  // 可选
}
```

---

### 2.5 标签模块 (`/api/tags`)

#### 获取标签列表

```http
GET /api/tags
Authorization: Bearer <token>
```

**响应**：
```json
[
  {
    "id": "tag_uuid",
    "name": "重要",
    "color": "#ff0000",
    "document_count": 5,
    "owner_id": "user_uuid"
  }
]
```

#### 创建标签

```http
POST /api/tags
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "紧急",
  "color": "#ff0000"
}
```

---

### 2.6 评论模块 (`/api/comments`, `/api/documents/{doc_id}/comments`)

#### 获取文档评论

```http
GET /api/documents/{doc_id}/comments
Authorization: Bearer <token>
```

**响应**：
```json
[
  {
    "id": "comment_uuid",
    "doc_id": "doc_uuid",
    "user": {
      "id": "user_uuid",
      "nickname": "张三",
      "avatar_url": null
    },
    "parent_id": null,
    "content": "这里写得很好",
    "position": { "start": 0, "end": 10 },
    "is_resolved": false,
    "reply_count": 2,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  }
]
```

#### 创建评论

```http
POST /api/comments
Authorization: Bearer <token>
Content-Type: application/json

{
  "doc_id": "doc_uuid",
  "content": "评论内容",
  "parent_id": "parent_comment_uuid",  // 可选，回复评论
  "position": { "start": 0, "end": 10 }  // 可选
}
```

#### 解决/取消解决评论

```http
PUT /api/comments/{comment_id}/resolve
PUT /api/comments/{comment_id}/unresolve
Authorization: Bearer <token>
```

#### 删除评论

```http
DELETE /api/comments/{comment_id}
Authorization: Bearer <token>
```

---

### 2.7 通知模块 (`/api/notifications`)

#### 获取通知列表

```http
GET /api/notifications?limit=50
Authorization: Bearer <token>
```

**响应**：
```json
[
  {
    "id": "notif_uuid",
    "notification_type": "comment",
    "title": "新评论",
    "content": "张三 评论了你的文档",
    "resource_type": "document",
    "resource_id": "doc_uuid",
    "sender": {
      "id": "user_uuid",
      "nickname": "张三",
      "avatar_url": null
    },
    "is_read": false,
    "created_at": "2024-01-01T00:00:00Z"
  }
]
```

#### 获取未读数

```http
GET /api/notifications/unread-count
Authorization: Bearer <token>
```

**响应**：
```json
{
  "count": 5
}
```

#### 标记已读

```http
PUT /api/notifications/{notif_id}/read
Authorization: Bearer <token>
```

#### 全部标记已读

```http
PUT /api/notifications/read-all
Authorization: Bearer <token>
```

#### 删除通知

```http
DELETE /api/notifications/{notif_id}
Authorization: Bearer <token>
```

---

### 2.8 任务模块 (`/api/tasks`)

#### 获取任务列表

```http
GET /api/tasks?filter=all&limit=50
Authorization: Bearer <token>
```

**filter**: `all` | `assigned` | `created` | `pending` | `completed`

**响应**：
```json
[
  {
    "id": "task_uuid",
    "doc_id": "doc_uuid",
    "doc_title": "项目文档",
    "title": "完成草稿",
    "assignee": {
      "id": "user_uuid",
      "nickname": "李四",
      "avatar_url": null
    },
    "status": "pending",
    "priority": "high",
    "due_date": "2024-01-10T00:00:00Z",
    "created_at": "2024-01-01T00:00:00Z"
  }
]
```

#### 创建任务

```http
POST /api/tasks
Authorization: Bearer <token>
Content-Type: application/json

{
  "title": "新任务",
  "description": "任务描述",
  "doc_id": "doc_uuid",      // 可选
  "assignee_id": "user_uuid", // 可选
  "priority": "medium",      // "low" | "medium" | "high" | "urgent"
  "due_date": "2024-01-10"   // 可选
}
```

#### 更新任务状态

```http
PUT /api/tasks/{task_id}/status
Authorization: Bearer <token>
Content-Type: application/json

{
  "status": "in_progress"  // "pending" | "in_progress" | "completed" | "cancelled"
}
```

#### 删除任务

```http
DELETE /api/tasks/{task_id}
Authorization: Bearer <token>
```

---

### 2.9 版本控制模块 (`/api/versions`)

#### 获取文档版本历史

```http
GET /api/documents/{doc_id}/versions
Authorization: Bearer <token>
```

**响应**：
```json
[
  {
    "id": "version_uuid",
    "doc_id": "doc_uuid",
    "version_number": 1,
    "title": "文档标题",
    "content": "文档内容",
    "created_by": "user_uuid",
    "description": "初始版本",
    "created_at": "2024-01-01T00:00:00Z"
  }
]
```

#### 创建版本快照

```http
POST /api/documents/{doc_id}/versions
Authorization: Bearer <token>
Content-Type: application/json

{
  "description": "重要更新前的快照"
}
```

#### 恢复到指定版本

```http
POST /api/documents/{doc_id}/versions/{version_id}/restore
Authorization: Bearer <token>
```

---

### 2.10 用户模块 (`/api/users`)

#### 获取当前用户信息

```http
GET /api/users/me
Authorization: Bearer <token>
```

**响应**：
```json
{
  "id": "user_uuid",
  "email": "user@example.com",
  "nickname": "张三",
  "avatar_url": "https://...",
  "role": "user",
  "email_verified": true,
  "created_at": "2024-01-01T00:00:00Z"
}
```

#### 更新用户信息

```http
PUT /api/users/me
Authorization: Bearer <token>
Content-Type: application/json

{
  "nickname": "新昵称",
  "avatar_url": "https://..."  // 可选
}
```

#### 上传头像

```http
POST /api/users/me/avatar
Authorization: Bearer <token>
Content-Type: multipart/form-data

avatar: <图片文件>
```

---

### 2.11 角色和权限 (管理员)

#### 获取所有用户

```http
GET /api/users?limit=100
Authorization: Bearer <token>
```

#### 获取所有角色

```http
GET /api/roles
Authorization: Bearer <token>
```

#### 更新用户角色

```http
POST /api/users/{user_id}/role
Authorization: Bearer <token>
Content-Type: application/json

{
  "role_id": "admin_role_uuid"
}
```

---

## 3. WebSocket 协议

### 3.1 连接 URL

```
ws://localhost:3000/ws/documents/{doc_id}?token={jwt_token}
```

### 3.2 消息格式

所有消息都是 JSON 格式：

```json
{
  "type": "message_type",
  "data": { /* ... */ }
}
```

### 3.3 消息类型

#### 3.3.1 同步更新 (sync)

**客户端 → 服务器**：

```json
{
  "type": "sync",
  "update": "文档内容或十六进制CRDT状态"
}
```

#### 3.3.2 用户感知状态 (awareness)

**客户端 → 服务器**：

```json
{
  "type": "awareness",
  "state": {
    "user": {
      "user_id": "user_uuid",
      "nickname": "张三",
      "avatar_url": null
    },
    "cursor": {
      "line": 10,
      "column": 20
    },
    "selection": {
      "start": { "line": 10, "column": 20 },
      "end": { "line": 12, "column": 5 }
    },
    "color": "#FF6B6B"
  }
}
```

#### 3.3.3 用户加入 (user_joined)

**服务器 → 客户端**：

```json
{
  "type": "user_joined",
  "user_id": "user_uuid",
  "nickname": "李四"
}
```

#### 3.3.4 用户离开 (user_left)

**服务器 → 客户端**：

```json
{
  "type": "user_left",
  "user_id": "user_uuid"
}
```

#### 3.3.5 错误 (error)

**服务器 → 客户端**：

```json
{
  "type": "error",
  "message": "错误描述"
}
```

### 3.4 连接生命周期

1. **握手**：客户端发送带 token 的 WebSocket 连接请求
2. **验证**：服务器验证 token 和文档权限
3. **初始化**：服务器发送当前文档内容和在线用户列表
4. **通信**：双向收发消息
5. **心跳**：服务器每 30 秒发送 ping
6. **关闭**：用户离开或超时断开

---

## 4. 数据模型

### 4.1 用户 (User)

```typescript
{
  id: string;              // UUID
  email: string;           // 邮箱
  nickname: string;        // 昵称
  avatar_url?: string;     // 头像 URL
  role?: string;           // 角色: "user" | "admin"
  email_verified: boolean; // 邮箱验证状态
  created_at: string;      // ISO 8601
}
```

### 4.2 文档 (Document)

```typescript
{
  id: string;              // UUID
  title: string;           // 标题
  content: string;         // 文本内容
  crdt_state?: string;     // 十六进制 CRDT 状态
  owner: {
    id: string;
    nickname: string;
    email: string;
  };
  is_public: boolean;      // 是否公开
  created_at: string;
  updated_at: string;
}
```

### 4.3 协作者权限 (CollaboratorPermission)

```typescript
type CollaboratorPermission = "read" | "write" | "admin";
```

- **read**: 只读
- **write**: 可编辑内容
- **admin**: 可管理其他协作者

### 4.4 任务状态 (TaskStatus)

```typescript
type TaskStatus = "pending" | "in_progress" | "completed" | "cancelled";
```

### 4.5 任务优先级 (TaskPriority)

```typescript
type TaskPriority = "low" | "medium" | "high" | "urgent";
```

---

## 5. 错误响应

### 5.1 错误格式

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "错误描述"
  }
}
```

### 5.2 HTTP 状态码

| 状态码 | 含义 |
|--------|------|
| 200 | 成功 |
| 201 | 创建成功 |
| 400 | 请求参数错误 |
| 401 | 未认证 |
| 403 | 无权限 |
| 404 | 资源不存在 |
| 500 | 服务器错误 |

---

## 6. 代码示例

### 6.1 JavaScript/TypeScript 客户端

```typescript
// 登录
const loginResponse = await fetch('http://localhost:3000/api/auth/login', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    email: 'user@example.com',
    password: 'password123'
  })
});
const { token } = await loginResponse.json();

// 获取文档列表
const docsResponse = await fetch('http://localhost:3000/api/documents/accessible', {
  headers: { 'Authorization': `Bearer ${token}` }
});
const documents = await docsResponse.json();

// 建立 WebSocket 连接
const ws = new WebSocket(`ws://localhost:3000/ws/documents/${docId}?token=${token}`);

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log('收到消息:', message);
};
```

### 6.2 Rust 客户端 (reqwest)

```rust
use reqwest::Client;

let client = Client::new();

// 登录
let login_response: AuthResponse = client
    .post("http://localhost:3000/api/auth/login")
    .json(&serde_json::json!({
        "email": "user@example.com",
        "password": "password123"
    }))
    .send()
    .await?
    .json()
    .await?;

// 获取文档列表
let documents: Vec<Document> = client
    .get("http://localhost:3000/api/documents/accessible")
    .header("Authorization", format!("Bearer {}", login_response.token))
    .send()
    .await?
    .json()
    .await?;
```

---

文档版本: v1.0
最后更新: 2024-01-05
