use axum::{
    extract::State,
    routing::{get, post},
    Router,
    http::{StatusCode, HeaderMap, header},
    response::IntoResponse,
};
use std::{
    env,
    sync::Arc,
    time::Duration,
    fs,
};
use tokio::{
    sync::Semaphore,
    time::sleep,
    signal,
};
use tower::ServiceBuilder;
use tower_http::limit::RequestBodyLimitLayer;
use tracing_subscriber;
use once_cell::sync::Lazy;
use sysinfo::System;

// 预分配的静态响应内容，避免重复分配
static RESPONSE_BODY: &str = r#"{
    "choices": [
        {
            "message": {
                "role": "assistant",
                "content": "我是阿里云开发的一款超大规模语言模型，我叫通义千问。"
            },
            "finish_reason": "stop",
            "index": 0,
            "logprobs": null
        }
    ],
    "object": "chat.completion",
    "usage": {
        "prompt_tokens": 3019,
        "completion_tokens": 104,
        "total_tokens": 3123,
        "prompt_tokens_details": {
            "cached_tokens": 2048
        }
    },
    "created": 1735120033,
    "system_fingerprint": null,
    "model": "qwen-plus",
    "id": "chatcmpl-6ada9ed2-7f33-9de2-8bb0-78bd4035025a"
}"#;

// 预分配的响应头，避免重复创建
static RESPONSE_HEADERS: Lazy<HeaderMap> = Lazy::new(|| {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
    headers
});

// HTTP/2 配置说明:
//
// 在 axum 0.6 + hyper 0.14 环境下，HTTP/2 的最大并发流配置有以下特点：
//
// 1. **默认值**: hyper 默认 max_concurrent_streams = 200 (或接近此值)
// 2. **配置方式**: 当前版本的 axum::Server 没有直接暴露 HTTP/2 低级配置
// 3. **实际限制**:
//    - 单个 HTTP/2 连接: 最多 200 个并发 stream (hyper 控制)
//    - 应用层总限制: MAX_CONCURRENT 参数控制总体并发请求数
//    - 协议支持: HTTP/1.1 和 HTTP/2 (h2c) 自动协商
//
// 4. **性能考虑**:
//    - HTTP/2 多路复用减少连接开销
//    - 200 个并发 stream 足够支持大部分应用场景
//    - 应用层信号量提供额外的并发控制和资源保护
//
// 5. **验证方法**: 使用提供的测试脚本验证实际并发能力
//
// 如需更精细的 HTTP/2 配置，可考虑升级到 axum 0.7+ 或使用自定义 hyper 服务器。

// 应用配置
#[derive(Clone)]
struct AppConfig {
    max_concurrent_requests: usize,
    response_delay_seconds: u64,
    port: u16,
    worker_threads: usize,
    http2_max_concurrent_streams: u32,  // HTTP/2 最大并发 stream 数量
}

impl Default for AppConfig {
    fn default() -> Self {
        let cpu_count = num_cpus::get();

        // 根据CPU核心数智能配置（基于实测优化）
        let (max_concurrent, worker_threads) = match cpu_count {
            1 => (10000, 2),
            2 => (20000, 4),
            _ => (40000, 8),
        };

        Self {
            max_concurrent_requests: env::var("MAX_CONCURRENT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(max_concurrent),
            response_delay_seconds: env::var("RESPONSE_DELAY_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            port: env::var("PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8080),
            worker_threads: env::var("WORKER_THREADS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(worker_threads),
            http2_max_concurrent_streams: env::var("HTTP2_MAX_CONCURRENT_STREAMS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(200),  // 默认设置为 200 个并发 stream
        }
    }
}

// 内存信息来源
#[derive(Debug)]
enum MemorySource {
    CGroupV2,
    CGroupV1,
    SystemFallback,
}

// CPU 信息来源
#[derive(Debug)]
enum CpuSource {
    CGroupV2,
    CGroupV1,
    SystemFallback,
}

// 内存信息结构体
#[derive(Debug)]
struct MemoryInfo {
    total_bytes: u64,
    used_bytes: u64,
    available_bytes: u64,
    source: MemorySource,
}

// CPU 信息结构体
#[derive(Debug)]
struct CpuInfo {
    total_cores: f64,    // 总核心数（支持小数，如 1.5 核）
    available_cores: f64, // 可用核心数
    source: CpuSource,
}

// 应用状态
#[derive(Clone)]
struct AppState {
    semaphore: Arc<Semaphore>,
    config: AppConfig,
}

// 获取容器内存信息（优先使用cgroup，回退到系统信息）
fn get_container_memory_info() -> MemoryInfo {
    // 首先尝试从 cgroup v2 获取内存信息
    if let Some(memory_info) = get_cgroup_v2_memory_info() {
        return memory_info;
    }

    // 尝试从 cgroup v1 获取内存信息
    if let Some(memory_info) = get_cgroup_v1_memory_info() {
        return memory_info;
    }

    // 回退到系统信息（可能是宿主机信息）
    get_system_memory_info()
}

// 获取容器 CPU 信息（优先使用cgroup，回退到系统信息）
fn get_container_cpu_info() -> CpuInfo {
    // 首先尝试从 cgroup v2 获取 CPU 信息
    if let Some(cpu_info) = get_cgroup_v2_cpu_info() {
        return cpu_info;
    }

    // 尝试从 cgroup v1 获取 CPU 信息
    if let Some(cpu_info) = get_cgroup_v1_cpu_info() {
        return cpu_info;
    }

    // 回退到系统信息
    get_system_cpu_info()
}

// 从 cgroup v2 获取内存信息
fn get_cgroup_v2_memory_info() -> Option<MemoryInfo> {
    // cgroup v2 统一层级结构
    let cgroup_path = "/sys/fs/cgroup";

    // 读取内存限制
    let memory_max = fs::read_to_string(format!("{}/memory.max", cgroup_path)).ok()?;
    let total_bytes = if memory_max.trim() == "max" {
        // 没有限制，使用系统内存
        return None;
    } else {
        memory_max.trim().parse::<u64>().ok()?
    };

    // 读取内存使用情况
    let memory_current = fs::read_to_string(format!("{}/memory.current", cgroup_path)).ok()?;
    let used_bytes = memory_current.trim().parse::<u64>().ok()?;

    let available_bytes = total_bytes.saturating_sub(used_bytes);

    Some(MemoryInfo {
        total_bytes,
        used_bytes,
        available_bytes,
        source: MemorySource::CGroupV2,
    })
}

// 从 cgroup v1 获取内存信息
fn get_cgroup_v1_memory_info() -> Option<MemoryInfo> {
    // 读取当前进程的cgroup信息
    let cgroup_info = fs::read_to_string("/proc/self/cgroup").ok()?;

    // 查找memory子系统的路径
    let memory_cgroup_line = cgroup_info
        .lines()
        .find(|line| line.contains("memory"))?;

    let cgroup_path = memory_cgroup_line
        .split(':')
        .nth(2)?;

    let memory_cgroup_root = format!("/sys/fs/cgroup/memory{}", cgroup_path);

    // 读取内存限制
    let limit_file = format!("{}/memory.limit_in_bytes", memory_cgroup_root);
    let memory_limit = fs::read_to_string(&limit_file).ok()?;
    let total_bytes = memory_limit.trim().parse::<u64>().ok()?;

    // 检查是否实际有限制（非常大的值表示无限制）
    if total_bytes > (1_u64 << 50) { // 超过 1PB 认为是无限制
        return None;
    }

    // 读取内存使用情况
    let usage_file = format!("{}/memory.usage_in_bytes", memory_cgroup_root);
    let memory_usage = fs::read_to_string(&usage_file).ok()?;
    let used_bytes = memory_usage.trim().parse::<u64>().ok()?;

    let available_bytes = total_bytes.saturating_sub(used_bytes);

    Some(MemoryInfo {
        total_bytes,
        used_bytes,
        available_bytes,
        source: MemorySource::CGroupV1,
    })
}

// 从系统信息获取内存信息（回退方案）
fn get_system_memory_info() -> MemoryInfo {
    let mut sys = System::new_all();
    sys.refresh_memory();

    MemoryInfo {
        total_bytes: sys.total_memory() * 1024, // sysinfo 返回KB，转换为字节
        used_bytes: sys.used_memory() * 1024,
        available_bytes: sys.available_memory() * 1024,
        source: MemorySource::SystemFallback,
    }
}

// 从 cgroup v2 获取 CPU 信息
fn get_cgroup_v2_cpu_info() -> Option<CpuInfo> {
    let cgroup_path = "/sys/fs/cgroup";

    // 读取 CPU 限制 (cpu.max 格式: "配额 周期" 或 "max")
    let cpu_max = fs::read_to_string(format!("{}/cpu.max", cgroup_path)).ok()?;
    let cpu_max = cpu_max.trim();

    let total_cores = if cpu_max == "max" {
        // 没有限制，回退到系统 CPU 数
        return None;
    } else {
        let parts: Vec<&str> = cpu_max.split_whitespace().collect();
        if parts.len() != 2 {
            return None;
        }

        let quota = parts[0].parse::<u64>().ok()?;
        let period = parts[1].parse::<u64>().ok()?;

        if period == 0 {
            return None;
        }

        quota as f64 / period as f64
    };

    // CPU 使用率较难实时计算，这里假设可用核心数等于总核心数
    let available_cores = total_cores;

    Some(CpuInfo {
        total_cores,
        available_cores,
        source: CpuSource::CGroupV2,
    })
}

// 从 cgroup v1 获取 CPU 信息
fn get_cgroup_v1_cpu_info() -> Option<CpuInfo> {
    // 读取当前进程的cgroup信息
    let cgroup_info = fs::read_to_string("/proc/self/cgroup").ok()?;

    // 查找cpu子系统的路径
    let cpu_cgroup_line = cgroup_info
        .lines()
        .find(|line| line.contains("cpu,") || line.contains("cpu:"))?;

    let cgroup_path = cpu_cgroup_line
        .split(':')
        .nth(2)?;

    let cpu_cgroup_root = format!("/sys/fs/cgroup/cpu{}", cgroup_path);

    // 读取 CPU 配额和周期
    let quota_file = format!("{}/cpu.cfs_quota_us", cpu_cgroup_root);
    let period_file = format!("{}/cpu.cfs_period_us", cpu_cgroup_root);

    let quota = fs::read_to_string(&quota_file).ok()?
        .trim().parse::<i64>().ok()?;
    let period = fs::read_to_string(&period_file).ok()?
        .trim().parse::<u64>().ok()?;

    if quota <= 0 || period == 0 {
        // 没有限制
        return None;
    }

    let total_cores = quota as f64 / period as f64;
    let available_cores = total_cores;

    Some(CpuInfo {
        total_cores,
        available_cores,
        source: CpuSource::CGroupV1,
    })
}

// 从系统信息获取 CPU 信息（回退方案）
fn get_system_cpu_info() -> CpuInfo {
    let total_cores = num_cpus::get() as f64;

    CpuInfo {
        total_cores,
        available_cores: total_cores, // 假设所有核心都可用
        source: CpuSource::SystemFallback,
    }
}

fn main() {
    let config = AppConfig::default();

    println!("配置信息:");
    println!("  最大并发请求数: {}", config.max_concurrent_requests);
    println!("  响应延迟: {}秒", config.response_delay_seconds);
    println!("  工作线程数: {}", config.worker_threads);
    println!("  监听端口: {}", config.port);
    println!("  HTTP/2 最大并发流: {}", config.http2_max_concurrent_streams);

    // 构建优化的Tokio运行时
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(config.worker_threads)
        .thread_name("llm-mock-worker")
        .thread_stack_size(512 * 1024) // 512KB栈大小，比默认更小
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime");

    rt.block_on(async_main(config));
}

async fn async_main(config: AppConfig) {
    tracing_subscriber::fmt::init();

    let state = AppState {
        semaphore: Arc::new(Semaphore::new(config.max_concurrent_requests)),
        config: config.clone(),
    };

    let app = Router::new()
        .route("/compatible-mode/v1/chat/completions", post(mock_handler))
        .route("/health", get(health_check))
        .route("/metrics", get(metrics))
        .layer(
            ServiceBuilder::new()
                .layer(RequestBodyLimitLayer::new(1024 * 16)) // 限制请求体大小16KB
                .into_inner(),
        )
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.port);
    println!("启动服务器: {} (支持 HTTP/1 和 HTTP/2)", addr);
    println!("HTTP/2 配置:");
    println!("  - 每个连接最大并发 stream: {} (由 hyper 控制)", config.http2_max_concurrent_streams);
    println!("  - 总体并发限制: {} (由应用层信号量控制)", config.max_concurrent_requests);
    println!("  - 协议: HTTP/1.1 和 HTTP/2 (h2c) 自动协商");

    let server = axum::Server::bind(&addr.parse().unwrap())
        .tcp_nodelay(true)
        .tcp_keepalive(Some(Duration::from_secs(60))) // 启用TCP keepalive
        .serve(app.into_make_service());

    // 优雅关闭
    let graceful = server.with_graceful_shutdown(shutdown_signal());

    if let Err(e) = graceful.await {
        eprintln!("服务器错误: {}", e);
    }
}

// 优化的请求处理器
async fn mock_handler(State(state): State<AppState>) -> impl IntoResponse {
    // 获取信号量许可，控制并发数
    let _permit = match state.semaphore.try_acquire() {
        Ok(permit) => permit,
        Err(_) => {
            // 并发数超限，返回503
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                RESPONSE_HEADERS.clone(),
                "Service temporarily unavailable - too many concurrent requests",
            );
        }
    };

    // 模拟处理延迟
    sleep(Duration::from_secs(state.config.response_delay_seconds)).await;

    // 返回预分配的响应
    (StatusCode::OK, RESPONSE_HEADERS.clone(), RESPONSE_BODY)
}

// 健康检查端点
async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let available_permits = state.semaphore.available_permits();
    let total_permits = state.config.max_concurrent_requests;
    let usage_percent = ((total_permits - available_permits) as f64 / total_permits as f64) * 100.0;

    // 获取容器内存信息（会自动检测cgroup限制）
    let memory_info = get_container_memory_info();

    // 转换为MB便于显示 (字节 / 1024 / 1024 = MB)
    let total_memory_mb = memory_info.total_bytes / (1024 * 1024);
    let used_memory_mb = memory_info.used_bytes / (1024 * 1024);
    let available_memory_mb = memory_info.available_bytes / (1024 * 1024);
    let memory_usage_percent = if memory_info.total_bytes > 0 {
        (memory_info.used_bytes as f64 / memory_info.total_bytes as f64) * 100.0
    } else {
        0.0
    };

    // 获取容器 CPU 信息（会自动检测cgroup限制）
    let cpu_info = get_container_cpu_info();

    let status = if usage_percent > 90.0 || memory_usage_percent > 95.0 {
        StatusCode::SERVICE_UNAVAILABLE
    } else {
        StatusCode::OK
    };

    let memory_source = match memory_info.source {
        MemorySource::CGroupV2 => "cgroup_v2",
        MemorySource::CGroupV1 => "cgroup_v1",
        MemorySource::SystemFallback => "system_fallback",
    };

    let cpu_source = match cpu_info.source {
        CpuSource::CGroupV2 => "cgroup_v2",
        CpuSource::CGroupV1 => "cgroup_v1",
        CpuSource::SystemFallback => "system_fallback",
    };

    let response = format!(
        r#"{{"status":"{}","concurrent_usage":"{:.1}%","available_slots":{},"memory":{{"total_mb":{},"used_mb":{},"available_mb":{},"usage_percent":"{:.1}%","source":"{}"}},"cpu":{{"total_cores":{:.2},"available_cores":{:.2},"source":"{}"}}}}"#,
        if status == StatusCode::OK { "healthy" } else { "overloaded" },
        usage_percent,
        available_permits,
        total_memory_mb,
        used_memory_mb,
        available_memory_mb,
        memory_usage_percent,
        memory_source,
        cpu_info.total_cores,
        cpu_info.available_cores,
        cpu_source
    );

    (status, [("content-type", "application/json")], response)
}

// 指标端点
async fn metrics(State(state): State<AppState>) -> impl IntoResponse {
    let available_permits = state.semaphore.available_permits();
    let total_permits = state.config.max_concurrent_requests;
    let active_requests = total_permits - available_permits;

    let response = format!(
        "# HELP llm_mock_active_requests Currently active requests\n\
         # TYPE llm_mock_active_requests gauge\n\
         llm_mock_active_requests {}\n\
         # HELP llm_mock_max_concurrent_requests Maximum concurrent requests\n\
         # TYPE llm_mock_max_concurrent_requests gauge\n\
         llm_mock_max_concurrent_requests {}\n",
        active_requests, total_permits
    );

    (StatusCode::OK, [("content-type", "text/plain")], response)
}

// 优雅关闭信号处理
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            println!("收到Ctrl+C信号，开始优雅关闭...");
        },
        _ = terminate => {
            println!("收到终止信号，开始优雅关闭...");
        },
    }
}