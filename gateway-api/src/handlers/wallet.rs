use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::db::AppState;
use crate::models::{CreateWalletRequest, Wallet, WalletRow};

pub async fn create_wallet(
    State(state): State<AppState>,
    Json(req): Json<CreateWalletRequest>,
) -> Result<Json<Wallet>, (StatusCode, String)> {
    if req.balance < 0 {
        return Err((StatusCode::BAD_REQUEST, "balance must be >= 0".to_string()));
    }

    let agent_exists = sqlx::query_scalar::<_, i32>("SELECT 1 FROM agents WHERE id = $1")
        .bind(req.agent_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if agent_exists.is_none() {
        return Err((StatusCode::NOT_FOUND, "agent not found".to_string()));
    }

    let id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO wallets (id, agent_id, owner_name, balance)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(id)
    .bind(req.agent_id)
    .bind(&req.owner_name)
    .bind(req.balance)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Audit log
    let audit_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO audit_logs (id, wallet_id, action, amount, status, reason)
        VALUES ($1, $2, 'create', $3, 'completed', NULL)
        "#,
    )
    .bind(audit_id)
    .bind(id)
    .bind(req.balance)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(Wallet {
        id,
        agent_id: req.agent_id,
        owner_name: req.owner_name,
        balance: req.balance,
    }))
}

pub async fn get_wallet(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Wallet>, (StatusCode, String)> {
    let row = sqlx::query_as::<_, WalletRow>(
        "SELECT id, agent_id, owner_name, balance FROM wallets WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match row {
        Some(r) => Ok(Json(Wallet {
            id: r.id,
            agent_id: r.agent_id,
            owner_name: r.owner_name,
            balance: r.balance,
        })),
        None => Err((StatusCode::NOT_FOUND, "wallet not found".to_string())),
    }
}
