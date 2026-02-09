use crate::api::jwt::Claims;
use crate::config::ServerState;
use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
};

pub async fn create_device(
    State(state): State<ServerState>,
    Json(payload): Json,
    Extension(claims_jwt): Extension<Claims>,
) -> Result<(StatusCode, Json), StatusCode> {
    // Stocker le opaque_record dans la BDD associé au login_lookup
    sqlx::query!(
        "INSERT INTO devices (user_id) VALUES ($1) RETURNING id",
        login_lookup,
    )
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
}
