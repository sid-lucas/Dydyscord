use crate::models::{
    CreateUserPayload, RegisterStartRequest, RegisterStartResponse, User
};
use axum::{Json, extract::State, http::StatusCode};
use sqlx::PgPool;
use base64::Engine;
use opaque_ke::RegistrationRequest;
use opaque_ke::ServerRegistration;
use crate::opaque::{make_server_setup, OpaqueCiphersuite};

pub async fn root() -> (StatusCode, &'static str) {
    (StatusCode::OK, "Dydyscord Server is running!")
}

pub async fn create_user(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUserPayload>,
) -> Result<(StatusCode, Json<User>), StatusCode> {
    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (username) VALUES ($1) RETURNING id, username",
        payload.username
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(user)))
}


pub async fn RegistrationRequest(
    Json(payload): Json<RegisterStartRequest>,
) -> Result<(StatusCode, Json<RegisterStartResponse>), StatusCode> {

    // Récupération et décodage de la requête du client
    let registration_request_bytes = base64::engine::general_purpose::STANDARD
        .decode(&payload.registration_request)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let registration_request =
        RegistrationRequest::<OpaqueCiphersuite>::deserialize(&registration_request_bytes)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Récupération du nom d'utilisateur
    let username = payload.username;

    // Réalisation de la Registration Response côté serveur pour le client
    // ATTENTION : pour l'instant le ServerSetup est recréé à chaque requête, il faudra le persister
    let registration_response = ServerRegistration::<OpaqueCiphersuite>::start(
        &make_server_setup(),
        registration_request,
        username.as_bytes(),
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let registration_response = base64::engine::general_purpose::STANDARD
        .encode(registration_response.message.serialize());

    // Création de la réponse et envoi
    let response = RegisterStartResponse {
        registration_response,
    };
    Ok((StatusCode::OK, Json(response)))
}
