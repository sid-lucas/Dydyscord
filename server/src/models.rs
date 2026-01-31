use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct User {
    pub id: i32,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserPayload {
    pub username: String,
}


#[derive(Deserialize)]
pub struct RegisterStartRequest {
    pub username: String,
    pub registration_request: String, // base64
}

#[derive(Serialize)]
pub struct RegisterStartResponse {
    pub registration_response: String, // base64
}