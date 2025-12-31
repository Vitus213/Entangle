#!/usr/bin/env bash
# API 测试脚本

set -e

API_URL="http://127.0.0.1:3000"

echo "=== Entangle API 测试脚本 ==="
echo ""

# 测试健康检查
echo "1. 测试健康检查..."
curl -s "$API_URL/health"
echo ""
echo ""

# 注册用户
echo "2. 注册新用户..."
REGISTER_RESPONSE=$(curl -s -X POST "$API_URL/api/auth/register" \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"test123","nickname":"测试用户"}')
echo "$REGISTER_RESPONSE" | jq .
TOKEN=$(echo "$REGISTER_RESPONSE" | jq -r '.token')
echo ""

# 登录
echo "3. 登录用户..."
LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/api/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"test123"}')
echo "$LOGIN_RESPONSE" | jq .
TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.token')
echo ""

# 获取当前用户信息
echo "4. 获取当前用户信息..."
curl -s "$API_URL/api/me" \
  -H "Authorization: Bearer $TOKEN" | jq .
echo ""

# 获取用户权限
echo "5. 获取用户权限..."
curl -s "$API_URL/api/me/permissions" \
  -H "Authorization: Bearer $TOKEN" | jq .
echo ""

# 测试访问管理员功能（应该失败）
echo "6. 测试普通用户访问管理员功能（应该被拒绝）..."
curl -s "$API_URL/api/users" \
  -H "Authorization: Bearer $TOKEN" | jq .
echo ""

echo "=== 测试完成 ==="
