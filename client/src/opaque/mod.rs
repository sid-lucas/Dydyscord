use core::fmt;

use crate::api;
use crate::opaque::models::{
    LoginFinishRequest, LoginStartRequest, LoginStartResponse, RegisterFinishRequest,
    RegisterStartRequest, RegisterStartResponse,
};
use base64::Engine;
use inquire::{Password, Text};
use opaque_ke::argon2::Argon2;
use opaque_ke::{
    CipherSuite, ClientLogin, ClientLoginFinishParameters, ClientRegistration,
    ClientRegistrationFinishParameters, CredentialResponse, RegistrationResponse,
};
use rand::rngs::OsRng;
use reqwest::Error as ReqwestError;
use std::error::Error;
pub mod models;

struct DefaultCipherSuite;

impl CipherSuite for DefaultCipherSuite {
    type OprfCs = opaque_ke::Ristretto255;
    type KeyExchange = opaque_ke::TripleDh<opaque_ke::Ristretto255, sha2::Sha512>;
    type Ksf = Argon2<'static>;
}

pub fn register() -> Result<(), Box<dyn Error>> {
    let username = Text::new("Enter your username:").prompt();
    let username = match username {
        Ok(username) => username,
        Err(_) => return Err("Failed to read username".into()),
    };

    let password = Password::new("Enter your password:").prompt();
    let password = match password {
        Ok(password) => password.into_bytes(),
        Err(_) => return Err("Failed to read password".into()),
    };

    let mut client_rng = OsRng;

    // Démarrer le register client avec OPAQUE
    let start = ClientRegistration::<DefaultCipherSuite>::start(&mut client_rng, &password)
        .expect("Failed to start client registration");

    // Préparation de la request à envoyer au serveur
    let start_register_request =
        base64::engine::general_purpose::STANDARD.encode(start.message.serialize());

    // Call API (envoi requête et réception réponse)
    let register_response_b64 = api::opaque_register(RegisterStartRequest {
        username: &username,
        start_register_request,
    });
    let register_response_b64 = match register_response_b64 {
        Ok(response) => response,
        Err(e) => return Err(e.into()),
    };

    // Response base64 -> bytes
    let register_response_bytes = base64::engine::general_purpose::STANDARD
        .decode(&register_response_b64)
        .expect("Failed to decode base64 register response");
    // Response désérialisation
    let register_response =
        RegistrationResponse::<DefaultCipherSuite>::deserialize(&register_response_bytes)
            .expect("Failed to deserialize register response");

    // Démarrer le finish avec la réponse du serveur
    let finish = start
        .state
        .finish(
            &mut client_rng,
            &password,
            register_response,
            ClientRegistrationFinishParameters::default(),
        )
        .expect("Failed to finish client registration");

    // Préparation de la request à envoyer au serveur
    let finish_register_request =
        base64::engine::general_purpose::STANDARD.encode(finish.message.serialize());

    api::opaque_register_finish(RegisterFinishRequest {
        username: &username,
        finish_register_request,
    })
    .map_err(|e| e)?;

    println!("Registration completed successfully.");

    // CA c'est la master_key (dérivée du mdp) qui servira a dériver plein de sous-clés de chiffrement
    // Uniquement connue du client.
    let export_key = finish.export_key;

    Ok(())
}

pub fn login() -> Result<(), Box<dyn Error>> {
    let username = Text::new("Enter your username:").prompt();
    let username = match username {
        Ok(username) => username,
        Err(_) => return Err("Failed to read username".into()),
    };

    let password = Password::new("Enter your password:")
        .without_confirmation()
        .prompt();
    let password = match password {
        Ok(password) => password.into_bytes(),
        Err(_) => return Err("Failed to read password".into()),
    };

    let mut client_rng = OsRng;

    // Démarrer le login client avec OPAQUE
    let start = ClientLogin::<DefaultCipherSuite>::start(&mut client_rng, &password)
        .expect("Failed to start client login");

    // Préparation de la request à envoyer au serveur
    let start_login_request =
        base64::engine::general_purpose::STANDARD.encode(start.message.serialize());

    // Call API (envoi requête et réception réponse)
    let response = api::opaque_login(LoginStartRequest {
        username: &username,
        start_login_request,
    })
    .map_err(|e| e)?;

    // Response base64 -> bytes
    let login_response_bytes = base64::engine::general_purpose::STANDARD
        .decode(&response.start_login_response)
        .expect("Failed to decode base64 login response");
    // Response désérialisation
    let login_response =
        CredentialResponse::<DefaultCipherSuite>::deserialize(&login_response_bytes)
            .expect("Failed to deserialize login response");

    // Finaliser le login avec la réponse du serveur
    let finish = start
        .state
        .finish(
            &mut client_rng,
            &password,
            login_response,
            ClientLoginFinishParameters::default(),
        )
        .expect("Failed to finish client login");

    // Préparation de la request à envoyer au serveur
    let finish_login_request =
        base64::engine::general_purpose::STANDARD.encode(finish.message.serialize());

    api::opaque_login_finish(LoginFinishRequest {
        finish_login_request,
        nonce: response.nonce,
    })
    .map_err(|e| e)?;

    println!("Login completed successfully.");

    // Ca c'est la master_key (dérivée du mdp) qui servira a dériver plein de sous-clés de chiffrement
    // Uniquement connue du client.
    let _export_key = finish.export_key;

    // Ca c'est le secret partagé entre le client et le serveur
    let _session_key = finish.session_key;

    Ok(())
}
