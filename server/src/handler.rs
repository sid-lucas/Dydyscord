pub mod auth;
pub mod jwt;
pub mod mls;
pub mod root;

use hmac::Mac;
use secrecy::{ExposeSecret, SecretSlice};

fn login_lookup(pepper: &SecretSlice<u8>, username: &str) -> Vec<u8> {
    let normalized = username.trim().to_lowercase();

    let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(pepper.expose_secret())
        .expect("HMAC can take key of any size");

    mac.update(normalized.as_bytes());

    mac.finalize().into_bytes().to_vec()
}
