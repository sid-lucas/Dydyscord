use openmls::prelude::Ciphersuite;

pub const APP_NAME: &str = "dydyscord";

pub const DB_EXTENSION: &str = ".db";
pub const DB_KEY_EXTENSION: &str = ".key";

pub const OPENMLS_CIPHERSUITE: Ciphersuite =
    Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519;
