use crate::ServerState;
use axum::{Json, extract::State, http::StatusCode};
use base64::Engine;
use uuid::Uuid;

use crate::database::models::{CreateUserPayload, User};
use crate::handlers::auth::login_lookup;

pub async fn create_user(
    State(state): State<ServerState>,
    Json(payload): Json<CreateUserPayload>,
) -> Result<(StatusCode, Json<User>), StatusCode> {
    let opaque_record = base64::engine::general_purpose::STANDARD
        .decode(&payload.opaque_record)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let login_lookup = login_lookup(&state.pepper, &payload.username);
    let user_id = Uuid::new_v4();

    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (id, login_lookup, opaque_record) VALUES ($1, $2, $3) RETURNING id",
        user_id,
        login_lookup,
        opaque_record,
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(user)))
}
