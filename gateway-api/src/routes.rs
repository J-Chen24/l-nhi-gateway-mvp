use axum::{
    routing::{get, post},
    Router,
};

use crate::db::AppState;
use crate::handlers::{agent, audit, transfer, wallet};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(health_handler))
        .route("/agents", post(agent::create_agent))
        .route("/agents/:id", get(agent::get_agent))
        .route("/wallets", post(wallet::create_wallet))
        .route("/wallets/:id", get(wallet::get_wallet))
        .route("/transfer", post(transfer::transfer))
        .route("/audit-logs", get(audit::list_audit_logs))
        .with_state(state)
}

async fn health_handler() -> &'static str {
    "L-NHI Gateway is running"
}
