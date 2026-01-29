use ed25519_dalek as ed25519;
use ml_dsa as mldsa;
use mldsa::{MlDsa65, KeyGen};
use rand_core::OsRng;

struct Identity {
    identity_ed25519: ed25519::SigningKey,
    identity_mldsa: mldsa::KeyPair<MlDsa65>,
}

struct IdentityPubBundle {
    identity_ed25519: ed25519::VerifyingKey,
    identity_mldsa: mldsa::VerifyingKey<MlDsa65>,
}

fn generate_identity() -> (Identity, IdentityPubBundle) {
    let mut csprng = OsRng;

    // Génération de la paire asymétrique standard (ed25519)
    let identity_ed25519_sk = ed25519::SigningKey::generate(&mut csprng);
    let identity_ed25519_pk = identity_ed25519_sk.verifying_key();

    // Génération de la paire asymétrique PQ (ml-dsa)
    let identity_mldsa_kp = MlDsa65::key_gen(&mut csprng);
    let identity_mldsa_vk = identity_mldsa_kp.verifying_key().clone();

    (
        Identity {
            identity_ed25519: identity_ed25519_sk, // Paire ED25519
            identity_mldsa: identity_mldsa_kp,     // Paire ML-DSA
        },
        IdentityPubBundle {
            identity_ed25519: identity_ed25519_pk, // CléPub ED25519
            identity_mldsa: identity_mldsa_vk,     // CléPub ML-DSA
        },
    )
}
