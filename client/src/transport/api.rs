use std::time::Duration;

use common::{
    CreateDeviceResponse, DeviceKeyPackage, KeyPackagesUploadRequest, OpaqueLoginFinishRequest,
    OpaqueLoginStartRequest, OpaqueLoginStartResponse, OpaqueRegisterFinishRequest,
    OpaqueRegisterStartRequest, OpaqueRegisterStartResponse, UserKeyPackageRequest,
    WelcomeFetchResponse, WelcomeStoreRequest,
};
use once_cell::sync::Lazy;
use reqwest::Url;
use reqwest::cookie::{CookieStore, Jar};
use reqwest::{Method, StatusCode, blocking::Client};
use serde::Serialize;
use std::sync::Arc;

use crate::config::constant;
use crate::transport::error::TransportError;

static COOKIE_JAR: Lazy<Arc<Jar>> = Lazy::new(|| Arc::new(Jar::default()));

static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .cookie_provider(COOKIE_JAR.clone())
        .timeout(Duration::from_secs(constant::HTTP_TIMEOUT_SECS))
        .build()
        .unwrap()
});

// Return the header "Cookie: ..." for the server domain
pub fn session_cookie_header() -> Option<String> {
    let url = Url::parse(constant::SERVER_URL).ok()?;
    let hv = COOKIE_JAR.cookies(&url)?;
    hv.to_str().ok().map(|s| s.to_string())
}

// Build the full endpoint URL
fn url(path: &str) -> String {
    format!("{}{path}", constant::SERVER_URL)
}

// Send an HTTP request with an optional JSON payload
fn send_request<Req: Serialize>(
    method: Method,
    path: &str,
    payload: Option<&Req>,
) -> Result<reqwest::blocking::Response, TransportError> {
    let mut req = CLIENT.request(method, url(path));
    if let Some(p) = payload {
        req = req.json(p);
    }
    req.send().map_err(|_| TransportError::Network)
}

// Map status codes to transport errors for this endpoint (so it can be different depending on the context)
fn map_status(
    status: StatusCode,
    expected: StatusCode,
    overrides: &[(StatusCode, TransportError)],
) -> Result<(), TransportError> {
    if status == expected {
        return Ok(());
    }
    for (code, err) in overrides {
        if status == *code {
            return Err(err.clone());
        }
    }
    Err(default_status_error(status))
}

fn default_status_error(status: StatusCode) -> TransportError {
    match status {
        StatusCode::BAD_REQUEST => TransportError::BadRequest,
        StatusCode::UNAUTHORIZED => TransportError::Unauthorized,
        StatusCode::CONFLICT => TransportError::Conflict,
        s if s.is_server_error() => TransportError::Internal,
        _ => TransportError::UnexpectedStatus,
    }
}

// Start OPAQUE registration and return the server response
pub fn opaque_register(
    payload: OpaqueRegisterStartRequest,
) -> Result<OpaqueRegisterStartResponse, TransportError> {
    let resp = send_request(Method::POST, "/register/start", Some(&payload))?;
    map_status(
        resp.status(),
        StatusCode::CREATED,
        &[(StatusCode::CONFLICT, TransportError::UsernameTaken)],
    )?;
    resp.json().map_err(|_| TransportError::InvalidResponse)
}

// Finish OPAQUE registration
pub fn opaque_register_finish(payload: OpaqueRegisterFinishRequest) -> Result<(), TransportError> {
    let resp = send_request(Method::POST, "/register/finish", Some(&payload))?;
    map_status(resp.status(), StatusCode::OK, &[])?;
    Ok(())
}

// Start OPAQUE login and return the server response
pub fn opaque_login(
    payload: OpaqueLoginStartRequest,
) -> Result<OpaqueLoginStartResponse, TransportError> {
    let resp = send_request(Method::POST, "/login/start", Some(&payload))?;
    map_status(
        resp.status(),
        StatusCode::OK,
        &[(StatusCode::NOT_FOUND, TransportError::LoginFailed)],
    )?;
    resp.json().map_err(|_| TransportError::InvalidResponse)
}

// Finish OPAQUE login (finalize authentication)
pub fn opaque_login_finish(payload: OpaqueLoginFinishRequest) -> Result<(), TransportError> {
    let resp = send_request(Method::POST, "/login/finish", Some(&payload))?;
    map_status(resp.status(), StatusCode::OK, &[])?;
    Ok(())
}

// Create a new device and return the API response payload
pub fn create_device() -> Result<CreateDeviceResponse, TransportError> {
    let resp = send_request::<()>(Method::POST, "/device", None)?;
    map_status(resp.status(), StatusCode::CREATED, &[])?;
    resp.json().map_err(|_| TransportError::InvalidResponse)
}

// Validate the existing device on the server
pub fn get_device() -> Result<(), TransportError> {
    let resp = send_request::<()>(Method::GET, "/device", None)?;
    map_status(resp.status(), StatusCode::OK, &[])?;
    Ok(())
}

// Upload key packages for this device
pub fn send_key_packages(
    device_id: &str,
    payload: KeyPackagesUploadRequest,
) -> Result<(), TransportError> {
    let resp = send_request(
        Method::POST,
        &format!("/device/{device_id}/keypackages"),
        Some(&payload),
    )?;
    map_status(resp.status(), StatusCode::OK, &[])?;
    Ok(())
}

// Fetch key packages for a user to create a group
pub fn create_group(
    payload: UserKeyPackageRequest,
) -> Result<Vec<DeviceKeyPackage>, TransportError> {
    let resp = send_request(Method::POST, "/user/keypackage", Some(&payload))?;
    map_status(resp.status(), StatusCode::OK, &[])?;
    resp.json().map_err(|_| TransportError::InvalidResponse)
}

// Send an MLS welcome message for delivery
pub fn send_welcome(payload: WelcomeStoreRequest) -> Result<(), TransportError> {
    let resp = send_request(Method::POST, "/welcome", Some(&payload))?;
    map_status(resp.status(), StatusCode::OK, &[])?;
    Ok(())
}

// Fetch pending MLS welcome messages
pub fn fetch_welcome() -> Result<Vec<WelcomeFetchResponse>, TransportError> {
    let resp = send_request::<()>(Method::GET, "/welcome", None)?;
    map_status(resp.status(), StatusCode::OK, &[])?;
    resp.json().map_err(|_| TransportError::InvalidResponse)
}

// Call a session-protected endpoint to test the session
pub fn test_session() -> Result<(), TransportError> {
    let resp = send_request::<()>(Method::GET, "/test/session", None)?;
    map_status(resp.status(), StatusCode::OK, &[])?;
    Ok(())
}
