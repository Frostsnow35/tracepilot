# TracePilot

Agent runtime observability: ingest, store, and inspect what an AI agent does at runtime.

[简体中文](README.zh-CN.md)

## What it does

TracePilot ingests OpenTelemetry-compatible traces from AI agents, stores them in TimescaleDB, and renders them as a waterfall in the browser.

## Implemented

| Feature | Status |
|---|---|
| Structured trace ingest (span model follows OTel semantics: trace_id / parent_span_id / kind / status / duration_ms / attributes) | Done |
| TimescaleDB hypertable storage | Done |
| Trace query + waterfall visualization | Done |
| Project + API key management | Done |
| Realtime monitor (stat overview, per-agent stats, WebSocket live stream) | Done |
| Analytics (success rate, latency trend, cost breakdown) | Done |

## Tech stack

| Layer | Choice |
|---|---|
| Backend | Rust + Axum + tokio + sqlx |
| Storage | PostgreSQL 15 + TimescaleDB |
| Frontend | Next.js 14 + React + TypeScript + Tailwind + ECharts |
| SDK | Python |

## Quick start

```bash
# 1. Start the database (Postgres + TimescaleDB)
docker compose up -d db

# 2. Start the backend (default DB URL matches the compose db service)
cd backend && cargo run

# 3. Start the frontend
cd frontend && npm install && npm run dev

# 4. Run the demo agent to produce a real trace
python examples/demo_agent/run.py
```

Open http://localhost:3000.

## Architecture

```
Agent(SDK) --> ingest(Axum) --> storage(PG+TimescaleDB)
                               --> query --> Next.js frontend (waterfall)
```

See [docs/PLAN.md](docs/PLAN.md) for the full spec.

## Modules

| Module | Role | Path | Status |
|---|---|---|---|
| ingest | trace intake | `backend/crates/api/src/modules/ingest` | Done |
| storage | data layer | `backend/crates/api/src/modules/storage` | Done |
| query | trace lookup | `backend/crates/api/src/modules/query` | Done |
| iam | projects + API keys | `backend/crates/api/src/modules/iam` | Done |
| monitor | realtime metrics | `backend/crates/api/src/modules/monitor` | Done |
| analytics | success / latency / cost | `backend/crates/api/src/modules/analytics` | Done |
| recovery | retry / fallback observation | — | Roadmap |
| alert | alerting | — | Roadmap |

Modules are plugins: each implements the `Module` trait and registers one line in `modules/mod.rs`. `main.rs` mounts them in topological order under `/api/<name>`.

## API

| Method | Path | Purpose |
|---|---|---|
| POST | `/api/iam/projects` | Create project, returns api_key |
| POST | `/api/ingest/traces` | Ingest a trace + spans |
| GET | `/api/query/traces` | List traces (`project_id`, `limit`) |
| GET | `/api/query/traces/{id}` | Trace detail with spans |
| GET | `/api/monitor/stats` | Stats overview (`project_id`) |
| GET | `/api/monitor/agents` | Per-agent stats (`project_id`) |
| GET | `/api/monitor/stream` | WebSocket live stream (`project_id`, `api_key`) |
| GET | `/api/analytics/summary` | Success rate + latency percentiles + kind stats (`project_id`) |
| GET | `/api/analytics/latency` | Hourly latency timeseries (`project_id`) |
| GET | `/api/analytics/cost` | Cost breakdown by span kind (`project_id`) |
| GET | `/api/health` | Health check |

Auth: `X-TracePilot-Key: <api_key>` header on all non-health routes. WebSocket auth goes through the `api_key` query param because browsers cannot set custom headers on WS handshakes.

## Development

- Backend: `cargo fmt` + `cargo clippy -D warnings` + `cargo test`
- Frontend: ESLint + Prettier
- Commits: Conventional Commits
- Branches: `main` + `feature/*` + `fix/*`

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Apache-2.0, see [LICENSE](LICENSE).
