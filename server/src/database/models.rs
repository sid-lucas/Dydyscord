use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize)]
pub struct User {
    pub id: Uuid,
    pub login_lookup: Vec<u8>,
    pub opaque_record: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

#[derive(Debug, Deserialize)]
pub struct CreateUserPayload {
    pub username: String,
    pub opaque_record: String,
}
