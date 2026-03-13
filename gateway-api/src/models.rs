use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct Agent {
    pub id: Uuid,
    pub name: String,
    pub agent_type: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub agent_type: String,
}

#[derive(Debug, Serialize)]
pub struct Wallet {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub owner_name: String,
    pub balance: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateWalletRequest {
    pub agent_id: Uuid,
    pub owner_name: String,
    pub balance: i64,
}

#[derive(Debug, Deserialize)]
pub struct TransferRequest {
    pub from_wallet_id: Uuid,
    pub to_wallet_id: Uuid,
    pub amount: i64,
}

#[derive(FromRow)]
pub struct AgentRow {
    pub id: Uuid,
    pub name: String,
    pub agent_type: String,
}

#[derive(FromRow)]
pub struct WalletRow {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub owner_name: String,
    pub balance: i64,
}
