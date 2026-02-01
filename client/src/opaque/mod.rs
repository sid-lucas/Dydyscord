use opaque_ke::ClientRegistration;
use rand::RngCore;
use rand::rngs::OsRng;
use opaque_ke::CipherSuite;
use base64::Engine;
use serde::Serialize;

struct Default;

impl CipherSuite for Default { // EXEMPLE DE LA DOC, A MODIFIER ?
    type OprfCs = opaque_ke::Ristretto255;
    type KeyExchange = opaque_ke::TripleDh<opaque_ke::Ristretto255, sha2::Sha512>;
    type Ksf = opaque_ke::ksf::Identity;
}

const TEMP_USERNAME: &str = "my_username";
const TEMP_PASSWORD: &[u8] = b"My_password-123";

#[derive(Serialize)]
struct RegistrationRequest {
    username: String,
    registration_request: String,
}

pub fn register_start() {
    let mut client_rng = OsRng;
    // Démarrer le register avec OPAQUE
    let start = ClientRegistration::<Default>::start(&mut client_rng, TEMP_PASSWORD)
        .expect("ClientRegistration::start failed");

    // Recup du message et conversion en bytes puis base64 pour envoi au serveur
    let start_message_bytes = start.message.serialize();
    let start_message_b64 = base64::engine::general_purpose::STANDARD.encode(start_message_bytes);

    // Recup du state (pour register_finish)
    let start_state = start.state;

    // Curl du client sur le serv :
    // TODO : Séparer dans un fichier qui s'occupe du networking et remplacer par appel fonction :
    let url = "http://0.0.0.0:3000/register/start";
    let payload = RegistrationRequest {
        username: TEMP_USERNAME.to_string(),
        registration_request: start_message_b64.to_string(),
    };

    // TODO : Remplacer aussi par le fichier networking qui retourne juste la réponse ici
    let client = reqwest::blocking::Client::new();
    match client.post(url).json(&payload).send() {
        Ok(response) => {
            if response.status().is_success() {
                println!("Connected successfully!");
                println!("Serveur response: {}", response.text().unwrap());
            }
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }

}
