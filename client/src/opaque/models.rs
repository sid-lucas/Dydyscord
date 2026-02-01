use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct RegisterStartRequest<'a> {
    pub username: &'a str,
    pub start_request: String, // base64
}

#[derive(Deserialize)]
pub struct RegisterStartResponse {
    pub start_response: String, // base64
}

#[derive(Serialize)]
pub struct RegisterFinishRequest<'a> {
    pub username: &'a str,
    pub finish_request: String, // base64
}
