# 贡献指南

感谢你关注 TracePilot。欢迎贡献代码、文档、Issue。

## 开发环境

- Python 3.11+，Node 18+，Docker Desktop
- 用 `make` 命令启动，详见 README

## 分支策略

- `main`：可发布状态，保持 green
- `feature/*`：新功能，从 main 切出
- `fix/*`：修复，从 main 切出

## 提交规范（Conventional Commits）

```
feat(ingest): 支持批量 span 批量写入
fix(query): 修复 parent_span_id 根节点解析
docs: 补充 SDK 接入说明
test(storage): 补充 hypertable 分块测试
```

类型：`feat` / `fix` / `docs` / `style` / `refactor` / `perf` / `test` / `chore`。

## 代码规范

- 后端：Ruff + Black + mypy，pre-commit 自动拦截
- 前端：ESLint + Prettier
- 测试：pytest 覆盖率 ≥ 80%；新增功能必须有对应测试

## Pull Request

1. 从 feature 分支切出，标题用 Conventional Commits 格式
2. 描述清楚改了什么、为什么、怎么验证
3. CI 全绿 + 至少一人 review 通过才能合并

## 新增模块（插件化）

模块目录放 `backend/app/modules/<name>/`，实现 `ModuleSpec` 并在 `app/modules/__init__.py` 注册，核心不改。

## 行为准则

见 [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)。
