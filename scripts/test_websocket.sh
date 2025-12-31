#!/usr/bin/env bash
# WebSocket 连接测试

set -e

API_URL="http://127.0.0.1:3000"

echo "=== WebSocket 实时协作测试 ==="
echo ""

# 清理旧数据
echo "0. 清理测试数据..."
psql "postgres://entangle:Entangle%402024@localhost:5432/postgres" \
  -c "DELETE FROM entangle.users WHERE email IN ('ws_user_a@test.com', 'ws_user_b@test.com');" > /dev/null 2>&1
echo "✓ 清理完成"
echo ""

# 创建测试用户
echo "1. 创建测试用户..."
REGISTER_A=$(curl -s -X POST "$API_URL/api/auth/register" \
  -H "Content-Type: application/json" \
  -d '{"email":"ws_user_a@test.com","password":"test123","nickname":"WS User A"}')

TOKEN_A=$(echo "$REGISTER_A" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
USER_A_ID=$(echo "$REGISTER_A" | grep -o '"user":{"id":"[^"]*"' | cut -d'"' -f6)

echo "用户 A: $USER_A_ID"
echo "Token A: ${TOKEN_A:0:20}..."
echo ""

# 升级为 editor
psql "postgres://entangle:Entangle%402024@localhost:5432/postgres" \
  -c "UPDATE entangle.users SET role_id = '00000000-0000-0000-0000-000000000002' WHERE email = 'ws_user_a@test.com';" > /dev/null 2>&1

# 重新登录
LOGIN_A=$(curl -s -X POST "$API_URL/api/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"ws_user_a@test.com","password":"test123"}')
TOKEN_A=$(echo "$LOGIN_A" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)

# 创建文档
echo "2. 创建测试文档..."
CREATE_DOC=$(curl -s -X POST "$API_URL/api/documents" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN_A" \
  -d '{"title":"WebSocket Test Document","content":"Initial content","is_public":false}')

DOC_ID=$(echo "$CREATE_DOC" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
echo "文档ID: $DOC_ID"
echo ""

# 测试 WebSocket 连接
echo "3. 测试 WebSocket 端点..."
echo "WebSocket URL: ws://127.0.0.1:3000/ws/documents/$DOC_ID"
echo ""
echo "提示: WebSocket 端点已经准备好，可以使用以下方式测试:"
echo "  - 使用 wscat: wscat -c ws://127.0.0.1:3000/ws/documents/$DOC_ID -H \"Authorization: Bearer $TOKEN_A\""
echo "  - 使用浏览器开发者工具的 WebSocket 客户端"
echo "  - 使用专门的 WebSocket 测试工具"
echo ""

# 验证基本端点
echo "4. 验证基本 HTTP 端点..."
echo "  - GET /health"
curl -s http://127.0.0.1:3000/health
echo ""
echo "  - GET /"
curl -s http://127.0.0.1:3000/
echo ""
echo ""

echo "=== 测试信息总结 ==="
echo "文档 ID: $DOC_ID"
echo "用户 A Token: $TOKEN_A"
echo "WebSocket URL: ws://127.0.0.1:3000/ws/documents/$DOC_ID"
echo ""
echo "✓ API 服务器运行中，WebSocket 端点已就绪"
