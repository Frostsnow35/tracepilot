# TracePilot 方案与规格

Agent 运行时可观测平台。让 AI Agent 像后端服务一样可监控、可排查、可优化。

---

## 1. 产品定位

面向所有构建 AI Agent 的开发者与小团队，提供运行时可观测性：Agent 跑起来能干活，挂了能知道为什么，慢了能定位到哪一步。对标 LangSmith / Langfuse / Phoenix，差异化在**中文优先、自托管轻量、Agent 运行时治理（重试/恢复/上下文压缩）**。

**一句话**：Agent 的 Datadog。

---

## 2. 目标用户与核心场景

| 画像 | 场景 | 现有方案的不足 |
|---|---|---|
| 独立开发者做 Agent | 单 Agent 调试，跑挂了不知道哪一步 | 靠 print 调试，无结构化 trace |
| 小团队（1–5 人） | 多个 Agent 上线后要监控成功率/延迟 | LangSmith 贵且境外，自托管复杂 |
| AI 应用创业团队 | 要给投资人/客户看运行质量 | 缺中文、轻量的观测面板 |

---

## 3. 功能需求与模块对应（核心约束）

每个功能需求（FR）唯一对应一个后端模块 + 前端页面，解耦清晰。

| 需求 ID | 功能需求 | 后端模块 | 后端目录（Rust） | 前端页面 | 依赖模块 |
|---|---|---|---|---|---|
| FR-1 | trace/span 数据接入（OTel 兼容） | trace-ingest | `crates/api/src/modules/ingest` | —（SDK 调用） | storage, iam |
| FR-2 | trace/span 时序与元数据存储 | storage | `crates/api/src/modules/storage` | — | — |
| FR-3 | trace 查询与瀑布图/span 详情 | trace-query | `crates/api/src/modules/query` | `/traces/[id]` | storage |
| FR-4 | Agent 实时运行状态监控 | monitor | `crates/api/src/modules/monitor` | `/monitor` | storage（实时推送） |
| FR-5 | 成功率/延迟/成本分析洞察 | analytics | `crates/api/src/modules/analytics` | `/analytics` | storage |
| FR-6 | 重试/故障恢复策略观测 | recovery | `crates/api/src/modules/recovery` | `/recovery` | storage, analytics |
| FR-7 | 用户/项目/API Key 管理 | iam | `crates/api/src/modules/iam` | `/settings` | — |
| FR-8 | 阈值告警 | alert | `crates/api/src/modules/alert` | `/alerts` | monitor |

模块依赖单向：`iam` 为根，`storage` 次之，其余模块只依赖 `storage` 与 `iam`，无横向依赖。

---

## 4. 技术选型（参考成熟产品）

| 层 | 选型 | 理由 | 参考产品 |
|---|---|---|---|
| 后端 | **Rust + Axum + tokio** | 编译期内存安全、零成本抽象、高并发低延迟；trace 接入是高吞吐管道，Rust 是性能上限 | 自研（对齐 tracing 生态） |
| ORM/查询 | **sqlx（异步）** | 编译期校验 SQL、原生 PostgreSQL + TimescaleDB | SeaORM 备选 |
| 序列化 | **serde + serde_json** | Rust 事实标准，零拷贝 | — |
| 日志/追踪 | **tracing + tracing-subscriber + tracing-opentelemetry** | 平台自身即"可观测"，用 tracing 生态实现 tracing 平台，主题自洽 | OpenTelemetry |
| 存储 | PostgreSQL 15 + TimescaleDB | span 是时序数据，hypertable 天然适配；关系数据用 PG 统一 | TimescaleDB |
| 实时 | WebSocket（tokio-tungstenite） | 监控页实时推送 | Grafana Live |
| 前端 | Next.js 14 + React + TS + Tailwind + ECharts | 现代、SSR、图表生态强 | LangSmith、Datadog |
| SDK | Python + TypeScript + Rust，OpenTelemetry 兼容 | 一套标准，生态接入零成本 | OpenTelemetry |
| 部署 | Docker Compose（MVP）→ Helm（后续） | 自托管轻量 | — |
| 质量 | **rustfmt + clippy -D warnings + cargo test** + Playwright + pre-commit | Rust 标配，clippy 严格模式 | — |
| CI | GitHub Actions（cache + 分步 build/test/lint） | 开源标配 | — |

**不选 Langfuse 直接二开**：产品化要自主可控，且要体现自身技术深度（Rust 存储/查询/可视化全自研）。

**Rust 模块组织**：Rust 无运行时注册表，用 **Cargo workspace 分层 + Axum Router 组合 + `Module` trait** 实现插件化。每模块实现 `Module` trait（`name`/`router`/`depends_on`），注册表按依赖拓扑序 `Router::merge` 挂载，新增模块在 registry 登记一行即接入，核心 `main.rs` 不改。

---

## 5. 系统架构

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

接入链路：SDK 本地聚合批量 → ingest API 校验（鉴权+限流）→ 异步写库 → 返回 trace_id。
查询链路：前端拉 REST → query 模块解析 span 树 → 瀑布图渲染。

---

## 6. 数据模型核心实体

| 实体 | 关键字段 | 存储 |
|---|---|---|
| Project | id, name, owner, api_key_hash, created_at | PG |
| Agent | id, project_id, name, framework, version | PG |
| Trace | id, project_id, agent_id, status, started_at, duration_ms | PG |
| Span | id, trace_id, parent_span_id, name, kind, status, started_at, duration_ms, attributes(jsonb), events(jsonb) | TSDB hypertable |
| Event | span_id, timestamp, name, attributes | TSDB |
| Alert | id, project_id, rule, threshold, enabled, last_triggered_at | PG |

Span kind：`llm` / `tool` / `agent` / `retrieval` / `chain` / `custom`。

---

## 7. MVP 范围（垂直切片）

MVP 只做打通闭环的最小集，证明核心价值：

- **FR-1 接入**：ingest API + Python SDK
- **FR-2 存储**：span/trace 落库（TimescaleDB）
- **FR-3 查询**：trace 列表 + 瀑布图 + span 详情
- **FR-7 鉴权**：API Key 项目隔离
- **示例**：`examples/demo_agent` 产出含 LLM/tool 调用、一次失败+重试的真实 trace

MVP 验收：跑 demo_agent → 平台出现该 trace → 点开看瀑布图 → 失败 span 标红 → 重试 span 可见。

---

## 8. 工程规范与开源拓展

- **目录规范**：后端 Cargo workspace，`crates/core`（配置/模块 trait/类型）+ `crates/api`（Axum 服务，`src/modules/*` 一模块一目录，各自 `mod.rs` 含 `router`/`service`/`repo`）；前端 `src/app` 按页面 + `src/lib` 按领域。
- **代码规范**：`rustfmt` + `clippy -D warnings`（CI 阻断）+ `cargo test` 覆盖率门槛；pre-commit 拦截格式与 clippy。
- **开源拓展**：
  - 模块插件化：每模块实现 `Module` trait（`name`/`router`/`depends_on`），在 registry 登记一行即被 `Router::merge` 挂载，核心 `main.rs` 不改
  - SDK 开放：OTel 兼容，第三方 Agent 框架零改接入
  - 文档：Axum 路由文档 + MkDocs 开发者文档
  - License：Apache-2.0，贡献指南 CONTRIBUTING.md
- **样板整理**：README、CHANGELOG、CONTRIBUTING、CODE_OF_CONDUCT、Issue/PR 模板齐全。
- **禁止占位/降级**：所有路由、模块必须有真实实现，不留 `status: scaffold` 类占位、不留 `|| fallback`、不留 `|| true` 跳过检查。CI 全绿才可合并。

---

## 9. 里程碑路线

| 阶段 | 内容 | 交付 |
|---|---|---|
| M1 方案与骨架 | 本文档 + 工程骨架 | PLAN.md、目录、CI、规范 |
| M2 MVP 垂直切片 | FR-1/2/3/7 + demo_agent | 可跑通端到端 trace |
| M3 监控与分析 | FR-4/5 | 实时监控页、分析看板 |
| M4 恢复与告警 | FR-6/8 | 重试策略观测、告警 |
| M5 产品化 | 文档完善、部署脚本、Demo 站 | 可上线演示 |

---

## 10. 待确认决策

| 决策点 | 默认方案 | 说明 |
|---|---|---|
| 产品名 | TracePilot | 可改，早期改名成本低 |
| 时序存储 | TimescaleDB | 备选 ClickHouse（更重） |
| 前端 | Next.js + ECharts | 备选 Vue + AntV |
| 多租户 | API Key + Project 隔离 | MVP 不做 SSO |
| 告警通知渠道 | MVP 仅站内 | 后续接邮件/飞书/钉钉 |

> 决策若有异议随时提出；无异议则按默认方案执行。
