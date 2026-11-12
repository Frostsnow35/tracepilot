# 架构

## 系统分层

```
                    ┌─────────────────────────────────────────┐
 Agent (SDK) ──OTel──▶│  trace-ingest (Axum, 批量/异步)        │
                    │       │                                  │
                    │       ▼                                  │
                    │  storage ──▶ PostgreSQL + TimescaleDB     │
                    │       │         ├─ hypertable: spans      │
                    │       │         └─ tables: traces/projects │
                    │       ▼                                  │
                    │  query / monitor / analytics / recovery    │
                    │       │                                  │
                    │       ▼                                  │
                    │  WebSocket 实时推送 + REST API             │
                    └───────────────┬───────────────────────────┘
                                    ▼
                          Next.js 前端（trace/monitor/analytics）
```

## 模块依赖

模块依赖单向，无横向依赖：

```
        iam（根）
          │
          ▼
     storage
         │
    ┌────┴────┬────────┬────────┐
    ▼         ▼        ▼        ▼
 query    monitor  analytics  recovery
    │         │
    └─────────┴──────────────┐
                           ▼
                         alert
```

- **iam**：项目与 API Key 管理，所有非健康检查路由的鉴权入口
- **storage**：数据访问层，封装所有 SQL 查询，模块间共享
- **ingest**：trace/span 接入，依赖 storage + iam
- **query**：trace 列表与详情查询，依赖 storage + iam
- **monitor**：实时统计与 WebSocket 流，依赖 storage + iam
- **analytics**：成功率/延迟/成本分析，依赖 storage + iam
- **recovery**：重试与恢复观测，依赖 storage + analytics + iam
- **alert**：阈值告警规则与事件，依赖 monitor + storage + iam

## 模块插件化

Rust 无运行时注册表，用 **Cargo workspace 分层 + Axum Router 组合 + `Module` trait** 实现插件化：

每个模块实现 `Module` trait：

```rust
pub trait Module {
    fn name(&self) -> &'static str;
    fn depends_on(&self) -> Vec<&'static str> { vec![] }
    fn router(&self) -> Router<AppState>;
}
```

registry 按依赖拓扑序 `Router::nest("/api/<name>", ...)` 挂载，新增模块在 registry 登记一行即接入，核心 `main.rs` 不改。

## 数据流

### 接入链路

SDK 本地聚合批量 → ingest API 校验（鉴权 + 项目隔离）→ 事务写库（trace + spans 原子写入）→ 返回 trace_id

### 查询链路

前端拉 REST → query 模块先鉴权再按项目过滤 → 解析 span 树 → 瀑布图渲染

### 实时链路

监控页 WebSocket 连接（`api_key` query param 鉴权）→ 后端 broadcast channel 推送 → 前端实时更新

后台任务：
- **monitor poll**：5s 轮询新 trace，水位线模式（查询前捕获 `upper = now()`，避免跳过查询期间写入的 trace）
- **alert eval**：30s 评估启用规则，触发后记录事件并更新 `last_triggered_at`（10min 冷却）

## 数据模型

| 实体 | 关键字段 | 存储 |
|---|---|---|
| Project | id, name, owner, api_key_hash, created_at | PG |
| Agent | id, project_id, name, framework, version | PG |
| Trace | id, project_id, agent_id, status, started_at, duration_ms, recorded_at | PG |
| Span | id, trace_id, parent_span_id, name, kind, status, started_at, duration_ms, attributes(jsonb), events(jsonb), recorded_at | TSDB hypertable |
| AlertRule | id, project_id, metric, operator, threshold, enabled, last_triggered_at | PG |
| AlertEvent | id, rule_id, triggered_at, value | PG |

Span kind：`llm` / `tool` / `agent` / `retrieval` / `chain` / `custom`

### 关键设计

- **spans hypertable**：按 `recorded_at` 分区，TimescaleDB 原生时序存储
- **spans 复合主键** `(id, recorded_at)`：TS103 约束，hypertable 唯一索引必须含分区列
- **recorded_at 列**：migration 0002 新增，monitor 轮询水位线用
- **api_key_hash**：SHA-256 存储，不存明文
- **事务写入**：`insert_trace_with_spans` 包裹 trace + spans，避免半成品
