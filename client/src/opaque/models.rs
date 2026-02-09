use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Register

#[derive(Serialize)]
pub struct RegisterStartRequest<'a> {
    pub username: &'a str,
    pub start_register_request: String, // base64
}

#[derive(Deserialize)]
pub struct RegisterStartResponse {
    pub start_register_response: String, // base64
}

#[derive(Serialize)]
pub struct RegisterFinishRequest<'a> {
    pub username: &'a str,
    pub finish_register_request: String, // base64
}

// Login

#[derive(Serialize)]
pub struct LoginStartRequest<'a> {
    pub username: &'a str,
    pub start_login_request: String, // base64
}

#[derive(Deserialize)]
pub struct LoginStartResponse {
    pub start_login_response: String, // base64
    pub user_id: Uuid, // aussi utilisé comme clé-valeur pour retrouver le server_login_state
}

#[derive(Serialize)]
pub struct LoginFinishRequest {
    pub finish_login_request: String, // base64
    pub user_id: Uuid,                // clé-valeur pour retrouver le server_login_state
}

// Create new device

#[derive(Serialize)]
pub struct NewDeviceRequest<'a> {
    pub username: &'a str,
}
