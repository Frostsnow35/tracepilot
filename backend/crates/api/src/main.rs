mod error;
mod modules;
mod state;

use axum::Router;
use tokio::sync::broadcast;
use tower_http::trace::TraceLayer;
use tracepilot_core::Config;

use crate::modules::monitor::TraceEvent;
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tracepilot_api=info,tower_http=info".into()),
        )
        .init();

    let config = Config::from_env();
    tracing::info!(app = %config.app_name, env = %config.env, "starting");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("../../migrations").run(&pool).await?;
    tracing::info!("migrations applied");

    let storage = modules::storage::Storage::new(pool.clone());

    // 监控广播通道：后台轮询任务写入，WebSocket 客户端订阅。
    let (monitor_tx, _monitor_rx) = broadcast::channel::<TraceEvent>(256);

    let state = AppState {
        config: config.clone(),
        storage: storage.clone(),
        monitor_tx,
    };

    // 启动后台轮询：每 5 秒扫一次新 trace，广播给 WebSocket 客户端。
    {
        let storage = storage.clone();
        let monitor_tx = state.monitor_tx.clone();
        let pool = pool.clone();
        tokio::spawn(async move {
            let mut last_seen = chrono::Utc::now();
            loop {
                // 列出所有 project_id，逐个扫描新 trace。
                let projects: Vec<String> =
                    sqlx::query_scalar("SELECT id FROM projects ORDER BY created_at")
                        .fetch_all(&pool)
                        .await
                        .unwrap_or_default();

                for project_id in projects {
                    if let Ok(traces) = storage.list_traces_since(&project_id, last_seen, 200).await
                    {
                        for t in traces {
                            let _ = monitor_tx.send(TraceEvent {
                                trace_id: t.id,
                                project_id: t.project_id,
                                agent_id: t.agent_id,
                                status: t.status,
                                started_at: t.started_at,
                                duration_ms: t.duration_ms,
                            });
                        }
                    }
                }
                last_seen = chrono::Utc::now();
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        });
    }

    // 启动后台告警评估：每 30 秒扫一次启用的规则，超阈值记事件（带冷却）。
    {
        let storage = storage.clone();
        let pool = pool.clone();
        tokio::spawn(async move {
            loop {
                let projects: Vec<String> =
                    sqlx::query_scalar("SELECT id FROM projects ORDER BY created_at")
                        .fetch_all(&pool)
                        .await
                        .unwrap_or_default();

                for project_id in projects {
                    if let Err(e) = storage
                        .evaluate_alerts(
                            &project_id,
                            chrono::Utc::now(),
                            chrono::Duration::minutes(10),
                        )
                        .await
                    {
                        tracing::warn!(project_id = %project_id, error = %e, "alert evaluation failed");
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            }
        });
    }

    let mut app: Router<AppState> = Router::new();
    app = app.route("/api/health", axum::routing::get(|| async { "ok" }));
    for module in modules::registry() {
        // 模块路由挂载在 /api/<模块名> 下，与 SDK / 前端约定的调用路径一致。
        let prefix = format!("/api/{}", module.name());
        app = app.nest(&prefix, module.router());
    }
    let app = app.layer(TraceLayer::new_for_http()).with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    tracing::info!(addr = %listener.local_addr()?, "listening");
    axum::serve(listener, app).await?;
    Ok(())
}
