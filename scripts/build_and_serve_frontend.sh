#!/usr/bin/env bash
# 前端构建和服务脚本（使用 Trunk）

set -e

cd "$(dirname "$0")/.."

echo "🔨 正在构建 Leptos 前端..."
cd frontend

echo "📦 使用 Trunk 构建 WASM 应用..."
trunk build --release

echo ""
echo "✅ 构建完成！"
echo ""
echo "构建文件位置: frontend/dist/"
echo ""
echo "📂 构建产物:"
ls -lh dist/
echo ""
echo "启动选项："
echo "  1. 开发模式（热重载）: trunk serve"
echo "  2. 生产模式: 使用任意 HTTP 服务器提供 dist/ 目录"
echo ""
echo "示例 - 使用 Python 简易服务器:"
echo "  cd frontend/dist && python3 -m http.server 8080"
echo ""
