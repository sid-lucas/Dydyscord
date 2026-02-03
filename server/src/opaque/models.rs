use serde::{Deserialize, Serialize};

// Register

#[derive(Deserialize)]
pub struct RegisterStartRequest {
    pub username: String,
    pub start_register_request: String, // base64
}

#[derive(Serialize)]
pub struct RegisterStartResponse {
    pub start_register_response: String, // base64
}

#[derive(Deserialize)]
pub struct RegisterFinishRequest {
    pub username: String,
    pub finish_register_request: String, // base64
}

// Login

#[derive(Deserialize)]
pub struct LoginStartRequest {
    pub username: String,
    pub start_login_request: String, // base64
}

#[derive(Serialize)]
pub struct LoginStartResponse {
    pub start_login_response: String, // base64
    pub nonce: String,                // clé-valeur pour retrouver le server_login_state
}

#[derive(Deserialize)]
pub struct LoginFinishRequest {
    pub finish_login_request: String, // base64
    pub nonce: String,                // clé-valeur pour retrouver le server_login_state
}
