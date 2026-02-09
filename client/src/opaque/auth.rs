use crate::api;
use crate::error::ClientError;
use crate::mls::storage;
use crate::opaque::models::{
    LoginFinishRequest, LoginStartRequest, RegisterFinishRequest, RegisterStartRequest,
};
use crate::session;
use DefaultCipherSuite as DCS;
use base64::Engine;
use inquire::{Password, Text};
use opaque_ke::argon2::Argon2;
use opaque_ke::{
    CipherSuite, ClientLogin, ClientLoginFinishParameters, ClientRegistration,
    ClientRegistrationFinishParameters, CredentialResponse, RegistrationResponse,
};
use rand::rngs::OsRng;
use std::{thread, time::Duration};
use uuid::Uuid;

struct DefaultCipherSuite;

impl CipherSuite for DefaultCipherSuite {
    type OprfCs = opaque_ke::Ristretto255;
    type KeyExchange = opaque_ke::TripleDh<opaque_ke::Ristretto255, sha2::Sha512>;
    type Ksf = Argon2<'static>;
}

pub struct LoginResult {
    pub id: Uuid,
    pub export_key: Vec<u8>,
    pub session_key: Vec<u8>,
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
    let start = ClientRegistration::<DCS>::start(&mut client_rng, &password)
        .expect("Failed to start client registration");

    // Préparation de la request à envoyer au serveur
    let start_register_request =
        base64::engine::general_purpose::STANDARD.encode(start.message.serialize());

    // Call API (envoi requête et réception réponse)
    let response = api::opaque_register(RegisterStartRequest {
        username: &username,
        start_register_request,
    });
    let register_response_b64 = match response {
        Ok(response) => response.start_register_response,
        Err(e) => return Err(e.into()),
    };

    // Response base64 -> bytes
    let register_response_bytes = base64::engine::general_purpose::STANDARD
        .decode(&register_response_b64)
        .expect("Failed to decode base64 register response");
    // Response désérialisation
    let register_response = RegistrationResponse::<DCS>::deserialize(&register_response_bytes)
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

    // Call API (envoi requête et réception réponse)
    api::opaque_register_finish(RegisterFinishRequest {
        username: &username,
        finish_register_request,
    })
    .map_err(|e| e.into())?;

    // TODO : utiliser
    // CA c'est la master_key (dérivée du mdp) qui servira a dériver plein de sous-clés de chiffrement
    // Uniquement connue du client.
    let export_key = finish.export_key;

    Ok(())
}

pub fn login() -> Result<LoginResult, ClientError> {
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
    let start = ClientLogin::<DCS>::start(&mut client_rng, &password)
        .expect("Failed to start client login");

    // Préparation de la request à envoyer au serveur
    let start_login_request =
        base64::engine::general_purpose::STANDARD.encode(start.message.serialize());

    // Délai aléatoire pour éviter les attaques timing
    let random_delay = Duration::from_millis(300 + (rand::random::<u64>() % 200));
    thread::sleep(random_delay);

    // Call API (envoi requête et réception réponse)
    let response = api::opaque_login(LoginStartRequest {
        username: &username,
        start_login_request,
    });
    let (login_response_b64, nonce, id) = match response {
        Ok(response) => (
            response.start_login_response,
            response.nonce,
            response.user_id,
        ),
        Err(e) => return Err(e.into()),
    };

    // Response base64 -> bytes
    let login_response_bytes = base64::engine::general_purpose::STANDARD
        .decode(&login_response_b64)
        .expect("Failed to decode base64 login response");
    // Response désérialisation
    let login_response = CredentialResponse::<DCS>::deserialize(&login_response_bytes)
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
        .map_err(|_| ClientError::LoginFailed)?;

    let export_key = finish.export_key.to_vec();
    let session_key = finish.session_key.to_vec();

    //////////////
    // ICI CHECK SI A LA DB ET SI ON PEUT LA LIRE AVEC L'EXPORT_KEY POUR SAVOIR SI NOUVEAU DEVICE OU NON?
    // regarde ici si c'est un nouveau device:

    // Reconcile + récupère si le device est reconnu avant potentielle init de la db
    let new_device = !session::reconcile_device_storage(&id.to_string());

    // Récupèration/Création de la clé de chiffrement de la db
    let db_key = storage::get_or_create_db_key(&id.to_string(), &export_key).unwrap();

    // LIRE device_id de la db local sqlcipher (si n'existe pas, demander au serveur de le créer de son côté et de nou l'envoyer.)
    //let db_key = get_or_create_db_key(&user_id, &export_key)?;
    //let conn = open_sqlcipher(&db_key)?;
    //let device_id = read_device_id(&conn)?; // SELECT device_id FROM device_meta LIMIT 1

    // Préparation de la request à envoyer au serveur+
    let finish_login_request =
        base64::engine::general_purpose::STANDARD.encode(finish.message.serialize());

    // Call API (envoi requête et réception réponse)
    api::opaque_login_finish(LoginFinishRequest {
        finish_login_request,
        nonce,
    })
    .map_err(|e| e.into())?;

    Ok(LoginResult {
        id,
        export_key,
        session_key,
    })
}
