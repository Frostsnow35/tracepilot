// 模块注册表：新增模块在此登记一行即被 main 挂载，核心不改。
// 依赖顺序：iam 为根，storage 次之，ingest/query 依赖 storage。

pub mod analytics;
pub mod iam;
pub mod ingest;
pub mod monitor;
pub mod query;
pub mod storage;

use std::collections::HashSet;
use std::sync::Arc;

use axum::Router;

use crate::state::AppState;

/// 模块插件 trait。Rust 无运行时注册表，用 trait + registry 组合实现插件化。
pub trait Module: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn depends_on(&self) -> &'static [&'static str] {
        &[]
    }
    fn router(&self) -> Router<AppState>;
}

pub fn registry() -> Vec<Arc<dyn Module>> {
    let modules: Vec<Arc<dyn Module>> = vec![
        Arc::new(iam::IamModule),
        Arc::new(storage::StorageModule),
        Arc::new(ingest::IngestModule),
        Arc::new(query::QueryModule),
        Arc::new(monitor::MonitorModule),
        Arc::new(analytics::AnalyticsModule),
    ];
    topo_sort(modules)
}

/// 按 depends_on 依赖拓扑排序，保证模块挂载顺序正确（被依赖者先挂载）。
fn topo_sort(modules: Vec<Arc<dyn Module>>) -> Vec<Arc<dyn Module>> {
    let mut order: Vec<Arc<dyn Module>> = Vec::new();
    let mut visited: HashSet<&'static str> = HashSet::new();
    for m in &modules {
        visit(m, &modules, &mut visited, &mut order);
    }
    order
}

fn visit(
    target: &Arc<dyn Module>,
    all: &[Arc<dyn Module>],
    visited: &mut HashSet<&'static str>,
    order: &mut Vec<Arc<dyn Module>>,
) {
    if !visited.insert(target.name()) {
        return;
    }
    for dep in target.depends_on() {
        if let Some(d) = all.iter().find(|x| x.name() == *dep) {
            visit(d, all, visited, order);
        }
    }
    order.push(target.clone());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;

    struct FakeModule(&'static str, &'static [&'static str]);
    impl Module for FakeModule {
        fn name(&self) -> &'static str {
            self.0
        }
        fn depends_on(&self) -> &'static [&'static str] {
            self.1
        }
        fn router(&self) -> Router<AppState> {
            Router::new()
        }
    }

    #[test]
    fn topo_sort_orders_dependencies_first() {
        let modules: Vec<Arc<dyn Module>> = vec![
            Arc::new(FakeModule("ingest", &["storage", "iam"])),
            Arc::new(FakeModule("query", &["storage"])),
            Arc::new(FakeModule("storage", &["iam"])),
            Arc::new(FakeModule("iam", &[])),
        ];
        let sorted = topo_sort(modules);
        let names: Vec<&str> = sorted.iter().map(|m| m.name()).collect();

        let pos = |n: &str| names.iter().position(|x| *x == n).unwrap();
        assert!(pos("iam") < pos("storage"), "iam 应在 storage 前");
        assert!(pos("storage") < pos("ingest"), "storage 应在 ingest 前");
        assert!(pos("storage") < pos("query"), "storage 应在 query 前");
    }
}
