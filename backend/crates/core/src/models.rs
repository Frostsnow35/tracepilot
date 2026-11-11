use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub id: String,
    pub project_id: String,
    pub agent_id: Option<String>,
    pub status: String,
    pub started_at: String,
    pub duration_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    pub id: String,
    pub trace_id: String,
    pub parent_span_id: Option<String>,
    pub name: String,
    pub kind: String,
    pub status: String,
    pub started_at: String,
    pub duration_ms: i64,
    #[serde(default)]
    pub attributes: serde_json::Value,
}
