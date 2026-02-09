use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::constants;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum TokenType {
    Auth,
    Access,
    Refresh,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    sub: String,    // Optional. Subject, whom token refers to (user_id ou device_id?)
    typ: TokenType, // Custom field created by me, type : Auth, Refresh, Access
    //prm: String, // Custom field created by me, Permissions/prm (role)
    aud: String, // Optional. Audience (ex: payments-service)
    exp: usize, // Required. (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: usize, // Optional. Issued at (as UTC timestamp)
    jti: Uuid, // Optional. JWT ID, can be UUIDv4 (used when user logout and send request to server to invalidate this specific JWT)
}

impl Claims {
    pub fn new(sub: &str, typ: TokenType) -> Self {
        let now = Utc::now().timestamp();
        let ttl = constants::JWT_TTL;

        Claims {
            sub: sub.to_string(),
            typ,
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

    pub fn typ(&self) -> &TokenType {
        &self.typ
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

pub fn create_jwt(sub: &str, typ: TokenType) -> Result<String, jsonwebtoken::errors::Error> {
    // Header du token
    let mut header = Header::new(Algorithm::HS256);
    header.typ = Some("JWT".to_string());

    // Payload du token
    let claims = Claims::new(sub, typ);

    // Envoie retour du token (header + payload + signature)
    jsonwebtoken::encode::<Claims>(
        &header,
        &claims,
        &EncodingKey::from_secret(constants::JWT_SECRET_KEY.get().unwrap().as_ref()),
    )
}

pub async fn verify_jwt_with_type(
    mut req: Request,
    next: Next,
    typ: TokenType,
) -> Result<Response, StatusCode> {
    // Récupère le cookie
    let cookie = req
        .headers()
        .get("Cookie")
        .and_then(|v: &axum::http::HeaderValue| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Récupère le JWT du cookie
    let token = cookie
        .split(';')
        .map(|cookie: &str| cookie.trim())
        .find_map(|cookie: &str| cookie.strip_prefix(constants::AUTH_HEADER))
        .map(|token: &str| token.trim_start_matches('=').trim())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_audience(&[constants::JWT_AUDIENCE]);

    // Decode token + signature OK + checks de claims (selon Validation)
    let decoded = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(constants::JWT_SECRET_KEY.get().unwrap().as_ref()),
        &validation,
    );

    match decoded {
        Ok(data) => {
            if data.claims.typ == typ {
                // Add claims to request extensions if needed
                req.extensions_mut().insert(data.claims);

                // Proceed to the next middleware or handler
                Ok(next.run(req).await)
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

pub async fn verify_jwt_access(req: Request, next: Next) -> Result<Response, StatusCode> {
    verify_jwt_with_type(req, next, TokenType::Access).await
}

pub async fn verify_jwt_auth(req: Request, next: Next) -> Result<Response, StatusCode> {
    verify_jwt_with_type(req, next, TokenType::Auth).await
}
