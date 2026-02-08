use chrono::Utc;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::constants;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String, // Optional. Subject, whom token refers to (user_id ou device_id?)
    prm: String, // Custom field created by me, Permissions/prm
    aud: String, // Optional. Audience (ex: payments-service)
    exp: usize, // Required. (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: usize, // Optional. Issued at (as UTC timestamp)
    jti: Uuid, // Optional. JWT ID, can be UUIDv4 (used when user logout and send request to server to invalidate this specific JWT)
}

impl Claims {
    pub fn new(sub: &str, prm: &str) -> Self {
        let now = Utc::now().timestamp();
        let ttl = constants::JWT_TTL;

        Claims {
            sub: sub.to_string(),
            prm: prm.to_string(),
            aud: constants::JWT_AUDIENCE.to_string(),
            exp: (now + ttl) as usize,
            iat: now as usize,
            jti: Uuid::new_v4(),
        }
    }

    // Getters
    pub fn sub(&self) -> &str {
        &self.sub
    }

    pub fn prm(&self) -> &str {
        &self.prm
    }

    pub fn aud(&self) -> &str {
        &self.aud
    }

    pub fn exp(&self) -> usize {
        self.exp
    }

    pub fn iat(&self) -> usize {
        self.iat
    }

    pub fn jti(&self) -> &Uuid {
        &self.jti
    }
}

pub fn create_jwt(sub: &str, prm: &str) -> Result<String, jsonwebtoken::errors::Error> {
    // Header du token
    let mut header = Header::new(Algorithm::HS256);
    header.typ = Some("JWT".to_string());

    // Payload du token
    let claims = Claims::new(sub, prm);

    // Envoie retour du token (header + payload + signature)
    jsonwebtoken::encode(
        &header,
        &claims,
        &EncodingKey::from_secret(constants::JWT_SECRET_KEY.get().unwrap().as_ref()),
    )
}
