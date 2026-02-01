use crate::ServerState;
use crate::database::models::{CreateUserPayload, User};
use crate::opaque::OpaqueCiphersuite;
use crate::opaque::models::{RegisterFinishRequest, RegisterStartRequest, RegisterStartResponse};
use axum::{Json, extract::State, http::StatusCode};
use base64::Engine;
use opaque_ke::RegistrationRequest;
use opaque_ke::RegistrationUpload;
use opaque_ke::ServerRegistration;

pub async fn root() -> (StatusCode, &'static str) {
    (StatusCode::OK, "Dydyscord Server is running!")
}

pub async fn create_user(
    State(state): State<ServerState>,
    Json(payload): Json<CreateUserPayload>,
) -> Result<(StatusCode, Json<User>), StatusCode> {
    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (username) VALUES ($1) RETURNING id, username",
        payload.username
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn register_start(
    State(state): State<ServerState>,
    Json(payload): Json<RegisterStartRequest>,
) -> Result<(StatusCode, Json<RegisterStartResponse>), StatusCode> {
    // Récupération et décodage de la requête du client
    let registration_request_bytes = base64::engine::general_purpose::STANDARD
        .decode(&payload.start_request)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let registration_request =
        RegistrationRequest::<OpaqueCiphersuite>::deserialize(&registration_request_bytes)
            .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Récupération du nom d'utilisateur
    let username = payload.username;

    // TODO :
    // Calculer le login_lookup avec le server_pepper et username, et stocker qqn part

    // Réalisation de la Registration Response côté serveur pour le client
    // A CHANGER : le credential_identifier doit être le login_lookup calculé et pas l'username directement...
    let registration_response = ServerRegistration::<OpaqueCiphersuite>::start(
        &state.opaque_setup,
        registration_request,
        username.as_bytes(),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let registration_response =
        base64::engine::general_purpose::STANDARD.encode(registration_response.message.serialize());

    // Création de la réponse et envoi
    let response = RegisterStartResponse {
        start_response: registration_response,
    };
    Ok((StatusCode::OK, Json(response)))
}

pub async fn register_finish(
    Json(payload): Json<RegisterFinishRequest>,
) -> Result<(StatusCode), StatusCode> {
    // Récupération et décodage de la requête du client
    let upload_bytes = base64::engine::general_purpose::STANDARD
        .decode(&payload.finish_request)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let upload = RegistrationUpload::<OpaqueCiphersuite>::deserialize(&upload_bytes)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let opaque_record = ServerRegistration::finish(upload);

    // Récupération du nom d'utilisateur
    let username = payload.username;

    // TODO :
    // Recalculer le login_lookup avec le server_pepper et username
    // Stocker le opaque_record dans la BDD associé au login_lookup

    Ok(StatusCode::OK)
}
