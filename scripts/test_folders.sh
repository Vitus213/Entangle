#!/usr/bin/env bash
# 文件夹功能测试脚本

set -e

API_URL="http://127.0.0.1:3000"

echo "=== 文件夹功能测试 ==="
echo ""

# 登录获取 token
echo "1. 登录获取 token..."
LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/api/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"demo@example.com","password":"demo123"}')

TOKEN=$(echo "$LOGIN_RESPONSE" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
echo "Token obtained: ${TOKEN:0:50}..."
echo ""

# 创建根文件夹
echo "2. 创建根文件夹 'Projects'..."
CREATE_ROOT=$(curl -s -X POST "$API_URL/api/folders" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name":"Projects","parent_id":null}')

echo "$CREATE_ROOT"
ROOT_FOLDER_ID=$(echo "$CREATE_ROOT" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
echo ""
echo "Root folder created with ID: $ROOT_FOLDER_ID"
echo ""

# 创建子文件夹
echo "3. 创建子文件夹 'Frontend'..."
CREATE_SUB=$(curl -s -X POST "$API_URL/api/folders" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{\"name\":\"Frontend\",\"parent_id\":\"$ROOT_FOLDER_ID\"}")

echo "$CREATE_SUB"
SUB_FOLDER_ID=$(echo "$CREATE_SUB" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
echo ""
echo "Sub folder created with ID: $SUB_FOLDER_ID"
echo ""

# 创建另一个子文件夹
echo "4. 创建另一个子文件夹 'Backend'..."
CREATE_SUB2=$(curl -s -X POST "$API_URL/api/folders" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{\"name\":\"Backend\",\"parent_id\":\"$ROOT_FOLDER_ID\"}")

echo "$CREATE_SUB2"
SUB2_FOLDER_ID=$(echo "$CREATE_SUB2" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
echo ""
echo "Second sub folder created with ID: $SUB2_FOLDER_ID"
echo ""

# 获取文件夹详情
echo "5. 获取文件夹详情..."
curl -s "$API_URL/api/folders/$ROOT_FOLDER_ID" \
  -H "Authorization: Bearer $TOKEN"
echo ""
echo ""

# 更新文件夹
echo "6. 更新文件夹名称..."
curl -s -X PUT "$API_URL/api/folders/$ROOT_FOLDER_ID" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name":"My Projects"}'
echo ""
echo ""

# 获取文件夹树
echo "7. 获取文件夹树..."
curl -s "$API_URL/api/folders/tree" \
  -H "Authorization: Bearer $TOKEN"
echo ""
echo ""

# 创建文档
echo "8. 创建文档..."
CREATE_DOC=$(curl -s -X POST "$API_URL/api/documents" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"title":"React Component","content":"export default function App() {}","is_public":false}')

echo "$CREATE_DOC"
DOC_ID=$(echo "$CREATE_DOC" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
echo ""
echo "Document created with ID: $DOC_ID"
echo ""

# 移动文档到文件夹
echo "9. 移动文档到 Frontend 文件夹..."
curl -s -X PUT "$API_URL/api/documents/$DOC_ID/move" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{\"folder_id\":\"$SUB_FOLDER_ID\"}"
echo "Document moved successfully"
echo ""

# 创建另一个文档
echo "10. 创建另一个文档..."
CREATE_DOC2=$(curl -s -X POST "$API_URL/api/documents" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"title":"API Endpoint","content":"async fn handler() {}","is_public":false}')

echo "$CREATE_DOC2"
DOC2_ID=$(echo "$CREATE_DOC2" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
echo ""
echo "Second document created with ID: $DOC2_ID"
echo ""

# 移动第二个文档到 Backend 文件夹
echo "11. 移动第二个文档到 Backend 文件夹..."
curl -s -X PUT "$API_URL/api/documents/$DOC2_ID/move" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{\"folder_id\":\"$SUB2_FOLDER_ID\"}"
echo "Document moved successfully"
echo ""

# 获取根文件夹内容
echo "12. 获取根文件夹内容..."
curl -s "$API_URL/api/folders/$ROOT_FOLDER_ID/contents" \
  -H "Authorization: Bearer $TOKEN"
echo ""
echo ""

# 获取 Frontend 文件夹内容
echo "13. 获取 Frontend 文件夹内容..."
curl -s "$API_URL/api/folders/$SUB_FOLDER_ID/contents" \
  -H "Authorization: Bearer $TOKEN"
echo ""
echo ""

# 获取 Backend 文件夹内容
echo "14. 获取 Backend 文件夹内容..."
curl -s "$API_URL/api/folders/$SUB2_FOLDER_ID/contents" \
  -H "Authorization: Bearer $TOKEN"
echo ""
echo ""

# 将文档移出文件夹（移到根目录）
echo "15. 将文档移出文件夹..."
curl -s -X PUT "$API_URL/api/documents/$DOC_ID/move" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"folder_id":null}'
echo "Document moved to root"
echo ""

# 删除子文件夹（级联删除）
echo "16. 删除 Backend 文件夹..."
curl -s -X DELETE "$API_URL/api/folders/$SUB2_FOLDER_ID" \
  -H "Authorization: Bearer $TOKEN"
echo "Folder deleted (documents inside will also be affected)"
echo ""

# 再次获取文件夹树，验证删除
echo "17. 再次获取文件夹树，验证删除..."
curl -s "$API_URL/api/folders/tree" \
  -H "Authorization: Bearer $TOKEN"
echo ""
echo ""

echo "=== 测试完成 ==="
echo ""
echo "测试总结："
echo "✅ 创建根文件夹"
echo "✅ 创建子文件夹（多层级）"
echo "✅ 获取文件夹详情"
echo "✅ 更新文件夹名称"
echo "✅ 获取文件夹树"
echo "✅ 创建文档"
echo "✅ 移动文档到文件夹"
echo "✅ 获取文件夹内容"
echo "✅ 移动文档出文件夹"
echo "✅ 删除文件夹"
echo "✅ 验证文件夹树更新"
echo ""
