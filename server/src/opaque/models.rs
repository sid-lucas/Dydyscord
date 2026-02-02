use serde::{Deserialize, Serialize};

// Register

#[derive(Deserialize)]
pub struct RegisterStartRequest {
    pub username: String,
    pub register_request: String, // base64
}

#[derive(Serialize)]
pub struct RegisterStartResponse {
    pub register_response: String, // base64
}

#[derive(Deserialize)]
pub struct RegisterFinishRequest {
    pub username: String,
    pub finish_request: String, // base64
}

// Login

#[derive(Deserialize)]
pub struct LoginStartRequest {
    pub username: String,
    pub start_request: String, // base64
}

#[derive(Serialize)]
pub struct LoginStartResponse {
    pub start_response: String, // base64
}

#[derive(Deserialize)]
pub struct LoginFinishRequest {
    pub username: String,
    pub finish_request: String, // base64
}