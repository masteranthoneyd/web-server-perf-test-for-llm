# LLM Mock Server - 高并发模拟服务

一个用Rust编写的高性能LLM慢响应模拟服务，专门用于压测LLM业务系统，支持在1核1G和2核2G的资源限制下实现高并发处理。

## 🆕 最新功能
- **实时内存监控**：健康检查端点现在返回详细的系统内存信息
- **智能健康判断**：基于并发使用率和内存使用率的双重健康检查
- **完整监控支持**：支持并发状态、内存状态的实时监控

## 🚀 性能优化亮点

### 1. **智能并发控制**
- 使用Tokio信号量精确控制最大并发数
- 1核1G配置：支持1500+并发
- 2核2G配置：支持3000+并发
- 超出限制时返回503状态码，避免系统崩溃

### 2. **资源优化配置**
- **工作线程数智能调整**：1核2线程，2核3线程
- **栈大小优化**：512KB线程栈（相比默认2MB大幅减少）
- **内存预分配**：响应体和响应头使用静态分配，零运行时开销

### 3. **零成本抽象**
- 使用`static`变量预分配响应内容
- `Lazy`静态初始化响应头，避免重复创建
- 编译时优化，运行时近乎零开销

### 4. **完整监控支持**
- `/health` - 健康检查端点，显示并发使用率和系统内存状态
- `/metrics` - Prometheus格式指标
- 实时监控活跃请求数、可用槽位和内存使用情况
- 智能健康状态判断：并发使用率>90%或内存使用率>95%时返回不健康状态

### 5. **生产级特性**
- 优雅关闭：支持SIGTERM和SIGINT信号
- TCP优化：启用keepalive和nodelay
- 请求体大小限制：16KB防止内存攻击
- 非root用户运行：提高安全性

## 📊 性能对比

| 配置 | 并发数 | 内存使用 | 工作线程 | 适用场景 |
|------|--------|----------|----------|----------|
| 1核1G | 1500 | ~800MB | 2 | 高密度部署 |
| 2核2G | 3000 | ~1.5GB | 3 | 标准部署 |

对比传统Java + Spring Boot：
- **并发能力**：提升10-50倍
- **内存效率**：每连接仅需2-8KB（vs Java的1-8MB）
- **响应延迟**：纳秒级上下文切换（vs Java的微秒级）

## 🛠️ 快速开始

### 本地运行

```bash
# 编译运行
cargo run

# 自定义配置
MAX_CONCURRENT=2000 WORKER_THREADS=2 cargo run

# 编译
cargo build --release
```

### Docker部署

```bash
# 构建镜像
docker build -t llm-mock-server .

# 运行1核1G配置
docker compose up -d llm-mock-1c1g

# 运行2核2G配置  
docker compose up -d llm-mock-2c2g
```

## 🔧 配置参数

| 环境变量 | 默认值 | 说明 |
|----------|--------|------|
| `MAX_CONCURRENT` | 智能配置 | 最大并发请求数 |
| `WORKER_THREADS` | 智能配置 | Tokio工作线程数 |
| `RESPONSE_DELAY_SECONDS` | 10 | 响应延迟秒数 |
| `PORT` | 8080 | 监听端口 |

智能默认配置：
- 1核：1500并发，2工作线程
- 2核：3000并发，3工作线程

## 📋 API端点

### 主要API
```bash
POST /compatible-mode/v1/chat/completions
Content-Type: application/json

{
  "model": "test",
  "messages": [{"role": "user", "content": "Hello"}]
}
```

### 监控端点
```bash
# 健康检查
GET /health
# 返回: 
{
  "status": "healthy",
  "concurrent_usage": "5.2%",
  "available_slots": 2844,
  "memory": {
    "total_mb": 46965,
    "used_mb": 1860,
    "available_mb": 45105,
    "usage_percent": "4.0%"
  }
}

# 指标监控  
GET /metrics
# 返回Prometheus格式指标
```

## 🧪 压测示例

```bash
# 基础压测
curl -X POST http://localhost:8080/compatible-mode/v1/chat/completions \
     -H "Content-Type: application/json" \
     -d '{"model":"test","messages":[{"role":"user","content":"Hello"}]}'

# 并发压测（使用ab）
ab -n 1000 -c 100 -p post_data.json -T application/json \
   http://localhost:8080/compatible-mode/v1/chat/completions

# 监控并发使用率和内存状态
watch -n 1 'curl -s http://localhost:8080/health | jq .'

# 仅监控内存使用情况
curl -s http://localhost:8080/health | jq '.memory'

# 监控特定指标
curl -s http://localhost:8080/health | jq '.memory.usage_percent'
```

## 🏗️ 核心技术架构

### 异步并发模型
```
请求 → 信号量控制 → 异步处理 → 静态响应 → 返回
```

### 内存使用优化
- **静态分配**：响应内容编译时确定
- **零拷贝**：直接返回静态字符串引用
- **精确控制**：信号量确保内存使用上限

### 线程模型
```
固定工作线程池 ←→ 异步任务队列 ←→ 事件循环
```

## 📈 监控和调优

### 关键指标

#### 并发指标
- `llm_mock_active_requests`：当前活跃请求数
- `llm_mock_max_concurrent_requests`：最大并发限制
- 健康检查中的`concurrent_usage`：并发使用率

#### 内存指标 🆕
- `memory.total_mb`：系统总内存（MB）
- `memory.used_mb`：已使用内存（MB）
- `memory.available_mb`：可用内存（MB）
- `memory.usage_percent`：内存使用率百分比

#### 健康状态判断
服务会在以下情况返回`SERVICE_UNAVAILABLE`状态：
- 并发使用率 > 90% **或者**
- 系统内存使用率 > 95%

### 调优建议
1. **内存不足**：减少`MAX_CONCURRENT`，监控`memory.usage_percent`
2. **CPU瓶颈**：调整`WORKER_THREADS`，观察并发处理能力
3. **网络延迟**：检查TCP参数和网络配置
4. **响应慢**：减少`RESPONSE_DELAY_SECONDS`用于测试
5. **资源限制测试**：使用Docker严格限制CPU和内存资源

## 🔒 安全特性

- 非root用户运行
- 请求体大小限制（16KB）
- 优雅关闭，防止数据丢失
- 健康检查防止级联故障

## 💡 最佳实践

1. **容器部署**：使用docker-compose进行资源限制
2. **监控集成**：配合Prometheus + Grafana监控
3. **负载均衡**：多实例部署提高可用性
4. **压测策略**：逐步增加并发，观察系统表现
5. **内存监控**：实时监控`memory.usage_percent`，避免内存溢出
6. **健康检查**：集成到负载均衡器，自动剔除不健康实例
7. **资源限制测试**：使用容器严格限制资源，验证真实性能

### 压测建议 🆕
```bash
# 1. 启动服务并监控
./target/release/llm-mock-server &
watch -n 1 'curl -s http://localhost:8080/health'

# 2. 逐步增加并发压测
for concurrent in 100 500 1000 1500 2000; do
  echo "测试 $concurrent 并发..."
  ab -n $concurrent -c $concurrent -s 30 \
     -p post_data.json -T application/json \
     http://localhost:8080/compatible-mode/v1/chat/completions
  
  # 检查内存使用情况
  curl -s http://localhost:8080/health | jq '.memory.usage_percent'
  sleep 5
done
```

## 🎯 实际使用场景

### 1核1G资源限制压测
```bash
# 启动受限服务（Docker方式）
docker run --cpus="1.0" --memory="1g" -p 8080:8080 llm-mock-server

# 监控内存使用情况
curl -s http://localhost:8080/health | jq '{
  status: .status,
  concurrent: .concurrent_usage,
  memory_mb: .memory.used_mb,
  memory_percent: .memory.usage_percent
}'

# 输出示例:
# {
#   "status": "healthy",
#   "concurrent": "45.2%",
#   "memory_mb": 856,
#   "memory_percent": "83.6%"
# }
```

### 压测场景下的内存预警
```bash
# 当内存使用率超过95%时，服务自动返回503状态
curl -s http://localhost:8080/health | jq '.status'
# 返回: "overloaded"

# 适合集成到监控告警系统
if [ "$(curl -s http://localhost:8080/health | jq -r '.memory.usage_percent | tonumber')" -gt 90 ]; then
  echo "⚠️ 内存使用率超过90%，请注意！"
fi
```

这个优化后的服务相比原版本，在相同资源下能处理数十倍的并发请求，并提供完整的内存监控功能，是进行LLM系统压测的理想选择。 