# Entangle 认证授权系统

完整的 RBAC (基于角色的访问控制) 系统已经实现完成。

## 功能特性

### 1. 用户认证
- ✅ 用户注册（自动分配 viewer 角色）
- ✅ 用户登录（JWT token）
- ✅ 密码加密（Argon2）
- ✅ Token 验证中间件

### 2. 权限管理
- ✅ 基于角色的访问控制（RBAC）
- ✅ 三种默认角色：admin、editor、viewer
- ✅ 六种默认权限：
  - `document:create` - 创建文档
  - `document:read` - 查看文档
  - `document:update` - 编辑文档
  - `document:delete` - 删除文档
  - `user:manage` - 管理用户
  - `system:config` - 系统配置

### 3. 角色权限
| 角色 | 权限 |
|------|------|
| admin | 所有权限 |
| editor | document:* (所有文档操作) |
| viewer | document:read (只读) |

## 数据库结构

已创建以下表：
- `users` - 用户表
- `roles` - 角色表
- `permissions` - 权限表
- `role_permissions` - 角色-权限关联表

## API 端点

### 公开端点（无需认证）
- `POST /api/auth/register` - 用户注册
- `POST /api/auth/login` - 用户登录

### 受保护端点（需要认证）
- `GET /api/me` - 获取当前用户信息
- `GET /api/me/permissions` - 获取当前用户权限
- `GET /api/users` - 列出所有用户（仅管理员）
- `GET /api/users/:id` - 获取指定用户（仅管理员）
- `POST /api/users/:id/role` - 修改用户角色（仅管理员）

## 快速开始

### 1. 运行数据库迁移
```bash
DATABASE_URL='postgres://entangle:Entangle%402024@localhost:5432/postgres' sqlx migrate run
```

### 2. 启动 API 服务器
```bash
cargo run --bin entangle-api
```

### 3. 测试 API
```bash
./test_api.sh
```

## 使用示例

### 注册用户
```bash
curl -X POST http://127.0.0.1:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"password123","nickname":"用户名"}'
```

### 登录
```bash
curl -X POST http://127.0.0.1:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"password123"}'
```

### 获取用户信息（需要 token）
```bash
curl http://127.0.0.1:3000/api/me \
  -H "Authorization: Bearer YOUR_TOKEN_HERE"
```

### 获取用户权限
```bash
curl http://127.0.0.1:3000/api/me/permissions \
  -H "Authorization: Bearer YOUR_TOKEN_HERE"
```

## 代码结构

```
crates/
├── api/              # API 层（路由、中间件）
│   ├── middleware/   # 认证中间件
│   └── routes/       # API 路由
├── auth/             # 认证授权模块
│   ├── jwt.rs        # JWT token 处理
│   ├── password.rs   # 密码加密
│   └── permission.rs # 权限检查服务
├── core/             # 核心业务逻辑
│   └── error.rs      # 统一错误处理
└── db/               # 数据库访问层
    ├── models/       # 数据模型
    └── repository/   # 数据仓库
        ├── user.rs
        ├── role.rs
        └── permission.rs
```

## 扩展权限系统

### 添加新权限
```sql
INSERT INTO permissions (id, name, resource, action, description)
VALUES (gen_random_uuid(), 'blog:create', 'blog', 'create', '创建博客');
```

### 为角色添加权限
```sql
INSERT INTO role_permissions (role_id, permission_id)
SELECT
  (SELECT id FROM roles WHERE name = 'editor'),
  (SELECT id FROM permissions WHERE name = 'blog:create');
```

### 在代码中检查权限
```rust
use entangle_auth::PermissionService;

let has_permission = PermissionService::has_permission(
    &pool,
    user_id,
    "blog:create"
).await?;

if !has_permission {
    return Err(AppError::Forbidden("需要 blog:create 权限".to_string()));
}
```

## 测试结果

✅ 用户注册成功
✅ 用户登录成功
✅ 获取用户信息成功
✅ 获取用户权限成功（viewer 只有 document:read）
✅ 管理员拥有所有权限
✅ 普通用户无法访问管理员功能

## 安全考虑

1. **密码加密**：使用 Argon2 算法加密密码
2. **JWT Token**：7 天有效期
3. **权限验证**：每个请求都会验证用户状态和权限
4. **环境变量**：敏感信息（JWT_SECRET）存储在 .env 中
5. **数据库连接**：使用连接池管理

## 环境变量

在 `.env` 文件中配置：
```env
DATABASE_URL=postgres://entangle:Entangle%402024@localhost:5432/postgres
JWT_SECRET=your-generated-secret-key
```

使用 `openssl rand -hex 32` 生成安全的密钥。
