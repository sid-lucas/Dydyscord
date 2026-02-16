use crate::config::constant;
use crate::config::server::ServerState;
use crate::database::model::{Device, KeyPackage, User};
use crate::handler::auth::jwt::Claims;
use crate::handler::auth::{self, jwt};

use axum::{Extension, Json, extract::State, http::StatusCode};
use axum_extra::extract::cookie::CookieJar;
use openmls::prelude::{
    KeyPackageIn, ProtocolVersion,
    tls_codec::{DeserializeBytes, Serialize as TlsSerialize},
};
use openmls_rust_crypto::OpenMlsRustCrypto;
use openmls_traits::OpenMlsProvider;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize)]
pub struct DeviceKeyPackage {
    pub device_id: Uuid,
    pub key_package: Vec<u8>,
}

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

pub async fn get_keypackage_from_username(
    State(state): State<ServerState>,
    Json(payload): Json<String>,
) -> Result<(StatusCode, Json<Vec<DeviceKeyPackage>>), StatusCode> {
    // Retrieve username and compute the corresponding login_lookup
    let login_lookup = auth::login_lookup(&state.pepper(), &payload);

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

/* Old function, to delete

pub async fn get_keypackage_from_username_old(
    State(state): State<ServerState>,
    Json(payload): Json<String>,
) -> Result<(StatusCode, Json<Vec<DeviceKeyPackage>>), StatusCode> {
    // Retrieve username and compute the corresponding login_lookup
    let login_lookup = auth::login_lookup(&state.pepper(), &payload);

    // Get the user from the login_lookup
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, login_lookup, opaque_record, created_at, updated_at
        FROM users
        WHERE login_lookup = $1
        "#,
        login_lookup,
    )
    .fetch_optional(&state.pool())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Get all the devices of the user_id
    let devices = sqlx::query_as!(
        Device,
        r#"
        SELECT id, user_id, created_at, updated_at
        FROM devices
        WHERE user_id = $1
        "#,
        user.id
    )
    .fetch_all(&state.pool())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get one keypackage for all the device_id
    let mut out = Vec::new();
    for device in devices {
        let kp = sqlx::query_as!(
            KeyPackage,
            r#"
            SELECT id, device_id, key_package, created_at, updated_at
            FROM key_packages
            WHERE device_id = $1
            ORDER BY created_at
            LIMIT 1
            "#,
            device.id
        )
        .fetch_optional(&state.pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if let Some(kp) = kp {
            out.push(DeviceKeyPackage {
                device_id: device.id,
                key_package: kp.key_package,
            });
        }
    }

    Ok((StatusCode::OK, Json(out)))
}
 */
