# SDK 接入

TracePilot 提供 Python SDK，API 与 OpenTelemetry 兼容，第三方 Agent 框架可零改接入。

## Python SDK

### 安装

```bash
cd sdk/python
pip install -e .
```

### 快速开始

```python
from tracepilot_sdk import TracePilotClient, Trace, Span

client = TracePilotClient(
    base_url="http://localhost:8000",
    api_key="tp_live_xxxxxxxxxxxxxxxx",
)

# 上报一条 trace
trace = Trace(
    id="trace-001",
    project_id="project-uuid",
    agent_id="agent-001",
    status="success",
    started_at="2026-11-11T10:00:00Z",
    duration_ms=1200,
)

spans = [
    Span(
        id="span-001",
        trace_id="trace-001",
        name="llm.call",
        kind="llm",
        status="success",
        started_at="2026-11-11T10:00:00Z",
        duration_ms=800,
        attributes={"model": "gpt-4", "attempt": 1},
    ),
    Span(
        id="span-002",
        trace_id="trace-001",
        parent_span_id="span-001",
        name="tool.search",
        kind="tool",
        status="success",
        started_at="2026-11-11T10:00:01Z",
        duration_ms=200,
        attributes={"tool": "web_search"},
    ),
]

client.ingest(trace, spans)
```

### 创建项目

```python
project = client.create_project("my-project")
# 返回 {id, name, api_key, ...}，api_key 仅此一次可见
```

### 查询

```python
# trace 列表
traces = client.list_traces(project_id="project-uuid")

# 单条 trace 详情
detail = client.get_trace(trace_id="trace-001")

# 监控
stats = client.monitor_stats(project_id="project-uuid")
agents = client.monitor_agents(project_id="project-uuid")

# 分析
summary = client.analytics_summary(project_id="project-uuid")
latency = client.analytics_latency(project_id="project-uuid")
cost = client.analytics_cost(project_id="project-uuid")

# 恢复
rstats = client.recovery_stats(project_id="project-uuid")
ragents = client.recovery_agents(project_id="project-uuid")
rtraces = client.recovery_traces(project_id="project-uuid")

# 告警
rules = client.alert_rules(project_id="project-uuid")
client.create_alert_rule("project-uuid", "error_rate", ">", 0.1)
client.update_alert_rule("project-uuid", "rule-uuid", enabled=False)
client.delete_alert_rule("project-uuid", "rule-uuid")
events = client.alert_events(project_id="project-uuid")
```

## Span Kind

SDK 支持的 span 类型：

| kind | 说明 |
|---|---|
| `llm` | LLM 调用 |
| `tool` | 工具调用 |
| `agent` | Agent 步骤 |
| `retrieval` | 检索 |
| `chain` | 链式调用 |
| `custom` | 自定义 |

## OTel 兼容

SDK 的数据模型与 OpenTelemetry 的 trace/span 结构对齐，已有 OTel 采集逻辑的 Agent 可直接映射字段接入，无需改造采集链路。

## 重试与恢复观测

SDK 不强制重试逻辑，但支持在 span 的 `attributes` 中记录重试信息：

```python
Span(
    id="span-003",
    trace_id="trace-001",
    name="llm.retry",
    kind="llm",
    status="success",
    started_at="2026-11-11T10:00:02Z",
    duration_ms=900,
    attributes={"model": "gpt-4", "attempt": 2},  # attempt > 1 即视为重试
)
```

后端 recovery 模块通过 `attributes->>'attempt' > 1` 判定重试 span（正则守卫容错，避免 JSONB 字段非数字时报错）。
