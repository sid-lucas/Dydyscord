use crate::config::server::ServerState;
use crate::database::model::Device;
use crate::handler::auth::jwt::Claims;
use axum::{Extension, Json, extract::State, http::StatusCode};
use uuid::Uuid;

pub async fn create_device(
    State(state): State<ServerState>,
    Extension(claims_jwt): Extension<Claims>,
) -> Result<(StatusCode, Json<Uuid>), StatusCode> {
    let user_id = Uuid::parse_str(claims_jwt.sub()).map_err(|_| StatusCode::BAD_REQUEST)?;

    let device = sqlx::query_as!(
        Device,
        r#"INSERT INTO devices (user_id) VALUES ($1) RETURNING id, user_id, created_at, updated_at"#,
        user_id
    )
    .fetch_one(&state.pool())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(device.id)))
}
