use crate::config::constant;
use crate::config::server::ServerState;
use axum::{
    extract::Request, extract::State, http::StatusCode, middleware::Next, response::Response,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use secrecy::{ExposeSecret, SecretSlice};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum TokenType {
    Auth,
    Session,
}

impl TokenType {
    pub fn ttl(&self) -> i64 {
        match self {
            TokenType::Auth => constant::JWT_AUTH_TTL,
            TokenType::Session => constant::JWT_SESSION_TTL,
        }
    }

    pub fn header(&self) -> &'static str {
        match self {
            TokenType::Auth => constant::JWT_AUTH_HEADER,
            TokenType::Session => constant::JWT_SESSION_HEADER,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    sub: String,    // Optional. Subject, whom token refers to (user_id or device_id)
    typ: TokenType, // Custom field created by me, type : Auth, Session, Access
    //prm: String, // Custom field created by me, Permissions/prm (role)
    aud: String, // Optional. Audience (ex: payments-service)
    exp: usize, // Required. (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: usize, // Optional. Issued at (as UTC timestamp)
    jti: Uuid, // Optional. JWT ID, can be UUIDv4 (used when user logout and send request to server to invalidate this specific JWT)
}

impl Claims {
    pub fn new(sub: &str, typ: TokenType) -> Self {
        let now = Utc::now().timestamp();
        let ttl = typ.ttl();

        Claims {
            sub: sub.to_string(),
            typ,
            aud: constant::JWT_AUDIENCE.to_string(),
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

fn create_jwt(
    sub: &str,
    typ: TokenType,
    key: &SecretSlice<u8>,
) -> Result<String, jsonwebtoken::errors::Error> {
    // Token header
    let mut header = Header::new(Algorithm::HS256);
    header.typ = Some("JWT".to_string());

    // Token payload
    let claims = Claims::new(sub, typ);

    // Return the token in JWT format (header + payload + signature)
    jsonwebtoken::encode::<Claims>(
        &header,
        &claims,
        &EncodingKey::from_secret(key.expose_secret()),
    )
}

pub fn create_cookie(
    sub: &str,
    typ: TokenType,
    key: &SecretSlice<u8>,
) -> Result<Cookie<'static>, jsonwebtoken::errors::Error> {
    let header = typ.header();
    let jwt = create_jwt(sub, typ, key)?;

    Ok(Cookie::build((header, jwt))
        .http_only(false) // TODO change
        .secure(false) // TODO Change: true forbids sending over HTTP. -> false for local testing for now.
        .same_site(SameSite::Strict)
        .path("/")
        .build())
}

pub async fn verify_jwt_with_type(
    mut req: Request,
    next: Next,
    typ: TokenType,
    key: &[u8],
) -> Result<Response, StatusCode> {
    // Retrieve the cookie
    let cookie = req
        .headers()
        .get("Cookie")
        .and_then(|v: &axum::http::HeaderValue| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Retrieve the JWT from the cookie
    let token = cookie
        .split(';')
        .map(|cookie: &str| cookie.trim())
        .find_map(|cookie: &str| cookie.strip_prefix(typ.header()))
        .map(|token: &str| token.trim_start_matches('=').trim())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_audience(&[constant::JWT_AUDIENCE]);

    // Decode token + signature OK + claims checks (according to Validation)
    let decoded =
        jsonwebtoken::decode::<Claims>(token, &DecodingKey::from_secret(key), &validation);

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

pub async fn verify_jwt_auth(
    State(state): State<ServerState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    verify_jwt_with_type(req, next, TokenType::Auth, state.jwt_key().expose_secret()).await
}

pub async fn verify_jwt_session(
    State(state): State<ServerState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    verify_jwt_with_type(
        req,
        next,
        TokenType::Session,
        state.jwt_key().expose_secret(),
    )
    .await
}
