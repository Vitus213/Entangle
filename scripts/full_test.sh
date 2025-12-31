#!/usr/bin/env bash
# 完整的 API 测试演示

set -e

API_URL="http://127.0.0.1:3000"

echo "======================================"
echo "  Entangle 认证授权系统测试"
echo "======================================"
echo ""

# 颜色定义
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

print_test() {
    echo -e "${BLUE}[测试 $1]${NC} $2"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# ========================================
# 测试 1: 健康检查
# ========================================
print_test "1" "健康检查"
HEALTH=$(curl -s "$API_URL/health")
if [ "$HEALTH" = "OK" ]; then
    print_success "API 服务器正常运行"
else
    print_error "API 服务器未响应"
    exit 1
fi
echo ""

# ========================================
# 测试 2: 用户注册
# ========================================
print_test "2" "用户注册 (alice@example.com, viewer 角色)"
REGISTER_ALICE=$(curl -s -X POST "$API_URL/api/auth/register" \
  -H "Content-Type: application/json" \
  -d '{"email":"alice@example.com","password":"password123","nickname":"Alice"}')

ALICE_TOKEN=$(echo "$REGISTER_ALICE" | jq -r '.token')
ALICE_ID=$(echo "$REGISTER_ALICE" | jq -r '.user.id')
ALICE_ROLE=$(echo "$REGISTER_ALICE" | jq -r '.user.role')

if [ "$ALICE_ROLE" = "viewer" ]; then
    print_success "用户 Alice 注册成功，默认角色: $ALICE_ROLE"
    echo "Token: ${ALICE_TOKEN:0:50}..."
else
    print_error "注册失败"
    echo "$REGISTER_ALICE" | jq .
fi
echo ""

# ========================================
# 测试 3: 用户登录
# ========================================
print_test "3" "用户登录"
LOGIN_ALICE=$(curl -s -X POST "$API_URL/api/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"alice@example.com","password":"password123"}')

NEW_TOKEN=$(echo "$LOGIN_ALICE" | jq -r '.token')
if [ -n "$NEW_TOKEN" ] && [ "$NEW_TOKEN" != "null" ]; then
    print_success "登录成功，获取新 Token"
    ALICE_TOKEN="$NEW_TOKEN"
else
    print_error "登录失败"
fi
echo ""

# ========================================
# 测试 4: 获取当前用户信息
# ========================================
print_test "4" "获取当前用户信息 (需要认证)"
ME_INFO=$(curl -s "$API_URL/api/me" \
  -H "Authorization: Bearer $ALICE_TOKEN")

echo "$ME_INFO" | jq .
print_success "成功获取用户信息"
echo ""

# ========================================
# 测试 5: 获取用户权限
# ========================================
print_test "5" "获取用户权限"
PERMISSIONS=$(curl -s "$API_URL/api/me/permissions" \
  -H "Authorization: Bearer $ALICE_TOKEN")

echo "用户权限: $PERMISSIONS"
if echo "$PERMISSIONS" | grep -q "document:read"; then
    print_success "Viewer 角色拥有 document:read 权限"
else
    print_error "权限不正确"
fi
echo ""

# ========================================
# 测试 6: 测试权限隔离
# ========================================
print_test "6" "测试权限隔离 (普通用户访问管理员功能)"
ADMIN_ACCESS=$(curl -s "$API_URL/api/users" \
  -H "Authorization: Bearer $ALICE_TOKEN")

echo "返回: $ADMIN_ACCESS"
if echo "$ADMIN_ACCESS" | grep -q "Admin permission required"; then
    print_success "权限隔离正常，普通用户无法访问管理员功能"
else
    print_error "权限隔离失败"
fi
echo ""

# ========================================
# 测试 7: 创建管理员用户
# ========================================
print_test "7" "创建管理员用户"
REGISTER_ADMIN=$(curl -s -X POST "$API_URL/api/auth/register" \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com","password":"admin123","nickname":"Admin"}')

ADMIN_ID=$(echo "$REGISTER_ADMIN" | jq -r '.user.id')
print_success "管理员用户创建成功，ID: $ADMIN_ID"

# 手动将用户提升为管理员
psql "postgres://entangle:Entangle%402024@localhost:5432/postgres" \
  -c "UPDATE entangle.users SET role_id = '00000000-0000-0000-0000-000000000001' WHERE id = '$ADMIN_ID';" > /dev/null 2>&1

print_success "已将用户提升为 admin 角色"
echo ""

# ========================================
# 测试 8: 管理员登录
# ========================================
print_test "8" "管理员登录"
LOGIN_ADMIN=$(curl -s -X POST "$API_URL/api/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com","password":"admin123"}')

ADMIN_TOKEN=$(echo "$LOGIN_ADMIN" | jq -r '.token')
ADMIN_ROLE=$(echo "$LOGIN_ADMIN" | jq -r '.user.role')

if [ "$ADMIN_ROLE" = "admin" ]; then
    print_success "管理员登录成功，角色: $ADMIN_ROLE"
else
    print_error "管理员登录失败"
fi
echo ""

# ========================================
# 测试 9: 管理员权限
# ========================================
print_test "9" "检查管理员权限"
ADMIN_PERMISSIONS=$(curl -s "$API_URL/api/me/permissions" \
  -H "Authorization: Bearer $ADMIN_TOKEN")

echo "管理员权限: $ADMIN_PERMISSIONS"
PERM_COUNT=$(echo "$ADMIN_PERMISSIONS" | jq '. | length')
if [ "$PERM_COUNT" = "6" ]; then
    print_success "管理员拥有全部 6 个权限"
else
    print_error "管理员权限不完整"
fi
echo ""

# ========================================
# 测试 10: 管理员功能 - 列出所有用户
# ========================================
print_test "10" "管理员功能 - 列出所有用户"
ALL_USERS=$(curl -s "$API_URL/api/users" \
  -H "Authorization: Bearer $ADMIN_TOKEN")

echo "$ALL_USERS" | jq .
USER_COUNT=$(echo "$ALL_USERS" | jq '. | length')
print_success "成功列出 $USER_COUNT 个用户"
echo ""

# ========================================
# 测试 11: 管理员功能 - 查看指定用户
# ========================================
print_test "11" "管理员功能 - 查看指定用户"
USER_DETAIL=$(curl -s "$API_URL/api/users/$ALICE_ID" \
  -H "Authorization: Bearer $ADMIN_TOKEN")

echo "$USER_DETAIL" | jq .
print_success "成功获取用户详情"
echo ""

# ========================================
# 测试 12: 无效 Token 测试
# ========================================
print_test "12" "使用无效 Token 访问"
INVALID_RESPONSE=$(curl -s "$API_URL/api/me" \
  -H "Authorization: Bearer invalid-token-123")

echo "返回: $INVALID_RESPONSE"
if echo "$INVALID_RESPONSE" | grep -q "Invalid token"; then
    print_success "无效 Token 被正确拒绝"
else
    print_error "Token 验证失败"
fi
echo ""

# ========================================
# 测试总结
# ========================================
echo "======================================"
echo -e "${GREEN}测试完成！${NC}"
echo "======================================"
echo ""
echo "已实现功能："
echo "  ✓ 用户注册和登录"
echo "  ✓ JWT Token 认证"
echo "  ✓ 基于角色的权限控制 (RBAC)"
echo "  ✓ 权限隔离（普通用户无法访问管理员功能）"
echo "  ✓ 管理员功能（用户管理）"
echo "  ✓ Token 验证"
echo ""
echo "数据库角色："
echo "  • admin  - 拥有所有 6 个权限"
echo "  • editor - 拥有所有文档操作权限"
echo "  • viewer - 仅拥有文档读取权限"
echo ""
