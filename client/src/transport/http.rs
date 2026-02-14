use crate::auth::opaque::{
    LoginFinishRequest, LoginStartRequest, LoginStartResponse, RegisterFinishRequest,
    RegisterStartRequest, RegisterStartResponse,
};
use crate::transport::error::TransportError;
use once_cell::sync::Lazy;
use reqwest::StatusCode;
use reqwest::blocking::Client;

const SERVER_URL: &str = "http://localhost:3000";

static CLIENT: Lazy<Client> = Lazy::new(|| Client::builder().cookie_store(true).build().unwrap());

pub fn opaque_register(
    payload: RegisterStartRequest<'_>,
) -> Result<RegisterStartResponse, TransportError> {
    let url = format!("{SERVER_URL}/register/start");
    let response = CLIENT
        .post(&url)
        .json(&payload)
        .send()
        .map_err(|_| TransportError::Network)?;

    match response.status() {
        StatusCode::CREATED => response.json().map_err(|_| TransportError::InvalidResponse),
        StatusCode::CONFLICT => Err(TransportError::UsernameTaken),
        StatusCode::BAD_REQUEST => Err(TransportError::BadRequest),
        _ => Err(TransportError::Server),
    }
}

pub fn opaque_register_finish(
    payload: RegisterFinishRequest<'_>,
) -> Result<String, TransportError> {
    let url = format!("{SERVER_URL}/register/finish");
    let response = CLIENT
        .post(&url)
        .json(&payload)
        .send()
        .map_err(|_| TransportError::Network)?;

    match response.status() {
        StatusCode::OK => Ok(response
            .text()
            .map_err(|_| TransportError::InvalidResponse)?),
        StatusCode::BAD_REQUEST => Err(TransportError::BadRequest),
        _ => Err(TransportError::Server),
    }
}

pub fn opaque_login(payload: LoginStartRequest<'_>) -> Result<LoginStartResponse, TransportError> {
    let url = format!("{SERVER_URL}/login/start");
    let response = CLIENT
        .post(&url)
        .json(&payload)
        .send()
        .map_err(|_| TransportError::Network)?;

    match response.status() {
        StatusCode::OK => response.json().map_err(|_| TransportError::InvalidResponse),
        StatusCode::NOT_FOUND => Err(TransportError::LoginFailed),
        StatusCode::BAD_REQUEST => Err(TransportError::BadRequest),
        _ => Err(TransportError::Server),
    }
}

pub fn opaque_login_finish(payload: LoginFinishRequest) -> Result<(), TransportError> {
    let url = format!("{SERVER_URL}/login/finish");
    let response = CLIENT
        .post(&url)
        .json(&payload)
        .send()
        .map_err(|_| TransportError::Network)?;

    match response.status() {
        StatusCode::OK => Ok(()),
        StatusCode::BAD_REQUEST => Err(TransportError::BadRequest),
        StatusCode::UNAUTHORIZED => Err(TransportError::Unauthorized),
        _ => Err(TransportError::Server),
    }
}

pub fn create_device() -> Result<String, TransportError> {
    let url = format!("{SERVER_URL}/device");
    let response = CLIENT
        .post(&url)
        .send()
        .map_err(|_| TransportError::Network)?;

    match response.status() {
        StatusCode::CREATED => Ok(response
            .text()
            .map_err(|_| TransportError::InvalidResponse)?),
        StatusCode::BAD_REQUEST => Err(TransportError::BadRequest),
        StatusCode::UNAUTHORIZED => Err(TransportError::Unauthorized),
        _ => Err(TransportError::Server),
    }
}

pub fn get_device() -> Result<(), TransportError> {
    let url = format!("{SERVER_URL}/device");
    let response = CLIENT
        .get(&url)
        .send()
        .map_err(|_| TransportError::Network)?;

    match response.status() {
        StatusCode::OK => Ok(()),
        StatusCode::BAD_REQUEST => Err(TransportError::BadRequest),
        StatusCode::UNAUTHORIZED => Err(TransportError::Unauthorized),
        _ => Err(TransportError::Server),
    }
}
