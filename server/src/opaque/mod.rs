use DefaultCipherSuite as DCS;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use opaque_ke::CipherSuite;
use opaque_ke::ServerSetup;
use opaque_ke::argon2::Argon2;
use rand::rngs::OsRng;

pub mod models;

pub struct DefaultCipherSuite;

impl CipherSuite for DefaultCipherSuite {
    type OprfCs = opaque_ke::Ristretto255;
    type KeyExchange = opaque_ke::TripleDh<opaque_ke::Ristretto255, sha2::Sha512>;
    type Ksf = Argon2<'static>;
}

pub fn _make_server_setup_for_env_file() {
    let mut rng = OsRng;
    let setup = ServerSetup::<DCS>::new(&mut rng);
    let setup_b64 = STANDARD.encode(setup.serialize());

    println!("{}", setup_b64);
}
