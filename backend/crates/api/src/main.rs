mod error;
mod modules;
mod state;

use axum::Router;
use tower_http::trace::TraceLayer;
use tracepilot_core::Config;

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

    let storage = modules::storage::Storage::new(pool);
    let state = AppState {
        config: config.clone(),
        storage,
    };

    let mut app: Router<AppState> = Router::new();
    app = app.route("/api/health", axum::routing::get(|| async { "ok" }));
    for module in modules::registry() {
        app = app.merge(module.router());
    }
    let app = app.layer(TraceLayer::new_for_http()).with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    tracing::info!(addr = %listener.local_addr()?, "listening");
    axum::serve(listener, app).await?;
    Ok(())
}
