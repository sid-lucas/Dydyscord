use crate::api;
use crate::opaque::models::{LoginFinishRequest, LoginStartRequest, LoginStartResponse, RegisterFinishRequest, RegisterStartRequest, RegisterStartResponse};
use base64::Engine;
use inquire::{Password, Text};
use opaque_ke::{
    CipherSuite, ClientLogin, ClientLoginFinishParameters, ClientRegistration, ClientRegistrationFinishParameters, CredentialResponse, RegistrationResponse
};
use rand::rngs::OsRng;

pub mod models;

#[derive(Debug)]
pub enum ClientError {
    Input,
    Api(String),
    Base64,
    Opaque,
    InvalidCredentials,
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::Input => write!(f, "input error"),
            ClientError::Api(s) => write!(f, "api error: {s}"),
            ClientError::Base64 => write!(f, "invalid base64"),
            ClientError::Opaque => write!(f, "opaque protocol error"),
            ClientError::InvalidCredentials => write!(f, "user unknown or invalid credentials"),
        }
    }
}

impl std::error::Error for ClientError {}

// EXEMPLE DE LA DOC, A MODIFIER ?
// A DEPLACER DANS UN ENDROIT PLUS GLOBAL ?
struct Default;
impl CipherSuite for Default {
    type OprfCs = opaque_ke::Ristretto255;
    type KeyExchange = opaque_ke::TripleDh<opaque_ke::Ristretto255, sha2::Sha512>;
    type Ksf = opaque_ke::ksf::Identity;
}

pub fn register() -> Result<(), ClientError> {
    let username = Text::new("Enter your username:")
        .prompt()
        .map_err(|_| ClientError::Input)?;

    let password = Password::new("Enter your password:")
        .prompt()
        .map_err(|_| ClientError::Input)?
        .into_bytes();

    let mut client_rng = OsRng;

    // Démarrer le register client avec OPAQUE
    let start = ClientRegistration::<Default>::start(
        &mut client_rng, 
        &password
    ).map_err(|_| ClientError::Opaque)?;

    // Préparation de la request à envoyer au serveur
    let start_register_request = base64::engine::general_purpose::STANDARD
        .encode(start.message.serialize());

    // Call API (envoi requête et réception réponse)
    let register_response_b64 = api::opaque_register(RegisterStartRequest {
        username: &username,
        start_register_request,
    })
    .map_err(|e| ClientError::Api(e.to_string()))?;

    // Response base64 -> bytes
    let register_response_bytes = base64::engine::general_purpose::STANDARD
        .decode(&register_response_b64)
        .map_err(|_| ClientError::Base64)?;
    // Response désérialisation
    let register_response = RegistrationResponse::<Default>::deserialize(&register_response_bytes)
        .map_err(|_| ClientError::Opaque)?;

    // Démarrer le finish avec la réponse du serveur
    let finish = start.state
        .finish(
            &mut client_rng,
            &password,
            register_response,
            ClientRegistrationFinishParameters::default(),
        )
        .map_err(|_| ClientError::Opaque)?;

    // Préparation de la request à envoyer au serveur
    let finish_register_request = base64::engine::general_purpose::STANDARD
        .encode(finish.message.serialize());

    api::opaque_register_finish(RegisterFinishRequest {
        username: &username,
        finish_register_request,
    })
    .map_err(|e| ClientError::Api(e.to_string()))?;

    println!("Registration completed successfully.");





    // CA c'est la master_key (dérivée du mdp) qui servira a dériver plein de sous-clés de chiffrement
    // Uniquement connue du client.
    let export_key = finish.export_key;

    Ok(())
}



pub fn login() -> Result<(), ClientError> {
    let username = Text::new("Enter your username:")
        .prompt()
        .map_err(|_| ClientError::Input)?;

    let password = Password::new("Enter your password:")
        .without_confirmation()
        .prompt()
        .map_err(|_| ClientError::Input)?
        .into_bytes();

    let mut client_rng = OsRng;

    // Démarrer le login client avec OPAQUE
    let start = ClientLogin::<Default>::start(
        &mut client_rng, 
        &password
    ).map_err(|_| ClientError::Opaque)?;

    // Préparation de la request à envoyer au serveur
    let start_login_request = base64::engine::general_purpose::STANDARD
        .encode(start.message.serialize());

    // Call API (envoi requête et réception réponse)
    let response = api::opaque_login(LoginStartRequest {
        username: &username,
        start_login_request,
    })
    .map_err(|e| ClientError::Api(e.to_string()))?;

    // Response base64 -> bytes
    let login_response_bytes = base64::engine::general_purpose::STANDARD
        .decode(&response.start_login_response)
        .map_err(|_| ClientError::Base64)?;
    // Response désérialisation
    let login_response = CredentialResponse::<Default>::deserialize(&login_response_bytes)
        .map_err(|_| ClientError::Opaque)?;

    // Finaliser le login avec la réponse du serveur
    let finish = start.state
        .finish(
            &mut client_rng,
            &password,
            login_response,
            ClientLoginFinishParameters::default(),
        )
        .map_err(|_| ClientError::InvalidCredentials)?;

    // Préparation de la request à envoyer au serveur
    let finish_login_request = base64::engine::general_purpose::STANDARD
        .encode(finish.message.serialize());

    api::opaque_login_finish(LoginFinishRequest {
        finish_login_request,
        nonce: response.nonce,
    })
    .map_err(|e| ClientError::Api(e.to_string()))?;

    println!("Login completed successfully.");


    // Ca c'est la master_key (dérivée du mdp) qui servira a dériver plein de sous-clés de chiffrement
    // Uniquement connue du client.
    let _export_key = finish.export_key;
    
    // Ca c'est le secret partagé entre le client et le serveur
    let _session_key = finish.session_key;

    Ok(())
}