#!/usr/bin/env bash
# 文档协作功能测试

set -e

API_URL="http://127.0.0.1:3000"

echo "=== 文档协作功能测试 ==="
echo ""

# 清理旧的测试用户（如果存在）
echo "0. 清理旧的测试数据..."
psql "postgres://entangle:Entangle%402024@localhost:5432/postgres" \
  -c "DELETE FROM entangle.users WHERE email IN ('owner@test.com', 'collab@test.com');" > /dev/null 2>&1
echo "✓ 清理完成"
echo ""

# 创建两个用户
echo "1. 创建测试用户..."
# 用户 A (owner)
REGISTER_A=$(curl -s -X POST "$API_URL/api/auth/register" \
  -H "Content-Type: application/json" \
  -d '{"email":"owner@test.com","password":"test123","nickname":"Owner"}')
echo "Register A response: $REGISTER_A" >&2
TOKEN_A=$(echo "$REGISTER_A" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
USER_A_ID=$(echo "$REGISTER_A" | grep -o '"user":{"id":"[^"]*"' | cut -d'"' -f6)

# 升级为 editor
psql "postgres://entangle:Entangle%402024@localhost:5432/postgres" \
  -c "UPDATE entangle.users SET role_id = '00000000-0000-0000-0000-000000000002' WHERE email = 'owner@test.com';" > /dev/null 2>&1

# 用户 B (collaborator)
REGISTER_B=$(curl -s -X POST "$API_URL/api/auth/register" \
  -H "Content-Type: application/json" \
  -d '{"email":"collab@test.com","password":"test123","nickname":"Collaborator"}')
echo "Register B response: $REGISTER_B" >&2
TOKEN_B=$(echo "$REGISTER_B" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
USER_B_ID=$(echo "$REGISTER_B" | grep -o '"user":{"id":"[^"]*"' | cut -d'"' -f6)

echo "用户 A (Owner): $USER_A_ID"
echo "用户 B (Collaborator): $USER_B_ID"
echo ""

# 用户 A 重新登录以获取新角色的 token
LOGIN_A=$(curl -s -X POST "$API_URL/api/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"owner@test.com","password":"test123"}')
TOKEN_A=$(echo "$LOGIN_A" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)

# 2. 用户 A 创建文档
echo "2. 用户 A 创建私有文档..."
CREATE_DOC=$(curl -s -X POST "$API_URL/api/documents" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN_A" \
  -d '{"title":"Collaborative Document","content":"Original content","is_public":false}')

DOC_ID=$(echo "$CREATE_DOC" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
echo "文档创建成功: $DOC_ID"
echo ""

# 3. 用户 B 尝试访问（应该失败）
echo "3. 用户 B 尝试访问私有文档（应该失败）..."
RESULT=$(curl -s "$API_URL/api/documents/$DOC_ID" \
  -H "Authorization: Bearer $TOKEN_B")
echo "$RESULT"
echo ""

# 4. 用户 A 添加用户 B 为协作者（写权限）
echo "4. 用户 A 添加用户 B 为协作者（写权限）..."
curl -s -X POST "$API_URL/api/documents/$DOC_ID/collaborators" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN_A" \
  -d "{\"user_id\":\"$USER_B_ID\",\"permission\":\"write\"}"
echo "协作者添加成功"
echo ""

# 5. 用户 B 再次尝试访问（应该成功）
echo "5. 用户 B 再次访问文档（应该成功）..."
RESULT=$(curl -s "$API_URL/api/documents/$DOC_ID" \
  -H "Authorization: Bearer $TOKEN_B")
echo "$RESULT" | grep -q "Collaborative Document" && echo "✓ 用户 B 可以访问文档" || echo "✗ 访问失败"
echo ""

# 6. 用户 B 编辑文档
echo "6. 用户 B 编辑文档..."
curl -s -X PUT "$API_URL/api/documents/$DOC_ID" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN_B" \
  -d '{"content":"Modified by collaborator"}' > /dev/null
echo "✓ 用户 B 成功编辑文档"
echo ""

# 7. 用户 B 尝试删除文档（应该失败，因为不是 owner）
echo "7. 用户 B 尝试删除文档（应该失败）..."
RESULT=$(curl -s -X DELETE "$API_URL/api/documents/$DOC_ID" \
  -H "Authorization: Bearer $TOKEN_B")
echo "$RESULT"
echo ""

# 8. 列出用户 B 可访问的文档
echo "8. 列出用户 B 可访问的文档..."
curl -s "$API_URL/api/documents/accessible" \
  -H "Authorization: Bearer $TOKEN_B" | grep -q "$DOC_ID" && echo "✓ 协作文档在可访问列表中" || echo "✗ 未找到"
echo ""

# 9. 用户 A 移除协作者
echo "9. 用户 A 移除协作者..."
curl -s -X DELETE "$API_URL/api/documents/$DOC_ID/collaborators/$USER_B_ID" \
  -H "Authorization: Bearer $TOKEN_A"
echo "✓ 协作者已移除"
echo ""

# 10. 用户 B 再次尝试访问（应该失败）
echo "10. 用户 B 再次尝试访问（应该失败）..."
RESULT=$(curl -s "$API_URL/api/documents/$DOC_ID" \
  -H "Authorization: Bearer $TOKEN_B")
echo "$RESULT"
echo ""

# 清理
echo "11. 清理测试数据..."
curl -s -X DELETE "$API_URL/api/documents/$DOC_ID" \
  -H "Authorization: Bearer $TOKEN_A" > /dev/null
echo "✓ 测试完成"
echo ""

echo "=== 协作功能测试完成 ==="
