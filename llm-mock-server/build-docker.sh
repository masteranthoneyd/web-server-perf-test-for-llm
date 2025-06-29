#!/bin/bash

set -e

echo "🐳 LLM Mock Server Docker 构建脚本"
echo "================================="

# 版本信息
VERSION=${1:-"latest"}
IMAGE_NAME="llm-mock-server"

echo "📋 构建信息:"
echo "  镜像名称: $IMAGE_NAME"
echo "  版本标签: $VERSION"
echo ""

# 第1步：编译Rust应用
echo "🔨 第1步: 编译Rust应用..."
cargo build --release

# 第2步：构建Docker镜像
echo ""
echo "🐳 第2步: 构建Docker镜像..."
echo "  使用Dockerfile.simple进行快速构建..."

docker build -f Dockerfile -t "$IMAGE_NAME:$VERSION" .

if [ "$VERSION" != "latest" ]; then
    docker tag "$IMAGE_NAME:$VERSION" "$IMAGE_NAME:latest"
    echo "  ✅ 已创建 latest 标签"
fi