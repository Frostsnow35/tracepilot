# TracePilot 开发者文档

TracePilot 是 Agent 运行时可观测平台——接入、存储、查看 Agent 运行时的执行轨迹，定位为"Agent 的 Datadog"。

## 文档导航

| 文档 | 内容 |
|---|---|
| [架构](architecture.md) | 系统分层、模块依赖、数据流、数据模型 |
| [API 参考](api.md) | 全部 REST 端点与 WebSocket 接口，含鉴权说明 |
| [SDK 接入](sdk.md) | Python / TypeScript SDK 使用与 OTel 兼容说明 |
| [部署](deployment.md) | Docker Compose 一键部署、环境变量、演示数据 |

## 快速开始

```bash
# 1. 启动全部服务（后端 + 前端 + 数据库）
make deploy

# 2. 灌入演示数据
make seed

# 3. 访问
#   前端: http://localhost:3000
#   后端: http://localhost:8000
```

## 项目结构

```
tracepilot/
├── backend/          # Rust + Axum 后端（Cargo workspace）
│   ├── crates/core/  # 配置、模块 trait、共享类型
│   └── crates/api/   # Axum 服务，src/modules/* 一模块一目录
├── frontend/         # Next.js 14 + React + TS + Tailwind + ECharts
├── sdk/              # Python / TypeScript SDK
├── examples/         # demo_agent 与端到端验证脚本
├── scripts/          # deploy.sh、seed_demo.py
├── docs/             # PLAN.md 与开发者文档
└── docker-compose.yml
```
