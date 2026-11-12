# AGENTS.md — TracePilot 开源贡献规范

本文件定义在 TracePilot 仓库中工作（含 AI 代理与人类贡献者）必须遵守的规则。进入本仓库前先读本文件；规则与 PLAN.md §8 工程规范保持一致，冲突时以本文件为执行口径。

## 项目概述

TracePilot 是 Agent 运行时可观测平台：接入、存储、查看 Agent 运行时的执行轨迹，定位为"Agent 的 Datadog"。

- 后端：Rust + Axum + sqlx + PostgreSQL/TimescaleDB（Cargo workspace：`backend/crates/core` + `backend/crates/api`）
- 前端：Next.js 14 + React + TypeScript + Tailwind + ECharts
- SDK：Python + TypeScript，OpenTelemetry 兼容
- 部署：Docker Compose（MVP）

## 目录结构

| 路径 | 用途 |
|---|---|
| `backend/crates/core` | 配置、`Module` trait、共享类型 |
| `backend/crates/api/src/modules/*` | 一模块一目录，各自 `mod.rs` 含 router / service / repo |
| `backend/migrations` | SQL 迁移 |
| `frontend/src/app` | 按页面组织路由 |
| `frontend/src/lib` | 按领域组织工具与 API 封装 |
| `sdk/` | Python / TypeScript SDK |
| `examples/` | demo_agent 与端到端验证脚本 |
| `scripts/` | 部署、演示数据脚本 |
| `docs/` | PLAN.md |

## 工程规范

### 代码风格

- Rust：`cargo fmt` + `clippy -D warnings`，CI 阻断，不允许任何 clippy 警告。
- TypeScript：遵循既有 `eslint` / `prettier` 配置，不引入新的 lint 规则除非有明确理由。
- 行尾：仓库检出为 CRLF，库内存储 LF，不要"修"行尾。

### 禁止占位与降级

所有路由、模块必须有真实实现。明确禁止：

- `status: scaffold` 类占位
- `|| fallback` 降级兜底
- `|| true` 跳过检查
- `TODO` / `FIXME` 残留（除确属待确认决策且已在 PLAN.md §10 登记）

提交前自查：`grep -rn "scaffold\|TODO\|FIXME\|placeholder\||| true\||| fallback" backend/crates frontend/src sdk/`

### 模块插件化

新增后端模块：

1. 实现 `Module` trait（`name` / `depends_on` / `router`）
2. 在 registry 登记一行
3. 核心 `main.rs` 不改

模块依赖单向：`iam` 为根，`storage` 次之，其余模块只依赖 `storage` 与 `iam`，无横向依赖。

### 数据库

- 迁移用 `sqlx::migrate!` 自动执行，新增迁移文件放 `backend/migrations/`，序号递增。
- spans 是 TimescaleDB hypertable，按 `recorded_at` 分区；唯一索引必须含分区列（TS103）。
- 聚合查询注意 NULL 安全：空项目下 `avg()` / `percentile_cont()` 返回 NULL，必须用 `Option<f64>` + `unwrap_or(0.0)` 解码。
- JSONB 字段做类型转换前先正则守卫（如 `attributes->>'attempt' ~ '^[0-9]+$'`），避免 cast 报错。

## 鉴权与项目隔离

- 所有非健康检查 REST 路由经 API Key 鉴权（默认头 `X-TracePilot-Key`，可配置）。
- API Key 与项目绑定，请求需校验路径中的 `project_id` 与 Key 对应项目一致，不匹配返回 401。
- 查询类接口先鉴权、再按项目过滤，禁止先查库后鉴权（避免未授权访问与存在性枚举）。
- WebSocket 鉴权走 `api_key` query param（浏览器 WS 无法设自定义头）。
- 服务端只存 API Key 的 SHA-256 哈希，不存明文。

## 事务与数据一致性

- trace + spans 写入用单事务（`insert_trace_with_spans`），避免半成品。
- 告警事件 + 冷却更新用单事务（`record_alert_event`）。
- 监控轮询用水位线模式：查询前捕获 `upper = now()` 作上界，避免跳过查询期间写入的 trace。

## 提交与分支

- 分支命名：`feat/<模块>-<简述>`、`fix/<简述>`、`docs/<简述>`、`refactor/<简述>`。
- 提交信息用中文，格式：`<type>: <简述>`，type 取 `feat` / `fix` / `docs` / `refactor` / `test` / `ci`。
- 一个提交做一件事；不要在提交里混入无关改动。

## 质量门禁（合并前必须全绿）

| 检查 | 命令 | 要求 |
|---|---|---|
| 格式 | `cargo fmt` | 无差异 |
| 静态检查 | `cargo clippy -D warnings` | 零警告 |
| 测试 | `cargo test` | 全部通过 |
| 前端构建 | `npm run build` | 无错误 |
| CI | GitHub Actions | 全绿才可合并 |

推送前在本地跑上述检查；CI 失败时修复后重跑，不要用 `--no-verify` 跳过钩子。

## PR 流程

1. 从 `main` 切分支，完成改动。
2. 本地跑质量门禁，全绿。
3. 开 PR，填写模板（变更概述、关联 Issue、验证方式、检查清单）。
4. 至少一个 review 通过且 CI 全绿后合并。
5. 合并用 squash merge，保持 `main` 历史线性。

## 文档

- README（EN + zh-CN）、CHANGELOG 随功能变更同步更新。
- PLAN.md 是方案与规格的权威来源，功能需求变更需先改 PLAN.md。
- 不在仓库中放置开发文档或提示词类内容；仓库只放可执行的代码、配置与面向用户/贡献者的规范文档。
