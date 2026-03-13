use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;

    let state = AppState { pool };

    let app = Router::new()
        .route("/", get(health_handler))
        .route("/wallets", post(create_wallet))
        .route("/wallets/:id", get(get_wallet))
        .route("/transfer", post(transfer))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("L-NHI Gateway listening on {}", addr);
    axum::serve(
        tokio::net::TcpListener::bind(addr).await?,
        app,
    )
    .await?;

    Ok(())
}

async fn health_handler() -> &'static str {
    "L-NHI Gateway is running"
}

// --- Wallet types ---

#[derive(Debug, Serialize)]
struct Wallet {
    id: Uuid,
    owner_name: String,
    balance: i64,
}

#[derive(Debug, Deserialize)]
struct CreateWalletRequest {
    owner_name: String,
    balance: i64,
}

#[derive(Debug, Deserialize)]
struct TransferRequest {
    from_wallet_id: Uuid,
    to_wallet_id: Uuid,
    amount: i64,
}

// --- Handlers ---

async fn create_wallet(
    State(state): State<AppState>,
    Json(req): Json<CreateWalletRequest>,
) -> Result<Json<Wallet>, (StatusCode, String)> {
    if req.balance < 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            "balance must be >= 0".to_string(),
        ));
    }

    let id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO wallets (id, owner_name, balance)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(id)
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
        owner_name: req.owner_name,
        balance: req.balance,
    }))
}

async fn get_wallet(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Wallet>, (StatusCode, String)> {
    let row = sqlx::query_as::<_, WalletRow>(
        "SELECT id, owner_name, balance FROM wallets WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match row {
        Some(r) => Ok(Json(Wallet {
            id: r.id,
            owner_name: r.owner_name,
            balance: r.balance,
        })),
        None => Err((StatusCode::NOT_FOUND, "wallet not found".to_string())),
    }
}

#[derive(FromRow)]
struct WalletRow {
    id: Uuid,
    owner_name: String,
    balance: i64,
}

async fn transfer(
    State(state): State<AppState>,
    Json(req): Json<TransferRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if req.amount <= 0 {
        write_audit_rejected(
            &state.pool,
            req.from_wallet_id,
            "transfer",
            req.amount,
            "amount must be positive",
        )
        .await;
        return Err((
            StatusCode::BAD_REQUEST,
            "amount must be positive".to_string(),
        ));
    }

    if req.from_wallet_id == req.to_wallet_id {
        write_audit_rejected(
            &state.pool,
            req.from_wallet_id,
            "transfer",
            req.amount,
            "cannot transfer to same wallet",
        )
        .await;
        return Err((
            StatusCode::BAD_REQUEST,
            "cannot transfer to same wallet".to_string(),
        ));
    }

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Fetch source wallet
    let from_row = sqlx::query_as::<_, WalletRow>(
        "SELECT id, owner_name, balance FROM wallets WHERE id = $1",
    )
    .bind(req.from_wallet_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if from_row.is_none() {
        tx.rollback().await.ok();
        write_audit_rejected(
            &state.pool,
            req.from_wallet_id,
            "transfer",
            req.amount,
            "source wallet not found",
        )
        .await;
        return Err((
            StatusCode::NOT_FOUND,
            "source wallet not found".to_string(),
        ));
    }

    let from_balance = from_row.as_ref().unwrap().balance;
    if from_balance < req.amount {
        tx.rollback().await.ok();
        write_audit_rejected(
            &state.pool,
            req.from_wallet_id,
            "transfer",
            req.amount,
            "insufficient balance",
        )
        .await;
        return Err((
            StatusCode::BAD_REQUEST,
            "insufficient balance".to_string(),
        ));
    }

    // Check destination exists
    let to_exists = sqlx::query_scalar::<_, i32>("SELECT 1 FROM wallets WHERE id = $1")
        .bind(req.to_wallet_id)
        .fetch_optional(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if to_exists.is_none() {
        tx.rollback().await.ok();
        write_audit_rejected(
            &state.pool,
            req.from_wallet_id,
            "transfer",
            req.amount,
            "destination wallet not found",
        )
        .await;
        return Err((
            StatusCode::NOT_FOUND,
            "destination wallet not found".to_string(),
        ));
    }

    // Execute transfer
    sqlx::query("UPDATE wallets SET balance = balance - $1 WHERE id = $2")
        .bind(req.amount)
        .bind(req.from_wallet_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query("UPDATE wallets SET balance = balance + $1 WHERE id = $2")
        .bind(req.amount)
        .bind(req.to_wallet_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Audit log (inside transaction)
    let audit_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO audit_logs (id, wallet_id, action, amount, status, reason)
        VALUES ($1, $2, 'transfer', $3, 'completed', NULL)
        "#,
    )
    .bind(audit_id)
    .bind(req.from_wallet_id)
    .bind(req.amount)
    .execute(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "status": "completed",
        "from_wallet_id": req.from_wallet_id,
        "to_wallet_id": req.to_wallet_id,
        "amount": req.amount
    })))
}

async fn write_audit_rejected(
    pool: &PgPool,
    wallet_id: Uuid,
    action: &str,
    amount: i64,
    reason: &str,
) {
    let audit_id = Uuid::new_v4();
    if let Err(e) = sqlx::query(
        r#"
        INSERT INTO audit_logs (id, wallet_id, action, amount, status, reason)
        VALUES ($1, $2, $3, $4, 'rejected', $5)
        "#,
    )
    .bind(audit_id)
    .bind(wallet_id)
    .bind(action)
    .bind(amount)
    .bind(reason)
    .execute(pool)
    .await
    {
        tracing::error!("Failed to write audit log: {}", e);
    }
}
