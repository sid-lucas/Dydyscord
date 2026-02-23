use axum::extract::ws::Message;
use axum::{Extension, Json, extract::State, http::StatusCode};
use base64::Engine;
use common::{DeviceKeyPackage, KeyPackagesUploadRequest, UserKeyPackageRequest};
use common::{WelcomeFetchResponse, WelcomeStoreRequest};
use openmls::prelude::{KeyPackageIn, ProtocolVersion, tls_codec::DeserializeBytes};
use openmls_rust_crypto::OpenMlsRustCrypto;
use openmls_traits::OpenMlsProvider;
use uuid::Uuid;

use crate::config::server::ServerState;
use crate::handler;
use crate::handler::jwt::Claims;

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
    if payload.invited.is_empty() {
        return Ok((StatusCode::OK, Json(vec![])));
    }

    // Retrieve usernames and compute the corresponding login_lookup list
    let login_lookups: Vec<String> = payload
        .invited
        .iter()
        .map(|username| handler::login_lookup(&state.pepper(), username))
        .collect();

    // TODO : REPRENDRE ICI : Faire que ca recup un keypackage pour chaque login_lookups

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
        WHERE u.login_lookup = ANY($1)
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
        &login_lookups
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

        if let Some(tx) = state.sockets.get(&device_id.to_string()) {
            let msg = Message::Text(r#"{"type":"welcome_ready"}"#.into());
            if tx.send(msg).is_err() {
                // socket dead -> cleanup
                state.sockets.remove(&device_id.to_string());
            }
        }
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
