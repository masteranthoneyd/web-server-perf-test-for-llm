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

# 构建
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