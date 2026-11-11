# Changelog

本项目遵循 [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) 与 [语义化版本](https://semver.org/lang/zh-CN/)。

## [Unreleased]

## [0.1.0] - 2026-11-11

### Added — M1 方案与骨架
- 方案与规格文档（docs/PLAN.md）：产品定位、FR-1~FR-8 模块对应表、技术选型、架构、数据模型、里程碑
- 工程骨架：目录规范、GitHub Actions CI、pre-commit、docker-compose、Makefile、LICENSE(Apache-2.0)
- Rust 后端：Cargo workspace（core + api），Axum + tokio + sqlx + tracing
- 模块插件化：`Module` trait（name/depends_on/router）+ registry 拓扑序挂载，新增模块登记一行即接入

### Added — M2 MVP 垂直切片
- FR-1 接入：ingest 模块 + Python SDK（OTel 兼容模型）
- FR-2 存储：migration 0001（projects/traces/spans，spans 为 TimescaleDB hypertable）
- FR-3 查询：trace 列表 + 详情（span 树）+ 前端瀑布图（ECharts）
- FR-7 鉴权：iam 模块（API Key 生成 + SHA-256 哈希存储 + 项目隔离）
- 示例：examples/demo_agent 产出含 LLM/tool 调用、失败+重试的真实 trace

### Added — M3 监控与分析
- FR-4 monitor：统计概览（total/success/error/avg/p50/p95）、按 Agent 聚合、WebSocket 实时流
  （broadcast channel + 5s 后台轮询水位线；WS 鉴权走 query api_key）
- FR-5 analytics：成功率/延迟分位/span 类型汇总、小时级延迟趋势、按 span 类型的成本拆解
- migration 0002：traces 加 recorded_at 列（monitor 轮询水位线用）
- 前端：/monitor（WS 实时流 + 轮询）、/analytics（ECharts 折线 + 饼图）

### Added — M4 恢复与告警
- FR-6 recovery：恢复观测汇总（错误/重试 span 数、恢复/失败 trace 数）、按 Agent 聚合、
  含失败或重试的 trace 列表；重试判定 `attributes->>'attempt' > 1`（正则守卫容错）
- FR-8 alert：规则 CRUD（metric ∈ error_rate/p95_ms/avg_ms，operator ∈ </>，threshold ≥ 0）、
  启用/停用（PATCH）、30s 后台评估、10min 冷却（last_triggered_at）
- migration 0003：alert_rules / alert_events（DOUBLE PRECISION，级联删除）
- 前端：/recovery、/alerts 页面
- 端到端验证脚本：examples/e2e_verify.py、examples/ws_verify.py、examples/m4_verify.py

### Fixed — 工程质量审查（M5 前）
- 监控水位线竞态：查询前捕获 now 作上界，避免窗口内 trace 被跳过
- 后台任务 DB 错误静默吞掉：补 tracing::warn!
- 内部 DB 错误细节泄露客户端：对外返回通用 "internal server error"
- query 详情先查库后鉴权：改为先鉴权再按项目过滤，消除未授权访问与存在性枚举
- alert 缺启停接口：新增 PATCH /rules/{id}
- record_alert_event 两条写非原子：事务包裹
- ingest trace+spans 中途失败留半成品：事务包裹（insert_trace_with_spans）
- evaluate_alerts N+1 查询：enabled_alert_rules 一次查出 last_triggered_at
- analytics/monitor/recovery 项目隔离：authenticate → authorize（按 project_id 校验）

### Changed
- 后端由 Python/FastAPI 迁移至 Rust/Axum（应需求：体现技术深度、内存安全、高吞吐）
- 数据库镜像切换为 timescale/timescaledb:latest-pg16
- 模块挂载从 Router::merge 改为 nest("/api/<name>")，与 SDK/前端调用路径对齐
- spans 主键改为复合键 (id, recorded_at)（TS103：hypertable 唯一索引必须含分区列）
- 聚合 SQL 类型转换：avg()::float8、sum()::bigint、date_trunc()::text（PG NUMERIC/TIMESTAMP 与 Rust 类型对齐）

[0.1.0]: https://github.com/Frostsnow35/tracepilot/releases/tag/v0.1.0
