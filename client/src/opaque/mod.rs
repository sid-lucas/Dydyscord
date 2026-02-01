use rand::RngCore;
use rand::rngs::OsRng;
use opaque_ke::{
    CipherSuite, ClientRegistration, ClientRegistrationFinishParameters, RegistrationResponse
};
use base64::Engine;
use crate::opaque::models::{
    RegisterStartRequest, RegisterStartResponse, RegisterFinishRequest
};

pub mod models;

// EXEMPLE DE LA DOC, A MODIFIER ?
// A DEPLACER DANS UN ENDROIT PLUS GLOBAL ?
struct Default;
impl CipherSuite for Default { 
    type OprfCs = opaque_ke::Ristretto255;
    type KeyExchange = opaque_ke::TripleDh<opaque_ke::Ristretto255, sha2::Sha512>;
    type Ksf = opaque_ke::ksf::Identity;
}
// Variable temporaire pour les tests
const TEMP_USERNAME: &str = "my_username";


pub fn register(password: &[u8]) {
    let mut client_rng = OsRng;

    // Démarrer le register avec OPAQUE
    let start = ClientRegistration::<Default>::start(&mut client_rng, password)
        .expect("ClientRegistration::start failed");

    // Recup du message et conversion en bytes puis base64 pour envoi au serveur
    let start_message_bytes = start.message.serialize();
    let start_message_b64 = base64::engine::general_purpose::STANDARD.encode(start_message_bytes);

    // Recup du state (pour register_finish)
    let start_state = start.state;

    // Curl du client sur le serv :
    // TODO : Séparer dans un fichier qui s'occupe du networking et remplacer par appel fonction :
    let url = "http://0.0.0.0:3000/register/start";
    let payload = RegisterStartRequest {
        username: TEMP_USERNAME.to_string(),
        start_request: start_message_b64.to_string(),
    };

    // TODO : Remplacer aussi par le fichier networking qui retourne juste la réponse ici
    let mut registration_response = String::new();
    let client = reqwest::blocking::Client::new();
    match client.post(url).json(&payload).send() {
        Ok(response) => {
            if response.status().is_success() {
                let response_payload = response.json::<RegisterStartResponse>().unwrap();
                registration_response = response_payload.start_response;
                println!("Serveur response: {}", registration_response);
            }
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }

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
            password,
            response,
            ClientRegistrationFinishParameters::default(),
        )
        .expect("ClientRegistration::finish failed");

    // Recup du message et conversion en bytes puis base64 pour envoi au serveur
    let finish_message_bytes = finish.message.serialize();
    let finish_message_b64 = base64::engine::general_purpose::STANDARD.encode(finish_message_bytes);




    // Curl du client sur le serv :
    // TODO : Séparer dans un fichier qui s'occupe du networking et remplacer par appel fonction :
    let url = "http://0.0.0.0:3000/register/finish";
    let payload = RegisterFinishRequest {
        username: TEMP_USERNAME.to_string(),
        finish_request: finish_message_b64.to_string(),
    };

    // TODO : Remplacer aussi par le fichier networking qui retourne juste la réponse ici
    match client.post(url).json(&payload).send() {
        Ok(response) => {
            if response.status().is_success() {
                println!("Final registration upload sent successfully.");
            }
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }

}
