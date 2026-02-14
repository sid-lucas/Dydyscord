use crate::config::constant;
use crate::config::server::ServerState;
use crate::database::model::Device;
use crate::handler::auth::jwt;
use crate::handler::auth::jwt::Claims;
use axum::{Extension, Json, extract::State, http::StatusCode};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use secrecy::ExposeSecret;
use uuid::Uuid;

pub async fn create_device(
    State(state): State<ServerState>,
    Extension(claims_jwt): Extension<Claims>,
) -> Result<(StatusCode, CookieJar, Json<Uuid>), StatusCode> {
    let user_id = Uuid::parse_str(claims_jwt.sub()).map_err(|_| StatusCode::BAD_REQUEST)?;

    let device = sqlx::query_as!(
        Device,
        r#"INSERT INTO devices (user_id) VALUES ($1) RETURNING id, user_id, created_at, updated_at"#,
        user_id
    )
    .fetch_one(&state.pool())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create the intermediate JWT (auth)
    let token = jwt::create_jwt(
        device.id.to_string().as_str(),
        jwt::TokenType::Refresh,
        state.jwt_key().expose_secret(),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let cookie = Cookie::build((constant::AUTH_HEADER, token))
        .http_only(false) // TODO change
        .secure(false) // TODO Change: true forbids sending over HTTP. -> false for local testing for now.
        .same_site(SameSite::Strict)
        .path("/")
        .build();

    let redis_key = format!("jwt:{}", &cookie);
    let state_bytes: Vec<u8> = "ok".as_bytes().to_vec(); // We don't need to store any specific state for the device creation, just the existence of the key with the correct TTL is enough for validation in the middleware
    let _: () = state
        .redis()
        .set_ex(&redis_key, state_bytes, constant::JWT_REFRESH_TTL)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let jar = CookieJar::new().add(cookie);

    Ok((StatusCode::CREATED, jar, Json(device.id)))
}
