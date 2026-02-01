use serde::Serialize;

#[derive(Serialize)]
pub struct RegistrationRequest {
    pub username: String,
    pub registration_request: String,
}