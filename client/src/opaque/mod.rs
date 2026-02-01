use crate::api;
use crate::opaque::models::{RegisterFinishRequest, RegisterStartRequest, RegisterStartResponse};
use base64::Engine;
use inquire::{Password, Text};
use opaque_ke::{
    CipherSuite, ClientRegistration, ClientRegistrationFinishParameters, RegistrationResponse,
};
use rand::rngs::OsRng;

pub mod models;

// EXEMPLE DE LA DOC, A MODIFIER ?
// A DEPLACER DANS UN ENDROIT PLUS GLOBAL ?
struct Default;
impl CipherSuite for Default {
    type OprfCs = opaque_ke::Ristretto255;
    type KeyExchange = opaque_ke::TripleDh<opaque_ke::Ristretto255, sha2::Sha512>;
    type Ksf = opaque_ke::ksf::Identity;
}

pub fn register() {
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

    // Démarrer le register avec OPAQUE
    let start = ClientRegistration::<Default>::start(&mut client_rng, &password)
        .expect("ClientRegistration::start failed");

    // Recup du message et conversion en bytes puis base64 pour envoi au serveur
    let start_message_bytes = start.message.serialize();
    let start_message_b64 = base64::engine::general_purpose::STANDARD.encode(start_message_bytes);

    // Recup du state (pour register_finish)
    let start_state = start.state;

    let payload = RegisterStartRequest {
        username: &username,
        start_request: start_message_b64.to_string(),
    };
    let registration_response = match api::opaque_register(payload) {
        Ok(response) => response,
        Err(e) => {
            eprintln!("Failed to send registration start request: {}", e);
            return;
        }
    };

    // Décoder la registration réponse du serveur
    let response_bytes = base64::engine::general_purpose::STANDARD
        .decode(&registration_response)
        .expect("Decoding base64 failed");
    let response = RegistrationResponse::<Default>::deserialize(&response_bytes)
        .expect("RegistrationResponse::deserialize failed");

    // Démarrer le finish avec la réponse du serveur
    let finish = start_state
        .finish(
            &mut client_rng,
            &password,
            response,
            ClientRegistrationFinishParameters::default(),
        )
        .expect("ClientRegistration::finish failed");

    // Recup du message et conversion en bytes puis base64 pour envoi au serveur
    let finish_message_bytes = finish.message.serialize();
    let finish_message_b64 = base64::engine::general_purpose::STANDARD.encode(finish_message_bytes);

    let payload = RegisterFinishRequest {
        username: &username,
        finish_request: finish_message_b64.to_string(),
    };

    match api::opaque_register_finish(payload) {
        Ok(_) => {
            println!("Registration completed successfully.");
        }
        Err(e) => {
            eprintln!("Failed to send registration finish request: {}", e);
        }
    }
}
