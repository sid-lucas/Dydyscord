use opaque_ke::ClientRegistration;
use rand::RngCore;
use rand::rngs::OsRng;
use opaque_ke::CipherSuite;

struct Default;

impl CipherSuite for Default { // EXEMPLE DE LA DOC, A MODIFIER ?
    type OprfCs = opaque_ke::Ristretto255;
    type KeyExchange = opaque_ke::TripleDh<opaque_ke::Ristretto255, sha2::Sha512>;
    type Ksf = opaque_ke::ksf::Identity;
}


fn main() {
    let mut client_rng = OsRng;
    let client_registration_start_result = 
        ClientRegistration::<Default>::start(&mut client_rng, b"password")?;
}