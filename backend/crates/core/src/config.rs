use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app_name: String,
    pub env: String,
    pub database_url: String,
    pub api_key_header: String,
    pub ingest_batch_max: usize,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        let get = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.to_string());
        Self {
            app_name: get("TRACEPILOT_APP_NAME", "TracePilot"),
            env: get("TRACEPILOT_ENV", "dev"),
            database_url: get(
                "TRACEPILOT_DATABASE_URL",
                "postgresql://tracepilot:tracepilot@localhost:5432/tracepilot",
            ),
            api_key_header: get("TRACEPILOT_API_KEY_HEADER", "x-tracepilot-key"),
            ingest_batch_max: std::env::var("TRACEPILOT_INGEST_BATCH_MAX")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(500),
            port: std::env::var("TRACEPILOT_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8000),
        }
    }
}
