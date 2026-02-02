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
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::Input => write!(f, "input error"),
            ClientError::Api(s) => write!(f, "api error: {s}"),
            ClientError::Base64 => write!(f, "invalid base64"),
            ClientError::Opaque => write!(f, "opaque protocol error"),
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

    // Recup du state (pour register_finish)
    let start_state = start.state;

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
    let finish = start_state
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



pub fn login() {
    let username = Text::new("Enter your username:").prompt();
    let username = match username {
        Ok(username) => username,
        Err(_) => {
            eprintln!("Failed to read username.");
            return;
        }
    };
    let password = Password::new("Enter your password:").prompt();
    let password = match password {
        Ok(password) => password.into_bytes(),
        Err(_) => {
            eprintln!("Failed to read password.");
            return;
        }
    };

    let mut client_rng = OsRng;

    // Démarrer le login avec OPAQUE
    let start = ClientLogin::<Default>::start(&mut client_rng, &password)
        .expect("ClientLogin::start failed");

    // Recup du message et conversion en bytes puis base64 pour envoi au serveur
    let start_message_bytes = start.message.serialize();
    let start_message_b64 = base64::engine::general_purpose::STANDARD.encode(start_message_bytes);

    // Envoi de la requête login au serveur et réception de la réponse
    let payload = LoginStartRequest {
        username: &username,
        start_request: start_message_b64.to_string(),
    };
    let login_response = match api::opaque_login(payload) {
        Ok(response) => response,
        Err(e) => {
            eprintln!("Failed to send login start request: {}", e);
            return;
        }
    };

    // Décoder la login réponse du serveur
    let response_bytes = base64::engine::general_purpose::STANDARD
        .decode(&login_response)
        .expect("Decoding base64 failed");

        
    // Finaliser le login avec la réponse du serveur
    let finish = start.state.finish(
        &mut client_rng,
        &password,
        CredentialResponse::<Default>::deserialize(&response_bytes).unwrap(),
        ClientLoginFinishParameters::default(),
    );
    if finish.is_err() { // Si mdp incorrect
        eprintln!("Login failed: Invalid credentials.");
        return;
    }
    
    let finish = finish.unwrap();

    // Recup du message et conversion en bytes puis base64 pour envoi au serveur
    let finish_message_bytes = finish
    .message
    .serialize();
    let finish_message_b64 = base64::engine::general_purpose::STANDARD.encode(finish_message_bytes);

    let payload = LoginFinishRequest {
        username: &username,
        finish_request: finish_message_b64.to_string(),
    };

    match api::opaque_login_finish(payload) {
        Ok(_) => {
            println!("Login completed successfully.");
        }
        Err(e) => {
            eprintln!("Failed to send login finish request: {}", e);
        }
    }

    // CA c'est la master_key (dérivée du mdp) qui servira a dériver plein de sous-clés de chiffrement
    // Uniquement connue du client.
    let export_key = finish.export_key;
    // CA c'est le secret partagé entre le client et le serveur
    let session_key = finish.session_key;

}