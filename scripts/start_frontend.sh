#!/bin/bash
# 启动前端开发服务器
# 用法: ./scripts/start_frontend.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT/frontend"

echo "🚀 启动前端开发服务器..."
echo "   端口: 8080"
echo "   地址: http://localhost:8080"
echo ""

# 检查 trunk 是否安装
if ! command -v trunk &> /dev/null; then
    echo "❌ 错误: trunk 未安装"
    echo "   请运行: cargo install trunk"
    exit 1
fi

# 检查 WASM target 是否安装
if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo "⚠️  安装 WASM target..."
    rustup target add wasm32-unknown-unknown
fi

# 启动 trunk 开发服务器
trunk serve --open false
