#!/bin/bash
# 启动后端 API 服务器
# 用法: ./scripts/start_backend.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo "🚀 启动后端 API..."
echo "   端口: 3000"
echo "   日志级别: info"
echo ""

# 设置环境变量
export RUST_LOG=${RUST_LOG:-info,entangle_api=debug}

# 检查 .env 文件
if [ ! -f .env ]; then
    echo "⚠️  警告: .env 文件不存在"
    echo "   请创建 .env 文件并配置 DATABASE_URL"
    exit 1
fi

# 启动服务
cargo run --bin entangle-api
