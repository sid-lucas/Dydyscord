use DefaultCipherSuite as DCS;
use base64::Engine;
use opaque_ke::CipherSuite;
use opaque_ke::ServerSetup;
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
use rand::RngCore;
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
    pub user_id: Uuid, // aussi utilisé comme clé-valeur pour retrouver le server_login_state
}

#[derive(Deserialize)]
pub struct LoginFinishRequest {
    pub finish_login_request: String, // base64
    pub user_id: String,              // clé-valeur pour retrouver le server_login_state
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
    // Récupération du start_register_request du client et décodage/désérialisation
    let decoded_request = base64::engine::general_purpose::STANDARD
        .decode(&payload.start_register_request)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let start_register_request = RegistrationRequest::<DCS>::deserialize(&decoded_request)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

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
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Si user existe, erreur :
    match user {
        Some(_) => return Err(StatusCode::CONFLICT), // username déjà pris
        None => (),                                  // continue normalement
    }

    // Démarrer le register server avec OPAQUE
    let start = ServerRegistration::<DCS>::start(
        &state.opaque_setup,
        start_register_request,
        login_lookup.as_slice(),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Préparation de la request à envoyer au serveur
    let start_register_response =
        base64::engine::general_purpose::STANDARD.encode(start.message.serialize());

    // Création de la réponse et envoi
    let response = RegisterStartResponse {
        start_register_response,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn register_finish(
    State(state): State<ServerState>,
    Json(payload): Json<RegisterFinishRequest>,
) -> Result<StatusCode, StatusCode> {
    // Récupération du finish_register_request du client et décodage/désérialisation
    let decoded_request = base64::engine::general_purpose::STANDARD
        .decode(&payload.finish_register_request)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let finish_register_request = RegistrationUpload::<DCS>::deserialize(&decoded_request)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Finalisation du register en créant le opaque_record à stocker
    let opaque_record = ServerRegistration::finish(finish_register_request)
        .serialize()
        .to_vec();

    // Récupération du nom d'utilisateur
    let username = payload.username;
    // Recalculer le login_lookup avec le server_pepper et username
    let login_lookup = login_lookup(&state.pepper.expose_secret(), &username);

    // Stocker le opaque_record dans la BDD associé au login_lookup
    sqlx::query!(
        "INSERT INTO users (login_lookup, opaque_record) VALUES ($1, $2)",
        login_lookup,
        opaque_record,
    )
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

pub async fn login_start(
    State(mut state): State<ServerState>,
    Json(payload): Json<LoginStartRequest>,
) -> Result<(StatusCode, Json<LoginStartResponse>), StatusCode> {
    // Récupération du start_login_request du client et décodage/désérialisation
    let start_login_request = CredentialRequest::<DCS>::deserialize(
        &base64::engine::general_purpose::STANDARD
            .decode(&payload.start_login_request)
            .map_err(|_| StatusCode::BAD_REQUEST)?,
    )
    .map_err(|_| StatusCode::BAD_REQUEST)?;

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
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Désérialisation du opaque_record si user existe
    let (opaque_record, user_id) = match user {
        Some(user) => {
            let opaque_record = ServerRegistration::<DCS>::deserialize(&user.opaque_record)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            (Some(opaque_record), user.id)
        }
        None => return Err(StatusCode::NOT_FOUND),
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
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Sauvegarde du server_login_state dans Redis avec expiration
    let redis_key = format!("opaque:login:{}", &user_id);
    let state_bytes: Vec<u8> = start.state.serialize().to_vec();
    let ttl_seconds: u64 = 120;
    let _: () = state
        .redis
        .set_ex(&redis_key, state_bytes, ttl_seconds)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Création de la réponse et envoi
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
    State(mut state): State<ServerState>,
    Json(payload): Json<LoginFinishRequest>,
) -> Result<CookieJar, StatusCode> {
    // Création de la clé avec le user_id
    let redis_key = format!("opaque:login:{}", payload.user_id);

    // Récupération du server_login_state depuis Redis
    let state_bytes: Option<Vec<u8>> = state
        .redis
        .get(&redis_key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Si la récupération n'a pas fonctionné (key invalide)
    let state_bytes: Vec<u8> = match state_bytes {
        Some(v) => v,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // Supprimer one-shot (évite replay)
    let _: () = state
        .redis
        .del(&redis_key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Décode/Désérialise le message final du client
    let finish_login_request = base64::engine::general_purpose::STANDARD
        .decode(&payload.finish_login_request)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let finish_login_request = CredentialFinalization::<DCS>::deserialize(&finish_login_request)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Désérialiser server_login_state puis finish()
    let server_login_state = ServerLogin::<DCS>::deserialize(&state_bytes)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let finish = server_login_state
        .finish(finish_login_request, ServerLoginParameters::default())
        .map_err(|_| StatusCode::UNAUTHORIZED)?; // mauvais mdp ou preuve invalide

    // secret partagé entre le client et le serveur
    let _session_key = finish.session_key;

    // Création du JWT intermédiaire (auth)
    let id = payload.user_id.to_string();
    let token = jwt::create_jwt(id.as_str(), jwt::TokenType::Auth)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let cookie = Cookie::build((constant::AUTH_HEADER, token))
        .http_only(false) // TODO change
        .secure(false) // TODO Change: true interdit l'envoi un HTTP. -> false pour test local pour l'instant.
        .same_site(SameSite::Strict)
        .path("/")
        .build(); // si erreur: remplace .build() par .finish()

    let jar = CookieJar::new().add(cookie);

    Ok(jar)
}
