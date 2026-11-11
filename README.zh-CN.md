# TracePilot

Agent 运行时可观测平台：接入、存储、查看 Agent 运行时的执行轨迹。

[English](README.md)

## 功能

| 功能 | 状态 |
|---|---|
| 结构化 Trace 接入（span 模型遵循 OTel 语义：trace_id / parent_span_id / kind / status / duration_ms / attributes） | 已完成 |
| TimescaleDB hypertable 存储 | 已完成 |
| Trace 查询 + 瀑布图可视化 | 已完成 |
| 项目与 API Key 管理 | 已完成 |

## 技术栈

| 层 | 选型 |
|---|---|
| 后端 | Rust + Axum + tokio + sqlx |
| 存储 | PostgreSQL 15 + TimescaleDB |
| 前端 | Next.js 14 + React + TypeScript + Tailwind + ECharts |
| SDK | Python |

## 快速开始

```bash
# 1. 启动数据库（Postgres + TimescaleDB）
docker compose up -d db

# 2. 启动后端（默认数据库地址与 compose 的 db 服务一致）
cd backend && cargo run

# 3. 启动前端
cd frontend && npm install && npm run dev

# 4. 运行示例 Agent，产生一条真实 trace
python examples/demo_agent/run.py
```

打开 http://localhost:3000 查看。

## 架构

```
Agent(SDK) --> ingest(Axum) --> storage(PG+TimescaleDB)
                               --> query --> Next.js 前端（瀑布图）
```

完整规格见 [docs/PLAN.md](docs/PLAN.md)。

## 模块

| 模块 | 职责 | 目录 | 状态 |
|---|---|---|---|
| ingest | 数据接入 | `backend/crates/api/src/modules/ingest` | 已完成 |
| storage | 数据层 | `backend/crates/api/src/modules/storage` | 已完成 |
| query | 查询 | `backend/crates/api/src/modules/query` | 已完成 |
| iam | 项目与 API Key | `backend/crates/api/src/modules/iam` | 已完成 |
| monitor | 实时指标 | — | 规划中 |
| analytics | 成功率 / 延迟 / 成本 | — | 规划中 |
| recovery | 重试 / 回退观测 | — | 规划中 |
| alert | 告警 | — | 规划中 |

模块插件化：每个模块实现 `Module` trait，在 `modules/mod.rs` 的 registry 登记一行即挂载。`main.rs` 按拓扑序把模块挂载到 `/api/<模块名>` 下，核心代码不改。

## API

| 方法 | 路径 | 用途 |
|---|---|---|
| POST | `/api/iam/projects` | 创建项目，返回 api_key |
| POST | `/api/ingest/traces` | 接入一条 trace 及其 spans |
| GET | `/api/query/traces` | 列出 traces（`project_id`、`limit`） |
| GET | `/api/query/traces/{id}` | trace 详情（含 spans） |
| GET | `/api/health` | 健康检查 |

鉴权：除健康检查外，所有路由需要 `X-TracePilot-Key: <api_key>` 请求头。

## 开发规范

- 后端：`cargo fmt` + `cargo clippy -D warnings` + `cargo test`
- 前端：ESLint + Prettier
- 提交：Conventional Commits
- 分支：`main` + `feature/*` + `fix/*`

详见 [CONTRIBUTING.md](CONTRIBUTING.md)。

## License

Apache-2.0，见 [LICENSE](LICENSE)。
