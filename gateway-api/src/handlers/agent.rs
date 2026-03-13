use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::db::AppState;
use crate::models::{Agent, AgentRow, CreateAgentRequest};

pub async fn create_agent(
    State(state): State<AppState>,
    Json(req): Json<CreateAgentRequest>,
) -> Result<Json<Agent>, (StatusCode, String)> {
    let id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO agents (id, name, agent_type)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(id)
    .bind(&req.name)
    .bind(&req.agent_type)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(Agent {
        id,
        name: req.name,
        agent_type: req.agent_type,
    }))
}

pub async fn get_agent(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Agent>, (StatusCode, String)> {
    let row =
        sqlx::query_as::<_, AgentRow>("SELECT id, name, agent_type FROM agents WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match row {
        Some(agent) => Ok(Json(Agent {
            id: agent.id,
            name: agent.name,
            agent_type: agent.agent_type,
        })),
        None => Err((StatusCode::NOT_FOUND, "agent not found".to_string())),
    }
}
