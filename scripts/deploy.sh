#!/usr/bin/env bash
# TracePilot 一键部署脚本：构建镜像 → 启动全部服务 → 等待健康检查 → 跑迁移。
# 用法：./scripts/deploy.sh [--down] [--rebuild]
#   --down      停止并移除容器（保留数据卷）
#   --rebuild   强制重建镜像（默认仅当镜像不存在时构建）

set -euo pipefail

cd "$(dirname "$0")/.."
ROOT="$(pwd)"

if [[ "${1:-}" == "--down" ]]; then
    echo "[deploy] stopping services..."
    docker compose down
    echo "[deploy] done (volumes preserved). Use 'docker compose down -v' to remove data."
    exit 0
fi

REBUILD="${2:-}"

echo "[deploy] building images..."
docker compose build --no-cache

echo "[deploy] starting services..."
docker compose up -d

echo "[deploy] waiting for backend health..."
for i in $(seq 1 30); do
    if curl -sf http://localhost:8000/api/health >/dev/null 2>&1; then
        echo "[deploy] backend healthy"
        break
    fi
    if [[ $i -eq 30 ]]; then
        echo "[deploy] ERROR: backend did not become healthy in 90s" >&2
        docker compose logs --tail=50 backend
        exit 1
    fi
    sleep 3
done

echo "[deploy] waiting for frontend..."
for i in $(seq 1 20); do
    if curl -sf http://localhost:3000 >/dev/null 2>&1; then
        echo "[deploy] frontend healthy"
        break
    fi
    if [[ $i -eq 20 ]]; then
        echo "[deploy] WARNING: frontend not reachable yet (build may still be running)"
    fi
    sleep 3
done

echo ""
echo "[deploy] TracePilot is up:"
echo "  frontend: http://localhost:3000"
echo "  backend:  http://localhost:8000/api/health"
echo "  db:       localhost:5432 (tracepilot/tracepilot/tracepilot)"
echo ""
echo "[deploy] next: run examples/demo_agent/run.py to produce a demo trace."
