#!/bin/bash
# 前端构建和运行脚本

set -e

cd "$(dirname "$0")/.."

echo "正在构建前端..."
cd frontend

# 使用 build-std 编译 WASM
RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+mutable-globals" \
    cargo build --target wasm32-unknown-unknown -Z build-std=std,panic_abort --release

echo "构建完成！"
echo ""
echo "WASM 文件位置:"
echo "  target/wasm32-unknown-unknown/release/entangle_frontend.wasm"
echo ""
echo "接下来需要："
echo "1. 使用 wasm-bindgen 生成 JavaScript 绑定"
echo "2. 使用 HTTP 服务器提供静态文件"
echo ""
echo "或者使用 trunk serve 来自动完成这些步骤"
