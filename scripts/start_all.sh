#!/bin/bash
# 启动所有服务 (后端 + 前端)
# 用法: ./scripts/start_all.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo "=========================================="
echo "   Entangle 开发环境启动脚本"
echo "=========================================="
echo ""

# 检查依赖
echo "📋 检查依赖..."

if ! command -v cargo &> /dev/null; then
    echo "❌ 错误: Rust 未安装"
    exit 1
fi

if ! command -v trunk &> /dev/null; then
    echo "⚠️  trunk 未安装，正在安装..."
    cargo install trunk
fi

if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo "⚠️  安装 WASM target..."
    rustup target add wasm32-unknown-unknown
fi

echo "✓ 依赖检查完成"
echo ""

# 启动后端
echo "🔧 启动后端 API (端口 3000)..."
export RUST_LOG=${RUST_LOG:-info,entangle_api=debug}
cargo run --bin entangle-api &
BACKEND_PID=$!
echo "   后端 PID: $BACKEND_PID"

# 等待后端启动
sleep 3

# 检查后端是否启动成功
if ! kill -0 $BACKEND_PID 2>/dev/null; then
    echo "❌ 后端启动失败"
    exit 1
fi

# 启动前端
echo "🎨 启动前端开发服务器 (端口 8080)..."
cd frontend
trunk serve --open false &
FRONTEND_PID=$!
echo "   前端 PID: $FRONTEND_PID"
cd ..

# 等待前端启动
sleep 3

echo ""
echo "=========================================="
echo "   ✓ 所有服务已启动"
echo "=========================================="
echo ""
echo "   前端:    http://localhost:8080"
echo "   后端:    http://localhost:3000"
echo "   API:     http://localhost:3000/api"
echo ""
echo "   按 Ctrl+C 停止所有服务"
echo ""

# 捕获退出信号
cleanup() {
    echo ""
    echo "🛑 停止服务..."
    kill $BACKEND_PID 2>/dev/null || true
    kill $FRONTEND_PID 2>/dev/null || true
    echo "✓ 服务已停止"
    exit 0
}

trap cleanup SIGINT SIGTERM

# 等待进程
wait
