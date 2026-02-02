use crate::ServerState;
use crate::opaque::OpaqueCiphersuite;
use crate::opaque::models::{ 
    RegisterFinishRequest, RegisterStartRequest, RegisterStartResponse,
    LoginStartRequest, LoginStartResponse, LoginFinishRequest,
};
use axum::{Json, extract::State, http::StatusCode};
use base64::Engine;
use hmac::Mac;
use rand::rngs::OsRng;
use opaque_ke::{
    ClientLogin, ClientLoginFinishParameters, ClientRegistration,
    ClientRegistrationFinishParameters, CredentialFinalization, CredentialRequest,
    CredentialResponse, RegistrationRequest, RegistrationResponse, RegistrationUpload, ServerLogin,
    ServerLoginParameters, ServerRegistration, ServerRegistrationLen, ServerSetup,
};
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
) -> Result<(StatusCode, Json<RegisterStartResponse>), StatusCode> {

    // Récupération du start_register_request du client et décodage/désérialisation
    let start_register_request =
    RegistrationRequest::<OpaqueCiphersuite>::deserialize(
        &base64::engine::general_purpose::STANDARD
            .decode(&payload.start_register_request)
            .map_err(|_| StatusCode::BAD_REQUEST)?,
    )
    .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Récupération du nom d'utilisateur
    let username = payload.username;

    // Calculer le login_lookup avec le server_pepper et username
    let login_lookup = login_lookup(&state.pepper, &username);

    // Démarrer le register server avec OPAQUE
    let start = ServerRegistration::<OpaqueCiphersuite>::start(
        &state.opaque_setup,
        start_register_request,
        login_lookup.as_slice(),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Préparation de la request à envoyer au serveur
    let start_register_response = base64::engine::general_purpose::STANDARD
        .encode(start.message.serialize());

    // Création de la réponse et envoi
    let response = RegisterStartResponse {
        start_register_response,
    };
    Ok((StatusCode::OK, Json(response)))
}

pub async fn register_finish(
    State(state): State<ServerState>,
    Json(payload): Json<RegisterFinishRequest>,
) -> Result<StatusCode, StatusCode> {

    // Récupération du finish_register_request du client et décodage/désérialisation
    let finish_register_request =
    RegistrationUpload::<OpaqueCiphersuite>::deserialize(
        &base64::engine::general_purpose::STANDARD
            .decode(&payload.finish_register_request)
            .map_err(|_| StatusCode::BAD_REQUEST)?,
    )
    .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Finalisation du register en créant le opaque_record à stocker
    let opaque_record = ServerRegistration::finish(finish_register_request).serialize().to_vec();

    // Récupération du nom d'utilisateur
    let username = payload.username;
    // Recalculer le login_lookup avec le server_pepper et username
    let login_lookup = login_lookup(&state.pepper, &username);

    // Stocker le opaque_record dans la BDD associé au login_lookup
    sqlx::query_as!(
        User,
        "INSERT INTO users (login_lookup, opaque_record) VALUES ($1, $2)",
        login_lookup,
        opaque_record,
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

pub async fn login_start(
    State(state): State<ServerState>,
    Json(payload): Json<LoginStartRequest>,
) -> Result<(StatusCode, Json<LoginStartResponse>), StatusCode> {
    
    // Récupération du nom d'utilisateur et login_lookup correspondant
    let username = payload.username;
    let login_lookup = login_lookup(&state.pepper, &username);

    // Récupération du opaque_record associé au login_lookup dans la BDD
    let record = sqlx::query_as!(
        User,
        "SELECT login_lookup, opaque_record FROM users WHERE login_lookup = $1",
        login_lookup,
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let opaque_record = 
        ServerRegistration::<OpaqueCiphersuite>::deserialize(&record.opaque_record).unwrap();

    // Récupération et décodage de la requête du client
    let login_request_bytes = base64::engine::general_purpose::STANDARD
        .decode(&payload.start_request)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut server_rng = OsRng;

    // Réalisation de la Login Response côté serveur pour le client
    let login_start_result = ServerLogin::start(
        &mut server_rng,
        &state.opaque_setup,
        Some(opaque_record),
        CredentialRequest::deserialize(&login_request_bytes).unwrap(),
        login_lookup.as_slice(),
        ServerLoginParameters::default(),
    )
    .unwrap();

    // Création de la réponse et envoi
    let login_response = base64::engine::general_purpose::STANDARD
        .encode(login_start_result.message.serialize());

    let response = LoginStartResponse {
        start_response: login_response, // TODO SIMPLIFIER AVEC NOM COMMUN
    };

    Ok((StatusCode::OK, Json(response)))
}

pub async fn login_finish(
    State(state): State<ServerState>,
    Json(payload): Json<LoginFinishRequest>,
) -> Result<StatusCode, StatusCode> {

    let finish_bytes = base64::engine::general_purpose::STANDARD
        .decode(&payload.finish_request)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let finish = CredentialFinalization::<OpaqueCiphersuite>::deserialize(&finish_bytes)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    //let session_key = finish.session_key;

    Ok(StatusCode::OK)
}
