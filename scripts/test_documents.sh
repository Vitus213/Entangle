#!/usr/bin/env bash
# 文档功能测试脚本

set -e

API_URL="http://127.0.0.1:3000"

echo "=== 文档功能测试 ==="
echo ""

# 登录获取 token
echo "1. 登录获取 token..."
LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/api/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"demo@example.com","password":"demo123"}')

TOKEN=$(echo "$LOGIN_RESPONSE" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
echo "Token obtained: ${TOKEN:0:50}..."
echo ""

# 创建文档
echo "2. 创建文档..."
CREATE_DOC=$(curl -s -X POST "$API_URL/api/documents" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"title":"My First Document","content":"Hello World!","is_public":false}')

echo "$CREATE_DOC"
DOC_ID=$(echo "$CREATE_DOC" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
echo ""
echo "Document created with ID: $DOC_ID"
echo ""

# 获取文档详情
echo "3. 获取文档详情..."
curl -s "$API_URL/api/documents/$DOC_ID" \
  -H "Authorization: Bearer $TOKEN"
echo ""
echo ""

# 更新文档
echo "4. 更新文档..."
curl -s -X PUT "$API_URL/api/documents/$DOC_ID" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"title":"Updated Title","content":"Updated content!"}'
echo ""
echo ""

# 列出我的文档
echo "5. 列出我的文档..."
curl -s "$API_URL/api/documents/my" \
  -H "Authorization: Bearer $TOKEN"
echo ""
echo ""

# 创建公开文档
echo "6. 创建公开文档..."
curl -s -X POST "$API_URL/api/documents" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"title":"Public Document","content":"This is public","is_public":true}'
echo ""
echo ""

# 列出公开文档
echo "7. 列出公开文档..."
curl -s "$API_URL/api/documents/public"
echo ""
echo ""

# 删除文档
echo "8. 删除文档..."
curl -s -X DELETE "$API_URL/api/documents/$DOC_ID" \
  -H "Authorization: Bearer $TOKEN"
echo "文档已删除"
echo ""

echo "=== 测试完成 ==="
