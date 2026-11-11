# Changelog

本项目遵循 [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) 与 [语义化版本](https://semver.org/lang/zh-CN/)。

## [Unreleased]

### Added
- 项目初始化，方案与规格文档（docs/PLAN.md）
- 工程骨架：目录规范、CI、pre-commit、docker-compose、Makefile
- Rust 后端：Cargo workspace（core + api），Axum + tokio + sqlx + tracing
- 模块插件化：`Module` trait + registry 拓扑序挂载，核心 main 不改
- MVP 模块实现：ingest（接入）、storage（PG+TimescaleDB）、query（查询）、iam（鉴权）
- 数据库迁移：projects / traces / spans（spans 为 TimescaleDB hypertable）
- 工程规范：cargo fmt + clippy -D warnings + cargo test，pre-commit 拦截，无占位/降级代码

### Changed
- 后端由 Python/FastAPI 迁移至 Rust/Axum（应需求：体现技术深度、内存安全、高吞吐）
- 数据库镜像切换为 timescale/timescaledb:pg16
