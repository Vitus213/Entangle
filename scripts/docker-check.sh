#!/usr/bin/env bash
# Docker 诊断脚本

echo "🔍 Docker 环境诊断"
echo "===================="
echo ""

# 1. 检查 Docker 服务
echo "1️⃣ 检查 Docker 服务状态..."
if systemctl is-active --quiet docker; then
    echo "✅ Docker 服务正在运行"
else
    echo "❌ Docker 服务未运行"
    echo "   执行: sudo systemctl start docker"
fi
echo ""

# 2. 检查 Docker 版本
echo "2️⃣ Docker 版本信息..."
docker --version
docker-compose --version
echo ""

# 3. 检查网络连接
echo "3️⃣ 检查网络连接..."
if curl -s -m 5 https://registry-1.docker.io/v2/ > /dev/null; then
    echo "✅ 可以连接到 Docker Hub"
else
    echo "❌ 无法连接到 Docker Hub"
    echo "   建议配置镜像加速器"
fi
echo ""

# 4. 检查镜像加速器配置
echo "4️⃣ 检查镜像加速器配置..."
if [ -f /etc/docker/daemon.json ]; then
    echo "✅ 配置文件存在:"
    cat /etc/docker/daemon.json
else
    echo "⚠️  未配置镜像加速器"
    echo "   建议配置国内镜像源"
fi
echo ""

# 5. 测试拉取镜像
echo "5️⃣ 测试拉取小镜像..."
if docker pull alpine:latest > /dev/null 2>&1; then
    echo "✅ 镜像拉取正常"
    docker rmi alpine:latest > /dev/null 2>&1
else
    echo "❌ 镜像拉取失败"
fi
echo ""

# 6. 检查已有镜像
echo "6️⃣ 检查项目所需镜像..."
echo "Redis:"
if docker images | grep -q "redis.*7-alpine"; then
    echo "  ✅ redis:7-alpine 已存在"
else
    echo "  ❌ redis:7-alpine 不存在"
fi

echo "openGauss:"
if docker images | grep -q "opengauss.*5.0.0"; then
    echo "  ✅ opengauss:5.0.0 已存在"
else
    echo "  ❌ opengauss:5.0.0 不存在"
fi
echo ""

echo "📋 建议操作："
echo "1. 配置镜像加速器: sudo vim /etc/docker/daemon.json"
echo "2. 重启 Docker: sudo systemctl restart docker"
echo "3. 拉取镜像: docker pull redis:7-alpine"
echo "4. 运行项目: just dev"
