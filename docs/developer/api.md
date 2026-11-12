# API 参考

所有非健康检查路由需要 API Key 鉴权。

## 鉴权

### REST API

请求头携带：

```
X-TracePilot-Key: <api_key>
```

header 名称可通过 `TRACEPILOT_API_KEY_HEADER` 环境变量配置，默认 `X-TracePilot-Key`。

### WebSocket

浏览器 WebSocket 无法设置自定义请求头，鉴权走 query param：

```
ws://localhost:8000/api/monitor/stream?api_key=<api_key>
```

### 项目隔离

API Key 与项目绑定。每个请求会校验该 Key 对应的项目 ID 与请求路径中的 `project_id` 一致，不匹配返回 401。

## 健康检查

### GET /api/health

无需鉴权。返回服务状态。

```json
{ "status": "ok" }
```

## FR-1 接入（ingest）

### POST /api/ingest/traces

批量上报 trace 与 spans。鉴权后在单事务内写入。

**请求体：**

```json
{
  "trace": {
    "id": "trace-uuid",
    "project_id": "project-uuid",
    "agent_id": "agent-uuid",
    "agent_name": "my-agent",
    "status": "success",
    "started_at": "2026-11-11T10:00:00Z",
    "duration_ms": 1200
  },
  "spans": [
    {
      "id": "span-uuid",
      "trace_id": "trace-uuid",
      "parent_span_id": null,
      "name": "llm.call",
      "kind": "llm",
      "status": "success",
      "started_at": "2026-11-11T10:00:00Z",
      "duration_ms": 800,
      "attributes": { "model": "gpt-4", "attempt": 1 },
      "events": []
    }
  ]
}
```

**响应：**

```json
{ "trace_id": "trace-uuid", "accepted": 2 }
```

## FR-3 查询（query）

### GET /api/query/traces

查询项目下的 trace 列表，支持时间窗口与分页。

**Query 参数：**

| 参数 | 说明 |
|---|---|
| `project_id` | 项目 ID（必填，与 API Key 绑定校验） |
| `after` | 起始时间（ISO 8601，含） |
| `before` | 结束时间（ISO 8601，含） |
| `limit` | 每页条数（默认 50） |
| `offset` | 偏移量 |

**响应：**

```json
{
  "traces": [
    {
      "id": "trace-uuid",
      "project_id": "project-uuid",
      "agent_id": "agent-uuid",
      "agent_name": "my-agent",
      "status": "success",
      "started_at": "2026-11-11T10:00:00Z",
      "duration_ms": 1200
    }
  ],
  "total": 1
}
```

### GET /api/query/traces/:id

查询单条 trace 详情，含 span 树。先鉴权，再按项目过滤，无权访问返回 404。

**路径参数：**

| 参数 | 说明 |
|---|---|
| `id` | trace ID |

**Query 参数：**

| 参数 | 说明 |
|---|---|
| `project_id` | 项目 ID（必填，与 API Key 绑定校验） |

**响应：**

```json
{
  "trace": { "id": "trace-uuid", "status": "success", "...": "..." },
  "spans": [
    {
      "id": "span-uuid",
      "trace_id": "trace-uuid",
      "parent_span_id": null,
      "name": "llm.call",
      "kind": "llm",
      "status": "success",
      "started_at": "2026-11-11T10:00:00Z",
      "duration_ms": 800,
      "attributes": { "model": "gpt-4" },
      "events": []
    }
  ]
}
```

## FR-4 监控（monitor）

### GET /api/monitor/stats

监控统计概览：total / success / error / avg / p50 / p95。空项目返回零值而非报错。

**Query 参数：** `project_id`（必填）

**响应：**

```json
{
  "total": 100,
  "success": 90,
  "error": 10,
  "avg_ms": 1200.5,
  "p50_ms": 900.0,
  "p95_ms": 2100.0
}
```

### GET /api/monitor/agents

按 Agent 聚合的实时状态。

**Query 参数：** `project_id`（必填）

**响应：**

```json
{
  "agents": [
    {
      "agent_id": "agent-uuid",
      "agent_name": "my-agent",
      "total": 50,
      "success": 45,
      "error": 5,
      "avg_ms": 1200.5
    }
  ]
}
```

### GET /api/monitor/stream

WebSocket 实时流。鉴权走 `api_key` query param。

```
ws://localhost:8000/api/monitor/stream?api_key=<api_key>&project_id=<project_id>
```

连接后每 5s 推送一次增量统计。

## FR-5 分析（analytics）

### GET /api/analytics/summary

成功率、延迟分位、span 类型汇总。

**Query 参数：** `project_id`、`after`、`before`（可选）

**响应：**

```json
{
  "total": 100,
  "success_rate": 0.9,
  "avg_ms": 1200.5,
  "p50_ms": 900.0,
  "p95_ms": 2100.0,
  "p99_ms": 3000.0,
  "span_kinds": { "llm": 200, "tool": 150, "agent": 100 }
}
```

### GET /api/analytics/latency

小时级延迟趋势。

**Query 参数：** `project_id`、`after`、`before`（可选）

**响应：**

```json
{
  "series": [
    { "hour": "2026-11-11T10:00:00Z", "avg_ms": 1200.0, "p95_ms": 2100.0 }
  ]
}
```

### GET /api/analytics/cost

按 span 类型的成本拆解。

**Query 参数：** `project_id`、`after`、`before`（可选）

**响应：**

```json
{
  "breakdown": [
    { "kind": "llm", "count": 200, "total_tokens": 50000, "estimated_cost": 12.5 }
  ]
}
```

## FR-6 恢复（recovery）

### GET /api/recovery/stats

恢复观测汇总：错误/重试 span 数、恢复/失败 trace 数。

**Query 参数：** `project_id`、`after`、`before`（可选）

**响应：**

```json
{
  "error_spans": 15,
  "retry_spans": 8,
  "recovered_traces": 5,
  "failed_traces": 3
}
```

### GET /api/recovery/agents

按 Agent 聚合的失败与重试统计。

**Query 参数：** `project_id`、`after`、`before`（可选）

**响应：**

```json
{
  "agents": [
    { "agent_name": "my-agent", "error_spans": 10, "retry_spans": 5, "failed_traces": 2 }
  ]
}
```

### GET /api/recovery/traces

含失败或重试 span 的 trace 列表。

**Query 参数：** `project_id`、`after`、`before`、`limit`、`offset`（可选）

**响应：**

```json
{
  "traces": [
    { "id": "trace-uuid", "status": "failed", "agent_name": "my-agent", "...": "..." }
  ]
}
```

## FR-7 鉴权（iam）

### POST /api/iam/projects

创建新项目，返回明文 API Key（仅此一次，服务端只存 SHA-256 哈希）。

**请求体：**

```json
{ "name": "my-project", "owner": "me@example.com" }
```

**响应：**

```json
{
  "id": "project-uuid",
  "name": "my-project",
  "owner": "me@example.com",
  "api_key": "tp_live_xxxxxxxxxxxxxxxx",
  "created_at": "2026-11-11T10:00:00Z"
}
```

## FR-8 告警（alert）

### GET /api/alert/rules

列出项目全部告警规则（含 `last_triggered_at`）。

**Query 参数：** `project_id`（必填）

**响应：**

```json
{
  "rules": [
    {
      "id": "rule-uuid",
      "project_id": "project-uuid",
      "metric": "error_rate",
      "operator": ">",
      "threshold": 0.1,
      "enabled": true,
      "last_triggered_at": "2026-11-11T10:00:00Z"
    }
  ]
}
```

### POST /api/alert/rules

创建告警规则。

**请求体：**

```json
{
  "project_id": "project-uuid",
  "metric": "error_rate",
  "operator": ">",
  "threshold": 0.1
}
```

支持 metric：`error_rate` / `p95_ms` / `avg_ms`
支持 operator：`<` / `>`

**响应：**

```json
{
  "id": "rule-uuid",
  "project_id": "project-uuid",
  "metric": "error_rate",
  "operator": ">",
  "threshold": 0.1,
  "enabled": true
}
```

### PATCH /api/alert/rules/:id

启用/停用告警规则。

**路径参数：** `id` — 规则 ID

**请求体：**

```json
{ "project_id": "project-uuid", "enabled": false }
```

**响应：**

```json
{ "updated": true, "enabled": false }
```

### DELETE /api/alert/rules/:id

删除告警规则。

**路径参数：** `id` — 规则 ID

**Query 参数：** `project_id`（必填）

**响应：** `204 No Content`

### GET /api/alert/events

列出告警触发事件。

**Query 参数：** `project_id`、`limit`、`offset`（可选）

**响应：**

```json
{
  "events": [
    {
      "id": "event-uuid",
      "rule_id": "rule-uuid",
      "triggered_at": "2026-11-11T10:00:00Z",
      "value": 0.15
    }
  ]
}
```

## 错误响应

统一 JSON 格式：

```json
{ "error": "message" }
```

| 状态码 | 含义 |
|---|---|
| 400 | 请求参数错误 |
| 401 | 鉴权失败（API Key 无效或项目不匹配） |
| 404 | 资源不存在或无权访问 |
| 500 | 服务内部错误（不泄露数据库细节） |
