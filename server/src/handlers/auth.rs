use crate::config::ServerState;
use crate::handlers::api_error::ApiError;
use crate::opaque::DefaultCipherSuite as DCS;
use crate::opaque::models::{
    LoginFinishRequest, LoginStartRequest, LoginStartResponse, RegisterFinishRequest,
    RegisterStartRequest, RegisterStartResponse,
};
use axum::response::Response;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::Mac;
use opaque_ke::{
    ClientLogin, ClientLoginFinishParameters, ClientRegistration,
    ClientRegistrationFinishParameters, CredentialFinalization, CredentialRequest,
    CredentialResponse, RegistrationRequest, RegistrationResponse, RegistrationUpload, ServerLogin,
    ServerLoginParameters, ServerRegistration, ServerRegistrationLen, ServerSetup,
};
use rand::RngCore;
use rand::rngs::OsRng;
use redis::AsyncCommands;
use secrecy::ExposeSecret;
use serde::Serialize;

use crate::database::models::User;

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
) -> Result<Json<RegisterStartResponse>, ApiError> {
    // Récupération du start_register_request du client et décodage/désérialisation
    let decoded_request = base64::engine::general_purpose::STANDARD
        .decode(&payload.start_register_request)
        .map_err(|e| ApiError::BadRequest((format!("Failed to decode request"))))?;
    let start_register_request = RegistrationRequest::<DCS>::deserialize(&decoded_request)
        .map_err(|_| ApiError::BadRequest(format!("Failed to deserialize request")))?;

    // Récupération du nom d'utilisateur et calcul du login_lookup correspondant
    let username = payload.username;
    let login_lookup = login_lookup(&state.pepper.expose_secret(), &username);

    // Check si l'utilisateur existe déjà dans la BDD
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, login_lookup, opaque_record, created_at, updated_at
        FROM users
        WHERE login_lookup = $1
        "#,
        login_lookup,
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::InternalError)?;

    // Si user existe, erreur :
    match user {
        Some(_) => return Err(ApiError::BadRequest("User already exists".to_string())),
        None => (), // continue normalement
    }

    // Démarrer le register server avec OPAQUE
    let start = ServerRegistration::<DCS>::start(
        &state.opaque_setup,
        start_register_request,
        login_lookup.as_slice(),
    )
    .map_err(|_| ApiError::InternalError)?;

    // Préparation de la request à envoyer au serveur
    let start_register_response =
        base64::engine::general_purpose::STANDARD.encode(start.message.serialize());

    // Création de la réponse et envoi
    let response = RegisterStartResponse {
        start_register_response,
    };

    Ok(Json(response))
}

pub async fn register_finish(
    State(state): State<ServerState>,
    Json(payload): Json<RegisterFinishRequest>,
) -> Result<Json<String>, ApiError> {
    // Récupération du finish_register_request du client et décodage/désérialisation
    let decoded_request = base64::engine::general_purpose::STANDARD
        .decode(&payload.finish_register_request)
        .map_err(|_| ApiError::BadRequest("Failed to decode request".to_string()))?;
    let finish_register_request = RegistrationUpload::<DCS>::deserialize(&decoded_request)
        .map_err(|_| ApiError::BadRequest("Failed to deserialize request".to_string()))?;

    // Finalisation du register en créant le opaque_record à stocker
    let opaque_record = ServerRegistration::finish(finish_register_request)
        .serialize()
        .to_vec();

    // Récupération du nom d'utilisateur
    let username = payload.username;
    // Recalculer le login_lookup avec le server_pepper et username
    let login_lookup = login_lookup(&state.pepper.expose_secret(), &username);

    // Stocker le opaque_record dans la BDD associé au login_lookup
    sqlx::query_as!(
        User,
        "INSERT INTO users (login_lookup, opaque_record) VALUES ($1, $2)",
        login_lookup,
        opaque_record,
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| ApiError::InternalError)?;

    Ok(Json("Registration successful".to_string()))
}

pub async fn login_start(
    State(mut state): State<ServerState>,
    Json(payload): Json<LoginStartRequest>,
) -> Result<Json<LoginStartResponse>, ApiError> {
    // Récupération du start_login_request du client et décodage/désérialisation
    let start_login_request = CredentialRequest::<DCS>::deserialize(
        &base64::engine::general_purpose::STANDARD
            .decode(&payload.start_login_request)
            .map_err(|_| ApiError::BadRequest("Failed to decode request".to_string()))?,
    )
    .map_err(|_| ApiError::BadRequest("Failed to deserialize request".to_string()))?;

    let mut server_rng = OsRng;

    // Récupération du nom d'utilisateur et calcul du login_lookup correspondant
    let username = payload.username;
    let login_lookup = login_lookup(&state.pepper.expose_secret(), &username);

    // Récupération du user correspondant au login_lookup dans la BDD
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, login_lookup, opaque_record, created_at, updated_at
        FROM users
        WHERE login_lookup = $1
        "#,
        login_lookup,
    )
    // fetch_optional pour ne pas révéler l'existence / non-existence d'un user
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::InternalError)?;

    // Désérialisation du opaque_record si user existe
    let opaque_record = match user {
        Some(user) => Some(
            ServerRegistration::<DCS>::deserialize(&user.opaque_record)
                .map_err(|_| ApiError::InternalError)?,
        ),
        None => return Err(ApiError::NotFound), // user non trouvé
    };

    // Démarrer le login server avec OPAQUE
    let start = ServerLogin::start(
        &mut server_rng,
        &state.opaque_setup,
        opaque_record,
        start_login_request,
        login_lookup.as_slice(),
        ServerLoginParameters::default(),
    )
    .map_err(|_| ApiError::InternalError)?;

    // Génération d'un nonce unique pour retrouver le server_login_state
    let mut nonce = [0u8; 32];
    OsRng.fill_bytes(&mut nonce);
    let nonce = URL_SAFE_NO_PAD.encode(nonce);
    let redis_key = format!("opaque:login:{}", &nonce);

    let state_bytes: Vec<u8> = start.state.serialize().to_vec();
    let ttl_seconds: u64 = 120;
    // Sauvegarde du server_login_state dans Redis avec expiration
    let _: () = state
        .redis
        .set_ex(&redis_key, state_bytes, ttl_seconds)
        .await
        .map_err(|_| ApiError::InternalError)?;

    // Création de la réponse et envoi
    let start_login_response =
        base64::engine::general_purpose::STANDARD.encode(start.message.serialize());

    Ok(Json(LoginStartResponse {
        start_login_response,
        nonce,
    }))
}

pub async fn login_finish(
    State(mut state): State<ServerState>,
    Json(payload): Json<LoginFinishRequest>,
) -> Result<Json<String>, ApiError> {
    let redis_key = format!("opaque:login:{}", payload.nonce);

    // Récupération du server_login_state depuis Redis
    let state_bytes: Option<Vec<u8>> = state
        .redis
        .get(&redis_key)
        .await
        .map_err(|_| ApiError::InternalError)?;

    // Si le nonce inconnu / expiré
    let state_bytes: Vec<u8> = match state_bytes {
        Some(v) => v,
        None => return Err(ApiError::Unauthorized),
    };

    // Supprimer one-shot (évite replay)
    let _: () = state
        .redis
        .del(&redis_key)
        .await
        .map_err(|_| ApiError::InternalError)?;

    // Décode/Désérialise le message final du client
    let finish_login_request = base64::engine::general_purpose::STANDARD
        .decode(&payload.finish_login_request)
        .map_err(|_| ApiError::BadRequest("Failed to decode request".to_string()))?;

    let finish_login_request = CredentialFinalization::<DCS>::deserialize(&finish_login_request)
        .map_err(|_| ApiError::BadRequest("Failed to deserialize request".to_string()))?;
    // Désérialiser server_login_state puis finish()
    let server_login_state =
        ServerLogin::<DCS>::deserialize(&state_bytes).map_err(|_| ApiError::InternalError)?;

    let finish = server_login_state
        .finish(finish_login_request, ServerLoginParameters::default())
        .map_err(|_| ApiError::Unauthorized)?; // mauvais mdp ou preuve invalide

    // secret partagé entre le client et le serveur
    let _session_key = finish.session_key;

    Ok(Json("Login successful".to_string()))
}
