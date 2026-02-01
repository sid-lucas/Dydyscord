use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RegisterStartRequest {
    pub username: String,
    pub registration_request: String, // base64
}

#[derive(Serialize)]
pub struct RegisterStartResponse {
    pub registration_response: String, // base64
}