use DefaultCipherSuite as DCS;
use base64::Engine;
use opaque_ke::CipherSuite;
use opaque_ke::argon2::Argon2;
use rand::rngs::OsRng;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::jwt;
use crate::config::server::ServerState;
use crate::constant;

use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};

use hmac::Mac;
use opaque_ke::{
    CredentialFinalization, CredentialRequest, RegistrationRequest, RegistrationUpload,
    ServerLogin, ServerLoginParameters, ServerRegistration,
};
use redis::AsyncCommands;
use secrecy::ExposeSecret;

use crate::database::model::User;

// Register

#[derive(Deserialize, Debug)]
pub struct RegisterStartRequest {
    pub username: String,
    pub start_register_request: String, // base64
}

#[derive(Serialize)]
pub struct RegisterStartResponse {
    pub start_register_response: String, // base64
}

#[derive(Deserialize)]
pub struct RegisterFinishRequest {
    pub username: String,
    pub finish_register_request: String, // base64
}

// Login

#[derive(Deserialize)]
pub struct LoginStartRequest {
    pub username: String,
    pub start_login_request: String, // base64
}

#[derive(Serialize)]
pub struct LoginStartResponse {
    pub start_login_response: String, // base64
    pub user_id: Uuid,                // also used as key-value to retrieve server_login_state
}

#[derive(Deserialize)]
pub struct LoginFinishRequest {
    pub finish_login_request: String, // base64
    pub user_id: String,              // key-value to retrieve server_login_state
}

pub struct DefaultCipherSuite;

impl CipherSuite for DefaultCipherSuite {
    type OprfCs = opaque_ke::Ristretto255;
    type KeyExchange = opaque_ke::TripleDh<opaque_ke::Ristretto255, sha2::Sha512>;
    type Ksf = Argon2<'static>;
}

fn login_lookup(pepper: &[u8], username: &str) -> Vec<u8> {
    let normalized = username.trim().to_lowercase();

    let mut mac =
        hmac::Hmac::<sha2::Sha256>::new_from_slice(pepper).expect("HMAC can take key of any size");

    mac.update(normalized.as_bytes());

    mac.finalize().into_bytes().to_vec()
}

pub async fn register_start(
    State(state): State<ServerState>,
    Json(payload): Json<RegisterStartRequest>,
) -> Result<(StatusCode, Json<RegisterStartResponse>), StatusCode> {
    // Retrieve start_register_request from the client and decode/deserialize it
    let decoded_request = base64::engine::general_purpose::STANDARD
        .decode(&payload.start_register_request)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let start_register_request = RegistrationRequest::<DCS>::deserialize(&decoded_request)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Retrieve username and compute the corresponding login_lookup
    let username = payload.username;
    let login_lookup = login_lookup(&state.pepper().expose_secret(), &username);

    // Check if the user already exists in the database
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
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // If user exists, error:
    match user {
        Some(_) => return Err(StatusCode::CONFLICT), // username already taken
        None => (),                                  // continue normally
    }

    // Start server registration with OPAQUE
    let start = ServerRegistration::<DCS>::start(
        &state.opaque_setup(),
        start_register_request,
        login_lookup.as_slice(),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Prepare the response to send to the client
    let start_register_response =
        base64::engine::general_purpose::STANDARD.encode(start.message.serialize());

    // Create and send the response
    let response = RegisterStartResponse {
        start_register_response,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn register_finish(
    State(state): State<ServerState>,
    Json(payload): Json<RegisterFinishRequest>,
) -> Result<StatusCode, StatusCode> {
    // Retrieve finish_register_request from the client and decode/deserialize it
    let decoded_request = base64::engine::general_purpose::STANDARD
        .decode(&payload.finish_register_request)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let finish_register_request = RegistrationUpload::<DCS>::deserialize(&decoded_request)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Finalize registration by creating opaque_record to store
    let opaque_record = ServerRegistration::finish(finish_register_request)
        .serialize()
        .to_vec();

    // Retrieve username
    let username = payload.username;
    // Recompute login_lookup with server_pepper and username
    let login_lookup = login_lookup(&state.pepper().expose_secret(), &username);

    // Store opaque_record in the database associated with login_lookup
    sqlx::query!(
        "INSERT INTO users (login_lookup, opaque_record) VALUES ($1, $2)",
        login_lookup,
        opaque_record,
    )
    .execute(&state.pool())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

pub async fn login_start(
    State(mut state): State<ServerState>,
    Json(payload): Json<LoginStartRequest>,
) -> Result<(StatusCode, Json<LoginStartResponse>), StatusCode> {
    // Retrieve start_login_request from the client and decode/deserialize it
    let start_login_request = CredentialRequest::<DCS>::deserialize(
        &base64::engine::general_purpose::STANDARD
            .decode(&payload.start_login_request)
            .map_err(|_| StatusCode::BAD_REQUEST)?,
    )
    .map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut server_rng = OsRng;

    // Retrieve username and compute the corresponding login_lookup
    let username = payload.username;
    let login_lookup = login_lookup(&state.pepper().expose_secret(), &username);

    // Retrieve user matching login_lookup from the database
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, login_lookup, opaque_record, created_at, updated_at
        FROM users
        WHERE login_lookup = $1
        "#,
        login_lookup,
    )
    // fetch_optional to avoid revealing whether a user exists or not
    .fetch_optional(&state.pool())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Deserialize opaque_record if user exists
    let (opaque_record, user_id) = match user {
        Some(user) => {
            let opaque_record = ServerRegistration::<DCS>::deserialize(&user.opaque_record)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            (Some(opaque_record), user.id)
        }
        None => return Err(StatusCode::NOT_FOUND),
    };

    // Start server login with OPAQUE
    let start = ServerLogin::start(
        &mut server_rng,
        &state.opaque_setup(),
        opaque_record,
        start_login_request,
        login_lookup.as_slice(),
        ServerLoginParameters::default(),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Save server_login_state in Redis with expiration
    let redis_key = format!("opaque:login:{}", &user_id);
    let state_bytes: Vec<u8> = start.state.serialize().to_vec();
    let _: () = state
        .redis()
        .set_ex(&redis_key, state_bytes, constant::REDIS_OPAQUE_STATE_TTL)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create and send the response
    let start_login_response =
        base64::engine::general_purpose::STANDARD.encode(start.message.serialize());

    Ok((
        StatusCode::OK,
        Json(LoginStartResponse {
            start_login_response,
            user_id,
        }),
    ))
}

pub async fn login_finish(
    State(state): State<ServerState>,
    Json(payload): Json<LoginFinishRequest>,
) -> Result<(StatusCode, CookieJar), StatusCode> {
    // Create the key with user_id
    let redis_key = format!("opaque:login:{}", payload.user_id);

    // Retrieve server_login_state from Redis
    let state_bytes: Option<Vec<u8>> = state
        .redis()
        .get(&redis_key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // If retrieval failed (invalid key)
    let state_bytes: Vec<u8> = match state_bytes {
        Some(v) => v,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // One-shot delete (prevents replay)
    let _: () = state
        .redis()
        .del(&redis_key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Decode/Deserialize the client's final message
    let finish_login_request = base64::engine::general_purpose::STANDARD
        .decode(&payload.finish_login_request)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let finish_login_request = CredentialFinalization::<DCS>::deserialize(&finish_login_request)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Deserialize server_login_state then finish()
    let server_login_state = ServerLogin::<DCS>::deserialize(&state_bytes)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let finish = server_login_state
        .finish(finish_login_request, ServerLoginParameters::default())
        .map_err(|_| StatusCode::UNAUTHORIZED)?; // wrong password or invalid proof

    // Shared secret between client and server
    let _session_key = finish.session_key;

    // Create the intermediate JWT (auth)
    let id = payload.user_id.to_string();
    let token = jwt::create_jwt(
        id.as_str(),
        jwt::TokenType::Auth,
        state.jwt_key().expose_secret(),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let cookie = Cookie::build((constant::AUTH_HEADER, token))
        .http_only(false) // TODO change
        .secure(false) // TODO Change: true forbids sending over HTTP. -> false for local testing for now.
        .same_site(SameSite::Strict)
        .path("/")
        .build();

    let jar = CookieJar::new().add(cookie);

    Ok((StatusCode::OK, jar))
}
