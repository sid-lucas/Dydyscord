use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct User {
    pub id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserPayload {
    pub username: String,
    pub opaque_record: String,
}
