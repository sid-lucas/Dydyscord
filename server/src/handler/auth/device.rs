use crate::config::constant;
use crate::config::server::ServerState;
use crate::database::model::Device;
use crate::handler::auth::jwt;
use crate::handler::auth::jwt::Claims;
use axum::{Extension, Json, extract::State, http::StatusCode};
use axum_extra::extract::cookie::CookieJar;
use openmls::prelude::{
    KeyPackageIn, ProtocolVersion,
    tls_codec::{DeserializeBytes, Serialize as TlsSerialize},
};
use openmls_rust_crypto::OpenMlsRustCrypto;
use openmls_traits::OpenMlsProvider;
use redis::AsyncCommands;
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

    let cookie = jwt::create_cookie(
        device.id.to_string().as_str(),
        jwt::TokenType::Session,
        state.jwt_key().as_ref(),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let redis_key = format!("jwt:{}", &cookie);
    let state_bytes: Vec<u8> = "ok".as_bytes().to_vec(); // We don't need to store any specific state for the device creation, just the existence of the key with the correct TTL is enough for validation in the middleware
    let _: () = state
        .redis()
        .set_ex(&redis_key, state_bytes, constant::JWT_SESSION_TTL as u64)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let jar = CookieJar::new().add(cookie);

    Ok((StatusCode::CREATED, jar, Json(device.id)))
}

pub async fn get_device(
    State(state): State<ServerState>,
    Extension(claims_jwt): Extension<Claims>,
) -> Result<(StatusCode, CookieJar), StatusCode> {
    let user_id = Uuid::parse_str(claims_jwt.sub()).map_err(|_| StatusCode::BAD_REQUEST)?;

    let device = sqlx::query_as!(
        Device,
        r#"SELECT id, user_id, created_at, updated_at FROM devices WHERE user_id = $1"#,
        user_id
    )
    .fetch_one(&state.pool())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let cookie = jwt::create_cookie(
        device.id.to_string().as_str(),
        jwt::TokenType::Session,
        state.jwt_key().as_ref(),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let redis_key = format!("jwt:{}", &cookie);
    let state_bytes: Vec<u8> = "ok".as_bytes().to_vec(); // We don't need to store any specific state for the device creation, just the existence of the key with the correct TTL is enough for validation in the middleware
    let _: () = state
        .redis()
        .set_ex(&redis_key, state_bytes, constant::JWT_SESSION_TTL as u64)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let jar = CookieJar::new().add(cookie);

    Ok((StatusCode::OK, jar))
}

pub async fn update_key_packages(
    State(state): State<ServerState>,
    Extension(claims_jwt): Extension<Claims>,
    Json(payload): Json<Vec<Vec<u8>>>,
) -> Result<StatusCode, StatusCode> {
    let device_id = Uuid::parse_str(claims_jwt.sub()).map_err(|_| StatusCode::BAD_REQUEST)?;

    for kp_bytes in payload {
        // Validation of the key_package received (in a scope, so provider drop before the .await)
        {
            let provider = OpenMlsRustCrypto::default();
            let kp_in = KeyPackageIn::tls_deserialize_exact_bytes(&kp_bytes)
                .map_err(|_| StatusCode::BAD_REQUEST)?;
            kp_in
                .validate(provider.crypto(), ProtocolVersion::Mls10) // TODO : Maybe put version in a const
                .map_err(|_| StatusCode::BAD_REQUEST)?;
        }

        // Store only the public KeyPackage bytes (TLS-serialized).
        sqlx::query!(
            r#"INSERT INTO key_packages (device_id, key_package) VALUES ($1, $2)"#,
            device_id,
            kp_bytes
        )
        .execute(&state.pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(StatusCode::OK)
}
