use OpaqueCipherSuite as Default;
use base64::Engine;

use crate::error::AppError;
use crate::transport::http;
use opaque_ke::argon2::Argon2;
use opaque_ke::{
    CipherSuite, ClientLogin, ClientLoginFinishParameters, ClientRegistration,
    ClientRegistrationFinishParameters, CredentialResponse, RegistrationResponse,
};
use rand::rngs::OsRng;
use std::{thread, time::Duration};
use uuid::Uuid;

use crate::auth::error::AuthError;
use serde::{Deserialize, Serialize};

// Register

#[derive(Serialize)]
pub struct RegisterStartRequest<'a> {
    pub username: &'a str,
    pub start_register_request: String, // base64
}

#[derive(Deserialize)]
pub struct RegisterStartResponse {
    pub start_register_response: String, // base64
}

#[derive(Serialize)]
pub struct RegisterFinishRequest<'a> {
    pub username: &'a str,
    pub finish_register_request: String, // base64
}

// Login

#[derive(Serialize)]
pub struct LoginStartRequest<'a> {
    pub username: &'a str,
    pub start_login_request: String, // base64
}

#[derive(Deserialize)]
pub struct LoginStartResponse {
    pub start_login_response: String, // base64
    pub user_id: Uuid, // aussi utilisé comme clé-valeur pour retrouver le server_login_state
}

#[derive(Serialize)]
pub struct LoginFinishRequest {
    pub finish_login_request: String, // base64
    pub user_id: Uuid,                // clé-valeur pour retrouver le server_login_state
}

struct OpaqueCipherSuite;

impl CipherSuite for OpaqueCipherSuite {
    type OprfCs = opaque_ke::Ristretto255;
    type KeyExchange = opaque_ke::TripleDh<opaque_ke::Ristretto255, sha2::Sha512>;
    type Ksf = Argon2<'static>;
}

pub struct LoginResult {
    pub id: Uuid,
    pub export_key: Vec<u8>, // TODO REVOIR avec SecretSlice<u8>
    pub session_key: Vec<u8>,
}

pub fn register(username: &str, password: &str) -> Result<(), AppError> {
    let mut client_rng = OsRng;

    // Démarrer le register client avec OPAQUE
    let start = ClientRegistration::<Default>::start(&mut client_rng, &password.as_bytes())
        .map_err(|_| AuthError::OpaqueRegisterStart)?;

    // Préparation de la request à envoyer au serveur
    let start_register_request =
        base64::engine::general_purpose::STANDARD.encode(start.message.serialize());

    // Call API (envoi requête et réception réponse)
    let response = http::opaque_register(RegisterStartRequest {
        username: &username,
        start_register_request,
    })?;
    let register_response_b64 = response.start_register_response;

    // Response base64 -> bytes
    let register_response_bytes = base64::engine::general_purpose::STANDARD
        .decode(&register_response_b64)
        .map_err(|_| AuthError::OpaqueDecode)?;
    // Response désérialisation
    let register_response = RegistrationResponse::<Default>::deserialize(&register_response_bytes)
        .map_err(|_| AuthError::OpaqueDeserialize)?;

    // Démarrer le finish avec la réponse du serveur
    let finish = start
        .state
        .finish(
            &mut client_rng,
            &password.as_bytes(),
            register_response,
            ClientRegistrationFinishParameters::default(),
        )
        .map_err(|_| AuthError::OpaqueRegisterFinish)?;

    // Préparation de la request à envoyer au serveur
    let finish_register_request =
        base64::engine::general_purpose::STANDARD.encode(finish.message.serialize());

    // Call API (envoi requête et réception réponse)
    http::opaque_register_finish(RegisterFinishRequest {
        username: &username,
        finish_register_request,
    })?;

    Ok(())
}

pub fn login(username: &str, password: &str) -> Result<LoginResult, AppError> {
    let mut client_rng = OsRng;

    // Démarrer le login client avec OPAQUE
    let start = ClientLogin::<Default>::start(&mut client_rng, &password.as_bytes())
        .map_err(|_| AuthError::OpaqueLoginStart)?;

    // Préparation de la request à envoyer au serveur
    let start_login_request =
        base64::engine::general_purpose::STANDARD.encode(start.message.serialize());

    // TODO : fix timing attack

    // Call API (envoi requête et réception réponse)
    let response = http::opaque_login(LoginStartRequest {
        username: &username,
        start_login_request,
    })?;
    let (login_response_b64, id) = (response.start_login_response, response.user_id);

    // Response base64 -> bytes
    let login_response_bytes = base64::engine::general_purpose::STANDARD
        .decode(&login_response_b64)
        .map_err(|_| AuthError::OpaqueDecode)?;
    // Response désérialisation
    let login_response = CredentialResponse::<Default>::deserialize(&login_response_bytes)
        .map_err(|_| AuthError::OpaqueDeserialize)?;

    // Finaliser le login avec la réponse du serveur
    let finish = start
        .state
        .finish(
            &mut client_rng,
            &password.as_bytes(),
            login_response,
            ClientLoginFinishParameters::default(),
        )
        .map_err(|_| AuthError::OpaqueLoginFinish)?;

    let export_key = finish.export_key.to_vec();
    let session_key = finish.session_key.to_vec();

    // Préparation de la request à envoyer au serveur+
    let finish_login_request =
        base64::engine::general_purpose::STANDARD.encode(finish.message.serialize());

    // Call API (envoi requête et réception réponse)
    http::opaque_login_finish(LoginFinishRequest {
        finish_login_request,
        user_id: id,
    })?;

    Ok(LoginResult {
        id,
        export_key,
        session_key,
    })
}
