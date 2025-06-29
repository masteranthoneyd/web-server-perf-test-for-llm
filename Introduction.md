# LLM 业务场景 Web Server 性能测试项目

## 1. 项目简介

本项目旨在通过模拟真实的大语言模型（LLM）调用场景，对不同的 Java Web Server 框架（Tomcat, Undertow, WebFlux）进行性能评估与比较。同时，项目也将分析不同 HTTP Client（OkHttp, WebClient, Java 21 HttpClient）对整体性能的影响，以期为相似业务场景的技术选型提供数据支持。

## 2. 业务场景与技术栈

### 2.1 业务流程

1.  前端应用向业务系统发起一个 RESTful API 请求。
2.  业务系统接收请求后，通过内部的 HTTP Client 调用一个模拟的 LLM 服务。
3.  模拟的 LLM 服务会随机等待 5 到 20 秒，然后返回一个与 OpenAI Chat Completion API 格式兼容的 JSON 响应。
4.  业务系统接收到响应后，将其透传给前端应用。

### 2.2 关键需求说明

*   **调用方式**: 业务系统与 LLM 模拟服务之间的调用需要支持**同步**和**异步**两种实现，以便对比其性能差异。
*   **响应模式**: LLM 模拟服务仅需支持**非流式（Unary）**响应。
*   **技术栈**: 业务系统将基于 **Java 21** 和 **Spring Boot 3** 构建。

### 2.3 API 规约

*   **业务系统 API 入口**:
    *   `POST /api/chat/completions`
    *   请求体示例:
        ```json
        {
            "prompt": "你好，请介绍一下你自己"
        }
        ```

*   **LLM Mock Service 端点**:
    `POST /compatible-mode/v1/chat/completions`

*   **LLM Mock Service 响应体示例**:
    ```json
    {
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
    }
    ```

## 3. 项目结构设计

项目将包含三个核心目录：

1.  `llm-mock-server`:
    *   **用途**: 一个用 Rust 实现的轻量级 LLM API 模拟器。
    *   **目的**: 提供一个高性能、低资源占用的服务端，以模拟 LLM 的响应延迟，为性能测试提供稳定的对端服务。

2.  `llm-webservers`:
    *   **用途**: 存放核心的 Java 业务系统代码。
    *   **结构**: 包含多个子模块：
        *   `llm-webserver-tomcat`: 基于 Spring Boot 内嵌 Tomcat 的实现。
        *   `llm-webserver-undertow`: 基于 Spring Boot 内嵌 Undertow 的实现。
        *   `llm-webserver-webflux`: 基于 Spring WebFlux 的响应式实现。
        *   `llm-gateway`: 一个统一的网关/客户端模块，封装了对 `llm-mock-server` 的调用逻辑。此模块将包含使用不同 HTTP Client（如 OkHttp, WebClient, Java 21 HttpClient）的实现。

3.  `perf-tester`:
    *   **用途**: 存放用于执行性能压测的脚本和工具。

## 4. 模块功能与实现要点

### 4.1 llm-mock-server (Rust)

*   使用高性能的 Rust Web 框架（如 Axum 或 Actix-web）构建。
*   监听 HTTP POST 请求。
*   接收到请求后，随机 `sleep` 5-20 秒。
*   返回一个固定的、符合 OpenAI Chat Completion API 格式的 JSON 对象。

### 4.2 llm-webservers (Java)

*   **API 接口**: 所有 Web Server 实现 (`tomcat`, `undertow`, `webflux`) 都应暴露一个相同的 REST API 端点，例如 `POST /api/chat`。
*   **HTTP Client 切换策略**: `llm-gateway` 模块将使用 **Spring Profiles** 来动态切换不同的 HTTP Client 实现。例如，定义 `okhttp`, `webclient`, `java21` 等 profiles，在启动时通过 `-Dspring.profiles.active=<profile_name>` 来激活对应的客户端配置。
*   **配置管理**: 使用 Spring Boot 的 `application.properties` 或 `application.yml` 来管理端口、LLM Mock 服务地址、超时时间等配置。

### 4.3 perf-tester

*   **工具选型**:
    *   **首选工具**: **Gatling**。因其能生成丰富、美观的 HTML 性能报告，非常适合进行详细的性能分析和结果展示。
    *   **备选工具**: `wrk`, `oha` 可用于快速的命令行基准测试；`JMeter`, `Locust` 适用于其他需要复杂脚本或不同生态的场景。
*   **测试指标**: 重点关注以下指标：
    *   **吞吐量 (Throughput)**: 每秒请求数 (RPS)。
    *   **延迟 (Latency)**: 平均响应时间、P99/P95/P50 响应时间。
    *   **错误率 (Error Rate)**。

## 5. 容器化部署与资源限制

*   **Dockerfile**: 项目中的每个可独立运行的服务（`llm-mock-server` 和各个 `llm-webserver-*` 模块）都必须提供一个 `Dockerfile`。
*   **部署方式**: 测试将采用独立启动容器的方式，而非使用 `docker-compose`。每次只启动一个待测的 Java 服务和一个 `llm-mock-server` 实例。
*   **资源限制**:
    *   **Java 业务服务**: 容器资源限制为 **1 CPU 核心** 和 **1 GB 内存**，以模拟典型的云原生部署环境。
        *   启动命令示例: `docker run --cpus=1 --memory=1g ...`
    *   **Rust LLM Mock 服务**: 容器资源**不做限制**，以确保其不会成为性能瓶颈。

## 6. 开发与测试指南

1.  **接口规范**: 统一所有 REST API 的请求和响应格式，方便自动化测试。
2.  **可扩展性**: 设计应便于未来增加新的 Web Server（如 Jetty）或 HTTP Client 实现。
3.  **性能监控**: 建议在 Java 应用中集成 Micrometer，并暴露 `/actuator/prometheus` 端点，以便在需要时进行更深入的性能剖析。
4.  **文档**: 为每个模块编写 `README.md`，清晰说明其功能、如何构建和运行。

## 7. 性能测试计划

### 7.1 测试矩阵

为了系统性地评估不同技术组合的性能，我们将执行以下测试矩阵。

| Web Server | HTTP Client | 调用模式 | 是否测试 | 备注 |
| :--- | :--- | :--- | :--- | :--- |
| Tomcat | OkHttp | Sync | 是 | 传统的阻塞 I/O 模型 |
| Tomcat | Java 21 HttpClient | Sync | 是 | |
| Tomcat | WebClient | Sync | 是 | `WebClient` 在阻塞模式下的表现 |
| Undertow | OkHttp | Sync | 是 | |
| Undertow | Java 21 HttpClient | Sync | 是 | |
| Undertow | WebClient | Sync | 是 | `WebClient` 在阻塞模式下的表现 |
| WebFlux | WebClient | Async | 是 | 纯响应式技术栈 |
| WebFlux | Java 21 HttpClient | Async | 是 | 在响应式环境中集成虚拟线程 |

### 7.2 Gatling 压测配置

*   **并发用户数 (Concurrent Users)**: 从 50, 100, 200 逐步增加，观察系统在不同压力下的表现。
*   **加压时间 (Ramp-up Period)**: 30 秒内达到目标并发数。
*   **测试时长 (Duration)**: 每个测试场景持续运行 5 分钟。

---

文档整理完毕。现在我们可以继续下一步了。
