# Entangle 测试指南

> 最后更新: 2026-01-04
> 包含后端 API 测试和前端 CRDT 实时协作测试

---

## 📋 第一部分: 后端 API 测试

### 前提条件
确保 API 服务器正在运行：
```bash
cargo run --bin entangle-api
```

### 测试 1: 用户注册

```bash
curl -X POST http://127.0.0.1:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"test123","nickname":"TestUser"}'
```

**期望结果**：返回 JWT token 和用户信息
```json
{
  "token": "eyJ0eXAiOiJKV1QiLC...",
  "user": {
    "id": "uuid",
    "email": "test@example.com",
    "nickname": "TestUser",
    "role": "viewer",
    "email_verified": false
  }
}
```

---

## 测试 2: 用户登录

```bash
curl -X POST http://127.0.0.1:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"test123"}'
```

**期望结果**：返回新的 JWT token

---

## 测试 3: 获取当前用户信息（需要认证）

```bash
# 先保存 token
TOKEN="你的token"

curl http://127.0.0.1:3000/api/me \
  -H "Authorization: Bearer $TOKEN"
```

**期望结果**：返回当前用户信息

---

## 测试 4: 获取用户权限

```bash
curl http://127.0.0.1:3000/api/me/permissions \
  -H "Authorization: Bearer $TOKEN"
```

**期望结果**：
```json
["document:read"]
```
（viewer 角色只有读权限）

---

## 测试 5: 测试权限隔离

```bash
# 普通用户尝试访问管理员功能
curl http://127.0.0.1:3000/api/users \
  -H "Authorization: Bearer $TOKEN"
```

**期望结果**：
```json
{"error":"Admin permission required"}
```

---

### 测试 6: 管理员功能

### 6.1 创建管理员用户
```bash
# 先注册一个用户
curl -X POST http://127.0.0.1:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com","password":"admin123","nickname":"Admin"}'

# 手动提升为管理员（获取返回的 user.id）
psql "postgres://entangle:Entangle%402024@localhost:5432/postgres" \
  -c "UPDATE entangle.users SET role_id = '00000000-0000-0000-0000-000000000001' WHERE email = 'admin@example.com';"
```

### 6.2 管理员登录
```bash
curl -X POST http://127.0.0.1:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com","password":"admin123"}'

# 保存管理员 token
ADMIN_TOKEN="管理员的token"
```

### 6.3 管理员查看所有权限
```bash
curl http://127.0.0.1:3000/api/me/permissions \
  -H "Authorization: Bearer $ADMIN_TOKEN"
```

**期望结果**：
```json
["document:create","document:delete","document:read","document:update","system:config","user:manage"]
```

### 6.4 管理员列出所有用户
```bash
curl http://127.0.0.1:3000/api/users \
  -H "Authorization: Bearer $ADMIN_TOKEN"
```

**期望结果**：返回所有用户列表

---

### 自动化测试

运行完整测试脚本：
```bash
./full_test.sh
```

这个脚本会自动测试所有功能并显示结果。

---

### 后端功能总结

### ✅ 认证系统
- 用户注册（Argon2 密码加密）
- 用户登录（JWT Token）
- Token 验证中间件

### ✅ 授权系统
- 基于角色的访问控制（RBAC）
- 3 种角色：admin、editor、viewer
- 6 种权限：document:*, user:manage, system:config
- 权限检查服务

### ✅ API 端点
**公开端点**：
- POST /api/auth/register
- POST /api/auth/login

**受保护端点**：
- GET /api/me
- GET /api/me/permissions
- GET /api/users（仅管理员）
- GET /api/users/:id（仅管理员）
- POST /api/users/:id/role（仅管理员）

### ✅ 数据库
- openGauss 5.1.0（PostgreSQL 兼容）
- 4 张表：users, roles, permissions, role_permissions
- 完整的 CRUD Repository

### ✅ 错误处理
- 统一的错误类型
- 友好的 HTTP 响应
- 详细的错误信息

---

### 数据库角色和权限

| 角色 | 权限 |
|------|------|
| admin | document:create, document:read, document:update, document:delete, user:manage, system:config |
| editor | document:create, document:read, document:update, document:delete |
| viewer | document:read |

---

### 环境变量配置

`.env` 文件配置：
```env
DATABASE_URL=postgres://entangle:Entangle%402024@localhost:5432/postgres
JWT_SECRET=your-generated-secret-key
```

使用 `openssl rand -hex 32` 生成安全的密钥。
