# 简化构建：直接使用编译好的二进制文件
FROM debian:12-slim

# 安装必要的运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 创建非root用户
RUN useradd -r -s /bin/false -m mockuser

WORKDIR /app

# 直接复制已编译的二进制文件
COPY target/release/llm-mock-server .

# 设置文件权限
RUN chown mockuser:mockuser /app/llm-mock-server && \
    chmod +x /app/llm-mock-server

# 切换到非root用户
USER mockuser

# 暴露端口
EXPOSE 8080

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# 启动服务
CMD ["./llm-mock-server"]