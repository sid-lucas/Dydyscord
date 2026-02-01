use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct RegisterStartRequest {
    pub username: String,
    pub start_request: String, // base64
}

#[derive(Deserialize)]
pub struct RegisterStartResponse {
    pub start_response: String, // base64
}

#[derive(Serialize)]
pub struct RegisterFinishRequest {
    pub username: String,
    pub finish_request: String, // base64
}