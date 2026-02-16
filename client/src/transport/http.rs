use common::{
    CreateDeviceResponse, DeviceKeyPackage, KeyPackagesUploadRequest, OpaqueLoginFinishRequest,
    OpaqueLoginStartRequest, OpaqueLoginStartResponse, OpaqueRegisterFinishRequest,
    OpaqueRegisterStartRequest, OpaqueRegisterStartResponse, UserKeyPackageRequest,
    WelcomeFetchResponse, WelcomeStoreRequest,
};
use once_cell::sync::Lazy;
use reqwest::blocking::Client;
use reqwest::StatusCode;

use crate::transport::error::TransportError;

const SERVER_URL: &str = "http://localhost:3000";

static CLIENT: Lazy<Client> = Lazy::new(|| Client::builder().cookie_store(true).build().unwrap());

pub fn opaque_register(
    payload: OpaqueRegisterStartRequest,
) -> Result<OpaqueRegisterStartResponse, TransportError> {
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

pub fn opaque_register_finish(payload: OpaqueRegisterFinishRequest) -> Result<(), TransportError> {
    let url = format!("{SERVER_URL}/register/finish");
    let response = CLIENT
        .post(&url)
        .json(&payload)
        .send()
        .map_err(|_| TransportError::Network)?;

    match response.status() {
        StatusCode::OK => Ok(()),
        StatusCode::BAD_REQUEST => Err(TransportError::BadRequest),
        _ => Err(TransportError::Server),
    }
}

pub fn opaque_login(
    payload: OpaqueLoginStartRequest,
) -> Result<OpaqueLoginStartResponse, TransportError> {
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

pub fn opaque_login_finish(payload: OpaqueLoginFinishRequest) -> Result<(), TransportError> {
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
        StatusCode::CREATED => {
            let body = response
                .json::<CreateDeviceResponse>()
                .map_err(|_| TransportError::InvalidResponse)?;
            Ok(body.device_id.to_string())
        }
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

pub fn send_key_packages(
    device_id: &str,
    key_packages: Vec<Vec<u8>>,
) -> Result<(), TransportError> {
    let url = format!("{SERVER_URL}/device/{device_id}/keypackages");
    let response = CLIENT
        .post(&url)
        .json(&KeyPackagesUploadRequest { key_packages })
        .send()
        .map_err(|_| TransportError::Network)?;

    match response.status() {
        StatusCode::OK => Ok(()),
        StatusCode::BAD_REQUEST => Err(TransportError::BadRequest),
        StatusCode::UNAUTHORIZED => Err(TransportError::Unauthorized),
        _ => Err(TransportError::Server),
    }
}

pub fn create_group(user: &str) -> Result<Vec<DeviceKeyPackage>, TransportError> {
    let url = format!("{SERVER_URL}/user/keypackage"); // TODO : change route name
    let response = CLIENT
        .post(&url)
        .json(&UserKeyPackageRequest {
            username: user.to_string(),
        })
        .send()
        .map_err(|_| TransportError::Network)?;

    match response.status() {
        StatusCode::OK => response.json().map_err(|_| TransportError::InvalidResponse),
        StatusCode::BAD_REQUEST => Err(TransportError::BadRequest),
        StatusCode::UNAUTHORIZED => Err(TransportError::Unauthorized),
        _ => Err(TransportError::Server),
    }
}

pub fn send_welcome(payload: WelcomeStoreRequest) -> Result<(), TransportError> {
    let url = format!("{SERVER_URL}/welcome");
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

pub fn fetch_welcome() -> Result<Vec<WelcomeFetchResponse>, TransportError> {
    let url = format!("{SERVER_URL}/welcome");
    let response = CLIENT
        .get(&url)
        .send()
        .map_err(|_| TransportError::Network)?;

    match response.status() {
        StatusCode::OK => response.json().map_err(|_| TransportError::InvalidResponse),
        StatusCode::BAD_REQUEST => Err(TransportError::BadRequest),
        StatusCode::UNAUTHORIZED => Err(TransportError::Unauthorized),
        _ => Err(TransportError::Server),
    }
}

pub fn test_session() -> Result<(), TransportError> {
    let url = format!("{SERVER_URL}/test/session");
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
