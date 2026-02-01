use crate::ServerState;
use axum::{Json, extract::State, http::StatusCode};
use crate::database::models::{CreateUserPayload, User};

pub async fn create_user(
    State(state): State<ServerState>,
    Json(payload): Json<CreateUserPayload>,
) -> Result<(StatusCode, Json<User>), StatusCode> {
    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (username) VALUES ($1) RETURNING id, username",
        payload.username
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(user)))
}