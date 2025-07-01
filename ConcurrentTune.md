# LLM 业务高并发参数调优手册

（以「客户端 = 调用外部 LLM API 的 Java WebClient 服务」「服务端 = 对外提供 LLM API 的 Rust/Axum 模拟器」为例，Kubernetes 部署场景可直接套用）

---

## 一、核心概念速查表

| 关键字 | 作用 | 常见瓶颈表现 |
|-------|------|-------------|
| `ulimit -n` / `nofile` | 进程可打开 FD 上限 (含 socket) | 报 `Too many open files` |
| `fs.file-max` | 节点全局 FD 表上限 | 同上，影响所有 Pod |
| `net.core.somaxconn` | 已完成三握手、待 `accept` 队列最大值 | Listen 队列丢包、握手慢 |
| `net.ipv4.tcp_max_syn_backlog` | 半开连接 (`SYN_RECV`) 队列最大值 | `SYN` 重传、握手失败 |
| `net.ipv4.ip_local_port_range` | **主动出站**连接可选源端口区间 | 报 `EADDRNOTAVAIL`、端口耗尽 |
| `net.netfilter.nf_conntrack_max` | NAT/iptables 连接跟踪表容量 | `table full, dropping packet` |
| `SETTINGS_MAX_CONCURRENT_STREAMS` | 每条 HTTP/2 连接并发 Stream 数 | H2 并发不足、端口占用高 |

---

## 二、客户端（Java / WebClient）调优

1. **连接池**  
   ```java
   ConnectionProvider provider = ConnectionProvider.builder("llm-pool")
       .maxConnections(1000)          // TCP 连接数
       .pendingAcquireMaxCount(2000)  // 等待队列
       .pendingAcquireTimeout(ofSeconds(30))
       .build();
   ```
   - HTTP/1.1：并发 ≈ `maxConnections`  
   - HTTP/2：并发 ≈ `maxConnections × SETTINGS_MAX_CONCURRENT_STREAMS`  
   - 若厂商 H2 默认 100 流，可先设 1000 连接 ≈ 10 万并发。

2. **HTTP/2 / TLS**  
   - `HttpClient.create().protocol(HttpProtocol.H2)`  
   - 若遇到端口耗尽请求供应商调大 `SETTINGS_MAX_CONCURRENT_STREAMS` 或 **多域名 / 多 VIP** 分流。

3. **JVM 资源**  
   - `-Xms` = `-Xmx`（避免 Full GC）  
   - `-XX:+UseG1GC`、`-XX:MaxGCPauseMillis=200`  
   - 监控堆外 DirectBuffer，还要关注 Reactor Netty `reactor.netty.connection.provider.*` 指标。

4. **线程池**  
   - Netty EventLoop = `cpu * 2` 足够；阻塞代码放 `boundedElastic`。  
   - 不明阻塞可用 `BlockHound` 检查。

5. **Pod / 容器**  
   ```yaml
   resources:
     limits:
       cpu: "2"
       memory: 2Gi
   ulimits:
     nofile: { soft: 65535, hard: 65535 }
   securityContext:
     sysctls:
       - { name: net.ipv4.ip_local_port_range, value: "1024 65535" }
   ```

6. **熔断 & 限流策略**  
   - 对 429/503 实现指数退避。  
   - 使用 Resilience4j / Sentinel 做并发隔离，避免级联雪崩。

---

## 三、服务端（Rust / Axum）调优

1. **应用层并发**  
   - `MAX_CONCURRENT=6000`（默认），可根据内核资源调至 3 万+。  
   - Tokio `worker_threads = CPU*2~4`，`thread_stack_size=512 KB` 减少内存。

2. **FD 与连接队列**  
   ```bash
   ulimit -n 65535
   sysctl -w fs.file-max=200000
   sysctl -w net.core.somaxconn=65535
   sysctl -w net.ipv4.tcp_max_syn_backlog=65535
   ```

3. **nf_conntrack**  
   ```bash
   sysctl -w net.netfilter.nf_conntrack_max=262144
   sysctl -w net.netfilter.nf_conntrack_tcp_timeout_established=1200
   ```

4. **端口池**（若服务端也需向外请求）  
   ```bash
   sysctl -w net.ipv4.ip_local_port_range="1024 65535"
   ```

5. **K8s 配置**  
   ```yaml
   securityContext:
     sysctls:
       - { name: net.core.somaxconn, value: "65535" }
       - { name: net.ipv4.tcp_max_syn_backlog, value: "65535" }
       - { name: net.netfilter.nf_conntrack_max, value: "262144" }
   ```

6. **监控**  
   - `/proc/self/limits` 中 `open files` vs `ls /proc/$$/fd | wc -l`  
   - Prometheus 指标：`node_netstat_TcpExt_ListenDrops`, `nf_conntrack_entries`, `llm_mock_active_requests`（示例）。

---

## 四、网络与基础设施

| 维度 | 建议 |
|------|------|
| **负载均衡** | L4(LVS) 连接跟踪成本低；L7(Istio/Envoy) 要同步调大 `max_connections`, `max_requests_per_connection`。 |
| **带宽** | 理论吞吐 = RPS × 响应包大小；注意 1 Gbps ≈ 125 MB/s。 |
| **NAT 表** | 公网 egress 出口若 NAT 端口不足易掉包，需要按连接数扩展 SNAT IP。 |
| **TLS** | TLS 握手耗 CPU，可启用 Session Resumption 或把重负放在 Ingress。 |
| **DNS** | 高并发下 DNS 缓存不足会放大解析延迟，可用集群 CoreDNS 缓存或 /etc/hosts 固定。 |

---

## 五、测试与演练流程

1. 调大参数 ➜ `k6 / wrk2` 压测 ➜ 观察 p99 延迟、5xx。  
2. 若握手失败：看 `ListenDrops` / `tcp_max_syn_backlog`。  
3. 若连接错误：查 `nf_conntrack: table full` / 端口耗尽 / FD。  
4. 若 p99 持续上升：CPU 饱和或下游限流。  
5. 设置自动降级（排队、拒绝、缓存）+ 可观测性告警。

---

## 六、常被忽略的细节

1. **HTTP/2 Stream 限制**  
   - 客户端、服务端、LB 任一端都可能把 `SETTINGS_MAX_CONCURRENT_STREAMS` 限低。  
   - 可用 `nghttp -ans https://host` 查看真实协商值。

2. **TIME_WAIT**  
   - 出站短连接过多时，`tcp_tw_reuse=1`、`tcp_tw_recycle`（慎用）可回收 TIME_WAIT；H2/LB 长连接更优。

3. **容器间 DNS 解析**  
   - 并发暴增时 CoreDNS QPS 不够会拖慢链路，需加副本或本地缓存。

4. **GC 暂停**  
   - Java 堆过大或未调 GC 参数会在高并发下放大 STW 时间。

5. **供应商限流策略变更**  
   - 主动抓 4xx/5xx 返回码并动态调节速率，避免黑名单。