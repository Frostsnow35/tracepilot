---
name: Bug report
about: 报告 TracePilot 的异常行为或错误
title: "[Bug]: "
labels: bug
assignees: ""
---

## 环境信息

- **TracePilot 版本**：(git commit 或 tag)
- **部署方式**：Docker Compose / 本地开发 / 其他
- **数据库**：PostgreSQL + TimescaleDB 版本
- **操作系统**：

## 问题描述

清晰描述问题是什么，包括：
- 期望行为
- 实际行为
- 复现条件

## 复现步骤

1. 启动环境（命令/配置）
2. 执行操作（API 调用 / SDK 调用 / 前端操作）
3. 观察到的结果

## 日志与证据

粘贴相关日志（后端 tracing 输出、前端控制台、API 响应等）。

```
<日志内容>
```

## 影响范围

- 是否阻塞开发/演示？
- 影响哪些模块（ingest / query / monitor / analytics / recovery / iam / alert）？
