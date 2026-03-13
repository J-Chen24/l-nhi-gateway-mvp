use axum::{extract::State, Json};

use crate::db::AppState;
use crate::models::{AuditLog, AuditLogRow};

pub async fn list_audit_logs(
    State(state): State<AppState>,
) -> Result<Json<Vec<AuditLog>>, (axum::http::StatusCode, String)> {
    let rows = sqlx::query_as::<_, AuditLogRow>(
        r#"
        SELECT id, wallet_id, action, amount, status, reason, created_at
        FROM audit_logs
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let logs: Vec<AuditLog> = rows
        .into_iter()
        .map(|r| AuditLog {
            id: r.id,
            wallet_id: r.wallet_id,
            action: r.action,
            amount: r.amount,
            status: r.status,
            reason: r.reason,
            created_at: r.created_at,
        })
        .collect();

    Ok(Json(logs))
}
