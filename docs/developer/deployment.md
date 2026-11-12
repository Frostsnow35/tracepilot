# 部署

## 环境要求

| 依赖 | 版本 |
|---|---|
| Docker | 24+ |
| Docker Compose | v2（`docker compose` 子命令） |
| 内存 | ≥ 2GB |

## 一键部署

```bash
./scripts/deploy.sh
```

脚本执行流程：

1. `docker compose build --no-cache` — 构建后端与前端镜像
2. `docker compose up -d` — 启动 db / backend / frontend
3. 轮询后端 `/api/health`（最多 90s）
4. 轮询前端（最多 60s）
5. 输出访问地址

**停止：**

```bash
./scripts/deploy.sh --down
```

保留数据卷；如需彻底清除数据：`docker compose down -v`

**强制重建镜像：**

```bash
./scripts/deploy.sh --rebuild
```

## 服务与端口

| 服务 | 容器 | 端口 | 说明 |
|---|---|---|---|
| db | tracepilot-db | 5432 | PostgreSQL 16 + TimescaleDB |
| backend | tracepilot-backend | 8000 | Rust/Axum API |
| frontend | tracepilot-frontend | 3000 | Next.js 静态站点 |

依赖顺序：`db` → `backend` → `frontend`，`db` 有 healthcheck 门控。

## 环境变量

复制 `.env.example` 为 `.env` 并按需修改，`docker compose` 自动读取。

| 变量 | 默认值 | 说明 |
|---|---|---|
| `POSTGRES_DB` | `tracepilot` | 数据库名 |
| `POSTGRES_USER` | `tracepilot` | 数据库用户 |
| `POSTGRES_PASSWORD` | `tracepilot` | 数据库密码 |
| `TRACEPILOT_DATABASE_URL` | 容器内自动拼接 | 后端数据库连接串 |
| `TRACEPILOT_ENV` | `dev` | 运行环境标识 |
| `TRACEPILOT_APP_NAME` | `TracePilot` | 应用名 |
| `TRACEPILOT_API_KEY_HEADER` | `x-tracepilot-key` | API Key 请求头名 |
| `TRACEPILOT_INGEST_BATCH_MAX` | `500` | 接入批量上限 |
| `TRACEPILOT_PORT` | `8000` | 后端监听端口 |
| `NEXT_PUBLIC_API_BASE` | `http://localhost:8000` | 前端 API 基地址 |

## 数据库迁移

后端启动时自动执行 migration（`sqlx::migrate!`），无需手动干预。迁移文件位于 `backend/migrations/`。

| 迁移 | 内容 |
|---|---|
| 0001 | 创建 projects / traces / spans 表，spans 建 hypertable |
| 0002 | traces 加 `recorded_at` 列（monitor 水位线用） |
| 0003 | 创建 alert_rules / alert_events 表 |

## 灌入演示数据

```bash
python scripts/seed_demo.py
```

生成 4 场景（正常 / 重试恢复 / 失败 / 多步骤）× 3 个 Agent 的演示 trace，自动创建 `demo-project` 并写入。

## 本地开发

### 后端

```bash
cd backend
# 需要本地 PostgreSQL + TimescaleDB
export TRACEPILOT_DATABASE_URL=postgresql://user:pass@localhost:5432/tracepilot
cargo run
```

### 前端

```bash
cd frontend
export NEXT_PUBLIC_API_BASE=http://localhost:8000
npm install
npm run dev
```

## 验证部署

```bash
# 后端健康检查
curl http://localhost:8000/api/health

# 创建项目
curl -X POST http://localhost:8000/api/iam/projects \
  -H "Content-Type: application/json" \
  -d '{"name":"verify"}'

# 用返回的 api_key 接入一条 trace
curl -X POST http://localhost:8000/api/ingest/traces \
  -H "X-TracePilot-Key: <api_key>" \
  -H "Content-Type: application/json" \
  -d '{
    "trace": {"id":"t1","project_id":"<project_id>","status":"success","started_at":"2026-11-12T10:00:00Z","duration_ms":100},
    "spans": []
  }'
```
