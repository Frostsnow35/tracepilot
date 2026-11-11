# TracePilot

我们正在做一个 Agent 运行时可观测平台 —— 让 AI Agent 像后端服务一样可监控、可排查、可优化，提升对运行时的观测的可审计性。

## 特性

- **结构化 Trace**：OpenTelemetry 兼容，LLM / tool / agent / retrieval / chain 全链路 span
- **瀑布图可视化**：trace 详情逐 span 展开，失败标红、重试可见
- **实时监控**：Agent 成功率、延迟、错误率 WebSocket 实时推送
- **分析洞察**：成功率 / 延迟分位 / 成本归因
- **恢复策略观测**：重试、回退、上下文压缩一目了然
- **自托管轻量**：Docker Compose 一键起，数据不出本机
- **Rust 后端**：Axum + tokio + sqlx，编译期内存安全，高吞吐接入

## 技术栈

| 层 | 选型 |
|---|---|
| 后端 | Rust + Axum + tokio + sqlx + tracing |
| 存储 | PostgreSQL 15 + TimescaleDB |
| 前端 | Next.js 14 + React + TS + Tailwind + ECharts |
| SDK | Python / TypeScript / Rust（OTel 兼容） |

## 快速开始

```bash
# 1. 启动数据库（Postgres + TimescaleDB）
docker compose up -d db

# 2. 启动后端（Rust）
cd backend && cargo run

# 3. 启动前端
cd frontend && npm install && npm run dev

# 4. 跑示例 Agent，产生真实 trace
python examples/demo_agent/run.py
```

打开 http://localhost:3000 查看 trace。

## 架构

```
Agent(SDK) --OTel--> trace-ingest(Axum) --> storage(PG+TimescaleDB)
                                          --> query/monitor/analytics/recovery
                                          --> WebSocket + REST --> Next.js 前端
```

详见 [docs/PLAN.md](docs/PLAN.md)。

## 模块

| 模块 | 职责 | 目录 |
|---|---|---|
| trace-ingest | 数据接入 | `backend/crates/api/src/modules/ingest` |
| storage | 存储 | `backend/crates/api/src/modules/storage` |
| trace-query | 查询 | `backend/crates/api/src/modules/query` |
| monitor | 实时监控 | （M3 接入） |
| analytics | 分析洞察 | （M3 接入） |
| recovery | 恢复策略 | （M4 接入） |
| iam | 鉴权 | `backend/crates/api/src/modules/iam` |
| alert | 告警 | （M4 接入） |

模块插件化：每模块实现 `Module` trait，在 `modules/mod.rs` registry 登记一行即挂载，核心 `main.rs` 不改。

## 开发规范

- 后端：`cargo fmt` + `cargo clippy -D warnings` + `cargo test`，pre-commit 拦截
- 前端：ESLint + Prettier
- 提交：Conventional Commits
- 分支：`main` + `feature/*` + `fix/*`

详见 [CONTRIBUTING.md](CONTRIBUTING.md)。

## License

Apache-2.0，见 [LICENSE](LICENSE)。
