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