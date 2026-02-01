use opaque_ke::CipherSuite;
use opaque_ke::ServerSetup;
use rand::rngs::OsRng;

pub mod models;

pub struct OpaqueCiphersuite;

impl CipherSuite for OpaqueCiphersuite {
    type OprfCs = opaque_ke::Ristretto255;
    type KeyExchange = opaque_ke::TripleDh<opaque_ke::Ristretto255, sha2::Sha512>;
    type Ksf = opaque_ke::ksf::Identity;
}

pub fn make_server_setup() -> ServerSetup<OpaqueCiphersuite> {
    let mut rng = OsRng;
    ServerSetup::<OpaqueCiphersuite>::new(&mut rng) // appelé une seule fois au démarrage du serveur
}
