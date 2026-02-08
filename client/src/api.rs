use crate::error::ClientError;
use crate::opaque::models::{
    LoginFinishRequest, LoginStartRequest, LoginStartResponse, RegisterFinishRequest,
    RegisterStartRequest, RegisterStartResponse,
};
use reqwest::StatusCode;

const SERVER_URL: &str = "http://localhost:3000";

pub fn opaque_register(
    payload: RegisterStartRequest<'_>,
) -> Result<RegisterStartResponse, ClientError> {
    let url = format!("{SERVER_URL}/register/start");
    let client = reqwest::blocking::Client::new();

    let response = client
        .post(&url)
        .json(&payload)
        .send()
        .map_err(|_| ClientError::Network)?;

    match response.status() {
        StatusCode::CREATED => response.json().map_err(|_| ClientError::InvalidResponse),
        StatusCode::CONFLICT => Err(ClientError::UsernameTaken),
        StatusCode::BAD_REQUEST => Err(ClientError::BadRequest),
        _ => Err(ClientError::Server),
    }
}

pub fn opaque_register_finish(payload: RegisterFinishRequest<'_>) -> Result<String, ClientError> {
    let url = format!("{SERVER_URL}/register/finish");
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&url)
        .json(&payload)
        .send()
        .map_err(|_| ClientError::Network)?;

    match response.status() {
        StatusCode::OK => Ok(response.text().map_err(|_| ClientError::InvalidResponse)?),
        StatusCode::BAD_REQUEST => Err(ClientError::BadRequest),
        _ => Err(ClientError::Server),
    }
}

pub fn opaque_login(payload: LoginStartRequest<'_>) -> Result<LoginStartResponse, ClientError> {
    let url = format!("{SERVER_URL}/login/start");
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&url)
        .json(&payload)
        .send()
        .map_err(|_| ClientError::Network)?;

    match response.status() {
        StatusCode::OK => response.json().map_err(|_| ClientError::InvalidResponse),
        StatusCode::NOT_FOUND => Err(ClientError::LoginFailed),
        StatusCode::BAD_REQUEST => Err(ClientError::BadRequest),
        _ => Err(ClientError::Server),
    }
}

pub fn opaque_login_finish(payload: LoginFinishRequest) -> Result<String, ClientError> {
    let url = format!("{SERVER_URL}/login/finish");
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&url)
        .json(&payload)
        .send()
        .map_err(|_| ClientError::Network)?;

    match response.status() {
        StatusCode::OK => Ok(response.text().map_err(|_| ClientError::InvalidResponse)?),
        StatusCode::BAD_REQUEST => Err(ClientError::BadRequest),
        StatusCode::UNAUTHORIZED => Err(ClientError::Unauthorized),
        _ => Err(ClientError::Server),
    }
}

pub fn new_device(payload: LoginFinishRequest) -> Result<String, ClientError> {
    let url = format!("{SERVER_URL}/login/finish");
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&url)
        .json(&payload)
        .send()
        .map_err(|_| ClientError::Network)?;

    match response.status() {
        StatusCode::OK => Ok(response.text().map_err(|_| ClientError::InvalidResponse)?),
        StatusCode::BAD_REQUEST => Err(ClientError::BadRequest),
        StatusCode::UNAUTHORIZED => Err(ClientError::Unauthorized),
        _ => Err(ClientError::Server),
    }
}
