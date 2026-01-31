use opaque_ke::CipherSuite;
use opaque_ke::ServerSetup;
use rand::rngs::OsRng;


struct Default;

impl CipherSuite for Default {
    type OprfCs = opaque_ke::Ristretto255;
    type KeyExchange = opaque_ke::TripleDh<opaque_ke::Ristretto255, sha2::Sha512>;
    type Ksf = opaque_ke::ksf::Identity;
}

fn make_server_setup() -> ServerSetup<Default> {
    let mut rng = OsRng;
    ServerSetup::<Default>::new(&mut rng) // CE SETUP DOIT ETRE STOCKE (PERSISTENT) ET UTILISÉ POUR TOUS LES CLIENTS
                                          // POUR L'INSTANT, SEULEMENT GARDé EN MEMOIRE
}