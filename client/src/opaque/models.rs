use serde::{Deserialize, Serialize};

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
    pub start_request: String, // base64
}

#[derive(Deserialize)]
pub struct LoginStartResponse {
    pub start_response: String, // base64
}

#[derive(Serialize)]
pub struct LoginFinishRequest<'a> {
    pub username: &'a str,
    pub finish_request: String, // base64
}