use tokio::sync::broadcast;
use tracepilot_core::Config;

use crate::modules::monitor::TraceEvent;

/// 共享应用状态：配置 + 数据库池 + 存储服务 + 监控广播通道。
/// 各模块通过它获取依赖，模块间不直接耦合。
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub storage: crate::modules::storage::Storage,
    pub monitor_tx: broadcast::Sender<TraceEvent>,
}
