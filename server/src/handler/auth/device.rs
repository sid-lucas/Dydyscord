use axum::{Extension, Json, extract::State, http::StatusCode};
use axum_extra::extract::cookie::CookieJar;
use base64::Engine;
use common::{
    CreateDeviceResponse, DeviceKeyPackage, KeyPackagesUploadRequest, UserKeyPackageRequest,
    WelcomeFetchResponse, WelcomeStoreRequest,
};
use openmls::prelude::{KeyPackageIn, ProtocolVersion, tls_codec::DeserializeBytes};
use openmls_rust_crypto::OpenMlsRustCrypto;
use openmls_traits::OpenMlsProvider;
use redis::AsyncCommands;
use uuid::Uuid;

use crate::config::constant;
use crate::config::server::ServerState;
use crate::database::model::Device;
use crate::handler::auth::jwt::Claims;
use crate::handler::auth::{self, jwt};

pub async fn create_device(
    State(state): State<ServerState>,
    Extension(claims_jwt): Extension<Claims>,
) -> Result<(StatusCode, CookieJar, Json<CreateDeviceResponse>), StatusCode> {
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

    Ok((
        StatusCode::CREATED,
        jar,
        Json(CreateDeviceResponse {
            device_id: device.id,
        }),
    ))
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
    Json(payload): Json<KeyPackagesUploadRequest>,
) -> Result<StatusCode, StatusCode> {
    let device_id = Uuid::parse_str(claims_jwt.sub()).map_err(|_| StatusCode::BAD_REQUEST)?;

    for kp_bytes in payload.key_packages {
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

pub async fn get_keypackage_from_username(
    State(state): State<ServerState>,
    Json(payload): Json<UserKeyPackageRequest>,
) -> Result<(StatusCode, Json<Vec<DeviceKeyPackage>>), StatusCode> {
    // Retrieve username and compute the corresponding login_lookup
    let login_lookup = auth::login_lookup(&state.pepper(), &payload.username);

    // Atomically selects and deletes the oldest key_package per device for the given user, returning the consumed key packages.
    // Ensures one-time consumption and avoids race conditions.
    let out = sqlx::query_as!(
        DeviceKeyPackage,
        r#"
        WITH candidates AS (
        SELECT
            kp.id,
            kp.device_id,
            kp.key_package,
            row_number() OVER (
            PARTITION BY kp.device_id
            ORDER BY kp.created_at ASC, kp.id ASC
            ) AS rn
        FROM users u
        JOIN devices d ON d.user_id = u.id
        JOIN key_packages kp ON kp.device_id = d.id
        WHERE u.login_lookup = $1
        ),
        to_delete AS (
        SELECT id, device_id, key_package
        FROM candidates
        WHERE rn = 1
        )
        DELETE FROM key_packages kp
        USING to_delete td
        WHERE kp.id = td.id
        RETURNING td.device_id, td.key_package
        "#,
        login_lookup
    )
    .fetch_all(&state.pool())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::OK, Json(out)))
}

pub async fn store_welcome(
    State(state): State<ServerState>,
    Json(payload): Json<WelcomeStoreRequest>,
) -> Result<StatusCode, StatusCode> {
    let welcome_bytes = base64::engine::general_purpose::STANDARD
        .decode(payload.welcome_b64)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    for device_id in payload.device_ids {
        sqlx::query!(
            r#"INSERT INTO welcomes (device_id, welcome) VALUES ($1, $2)"#,
            device_id,
            welcome_bytes,
        )
        .execute(&state.pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(StatusCode::OK)
}

pub async fn fetch_welcome(
    State(state): State<ServerState>,
    Extension(claims_jwt): Extension<Claims>,
) -> Result<(StatusCode, Json<Vec<WelcomeFetchResponse>>), StatusCode> {
    // Retrieve the device_id from the JWT Session
    let device_id = Uuid::parse_str(claims_jwt.sub()).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Grab all pending welcomes for this device
    let rows = sqlx::query!(
        r#"
        SELECT id, welcome
        FROM welcomes
        WHERE device_id = $1
        ORDER BY created_at
        "#,
        device_id
    )
    .fetch_all(&state.pool())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // If no welcome pending...
    if rows.is_empty() {
        return Ok((StatusCode::OK, Json(vec![])));
    }

    // One‑time messages: we delete right after reading
    let ids: Vec<i32> = rows.iter().map(|r| r.id).collect();
    sqlx::query!(r#"DELETE FROM welcomes WHERE id = ANY($1)"#, &ids)
        .execute(&state.pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Encode bytes to base64 for JSON response
    let out = rows
        .into_iter()
        .map(|r| WelcomeFetchResponse {
            welcome_b64: base64::engine::general_purpose::STANDARD.encode(r.welcome),
        })
        .collect();

    Ok((StatusCode::OK, Json(out)))
}
