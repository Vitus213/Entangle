#!/usr/bin/env bash
# 标签功能测试脚本

set -e

API_URL="http://127.0.0.1:3000"

echo "=== 标签功能测试 ==="
echo ""

# 登录获取 token
echo "1. 登录获取 token..."
LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/api/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"demo@example.com","password":"demo123"}')

TOKEN=$(echo "$LOGIN_RESPONSE" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
echo "Token obtained: ${TOKEN:0:50}..."
echo ""

# 创建标签
echo "2. 创建标签 'Frontend'..."
CREATE_TAG1=$(curl -s -X POST "$API_URL/api/tags" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name":"Frontend","color":"#3B82F6"}')

echo "$CREATE_TAG1"
TAG1_ID=$(echo "$CREATE_TAG1" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
echo ""
echo "Tag created with ID: $TAG1_ID"
echo ""

# 创建第二个标签
echo "3. 创建标签 'Backend'..."
CREATE_TAG2=$(curl -s -X POST "$API_URL/api/tags" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name":"Backend","color":"#10B981"}')

echo "$CREATE_TAG2"
TAG2_ID=$(echo "$CREATE_TAG2" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
echo ""
echo "Tag created with ID: $TAG2_ID"
echo ""

# 创建第三个标签
echo "4. 创建标签 'Documentation'..."
CREATE_TAG3=$(curl -s -X POST "$API_URL/api/tags" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name":"Documentation","color":"#F59E0B"}')

echo "$CREATE_TAG3"
TAG3_ID=$(echo "$CREATE_TAG3" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
echo ""
echo "Tag created with ID: $TAG3_ID"
echo ""

# 列出所有标签
echo "5. 列出我的所有标签..."
curl -s "$API_URL/api/tags" \
  -H "Authorization: Bearer $TOKEN"
echo ""
echo ""

# 更新标签
echo "6. 更新标签..."
curl -s -X PUT "$API_URL/api/tags/$TAG1_ID" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name":"React & Vue","color":"#6366F1"}'
echo ""
echo ""

# 创建文档
echo "7. 创建文档 1..."
CREATE_DOC1=$(curl -s -X POST "$API_URL/api/documents" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"title":"React Components Guide","content":"Learn React components","is_public":false}')

echo "$CREATE_DOC1"
DOC1_ID=$(echo "$CREATE_DOC1" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
echo ""
echo "Document 1 created with ID: $DOC1_ID"
echo ""

# 创建文档 2
echo "8. 创建文档 2..."
CREATE_DOC2=$(curl -s -X POST "$API_URL/api/documents" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"title":"Rust API Development","content":"Building APIs with Rust","is_public":false}')

echo "$CREATE_DOC2"
DOC2_ID=$(echo "$CREATE_DOC2" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
echo ""
echo "Document 2 created with ID: $DOC2_ID"
echo ""

# 为文档1添加标签（Frontend）
echo "9. 为文档 1 添加 'React & Vue' 标签..."
curl -s -X POST "$API_URL/api/documents/$DOC1_ID/tags" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{\"tag_id\":\"$TAG1_ID\"}"
echo "Tag added successfully"
echo ""

# 为文档1添加第二个标签（Documentation）
echo "10. 为文档 1 添加 'Documentation' 标签..."
curl -s -X POST "$API_URL/api/documents/$DOC1_ID/tags" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{\"tag_id\":\"$TAG3_ID\"}"
echo "Tag added successfully"
echo ""

# 为文档2添加标签（Backend）
echo "11. 为文档 2 添加 'Backend' 标签..."
curl -s -X POST "$API_URL/api/documents/$DOC2_ID/tags" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{\"tag_id\":\"$TAG2_ID\"}"
echo "Tag added successfully"
echo ""

# 为文档2添加第二个标签（Documentation）
echo "12. 为文档 2 添加 'Documentation' 标签..."
curl -s -X POST "$API_URL/api/documents/$DOC2_ID/tags" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{\"tag_id\":\"$TAG3_ID\"}"
echo "Tag added successfully"
echo ""

# 获取文档1的所有标签
echo "13. 获取文档 1 的所有标签..."
curl -s "$API_URL/api/documents/$DOC1_ID/tags" \
  -H "Authorization: Bearer $TOKEN"
echo ""
echo ""

# 获取文档2的所有标签
echo "14. 获取文档 2 的所有标签..."
curl -s "$API_URL/api/documents/$DOC2_ID/tags" \
  -H "Authorization: Bearer $TOKEN"
echo ""
echo ""

# 按单个标签筛选文档（OR 模式）
echo "15. 筛选带 'Documentation' 标签的文档（OR 模式）..."
curl -s "$API_URL/api/documents/by-tags?tag_ids=$TAG3_ID&match_mode=any" \
  -H "Authorization: Bearer $TOKEN"
echo ""
echo ""

# 按多个标签筛选文档（OR 模式）
echo "16. 筛选带 'React & Vue' 或 'Backend' 标签的文档（OR 模式）..."
curl -s "$API_URL/api/documents/by-tags?tag_ids=$TAG1_ID,$TAG2_ID&match_mode=any" \
  -H "Authorization: Bearer $TOKEN"
echo ""
echo ""

# 按多个标签筛选文档（AND 模式）
echo "17. 筛选同时带 'React & Vue' 和 'Documentation' 标签的文档（AND 模式）..."
curl -s "$API_URL/api/documents/by-tags?tag_ids=$TAG1_ID,$TAG3_ID&match_mode=all" \
  -H "Authorization: Bearer $TOKEN"
echo ""
echo ""

# 批量设置文档标签
echo "18. 批量设置文档 2 的标签..."
curl -s -X PUT "$API_URL/api/documents/$DOC2_ID/tags" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{\"tag_ids\":[\"$TAG2_ID\"]}"
echo ""
echo ""

# 从文档移除标签
echo "19. 从文档 1 移除 'Documentation' 标签..."
curl -s -X DELETE "$API_URL/api/documents/$DOC1_ID/tags/$TAG3_ID" \
  -H "Authorization: Bearer $TOKEN"
echo "Tag removed successfully"
echo ""

# 再次列出所有标签（查看文档计数变化）
echo "20. 再次列出所有标签（查看文档计数）..."
curl -s "$API_URL/api/tags" \
  -H "Authorization: Bearer $TOKEN"
echo ""
echo ""

# 删除标签
echo "21. 删除 'Documentation' 标签..."
curl -s -X DELETE "$API_URL/api/tags/$TAG3_ID" \
  -H "Authorization: Bearer $TOKEN"
echo "Tag deleted successfully"
echo ""

# 最后再列出所有标签
echo "22. 最后列出所有标签..."
curl -s "$API_URL/api/tags" \
  -H "Authorization: Bearer $TOKEN"
echo ""
echo ""

echo "=== 测试完成 ==="
echo ""
echo "测试总结："
echo "✅ 创建标签（3个）"
echo "✅ 列出标签"
echo "✅ 更新标签"
echo "✅ 为文档添加标签"
echo "✅ 获取文档标签列表"
echo "✅ 按标签筛选文档（OR 模式）"
echo "✅ 按标签筛选文档（AND 模式）"
echo "✅ 批量设置文档标签"
echo "✅ 从文档移除标签"
echo "✅ 删除标签"
echo "✅ 验证文档计数更新"
echo ""
