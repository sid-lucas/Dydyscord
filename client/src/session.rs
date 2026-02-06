use crate::error::ClientError;
use crate::mls::storage;
use crate::opaque::auth::LoginResult;

use uuid::Uuid;

#[derive(Debug)]
pub struct Session {
    pub user_id: Uuid,
    pub device_id: String,
    pub export_key: Vec<u8>,
    pub session_key: Vec<u8>,
    pub db_key: Vec<u8>,
}

impl Session {
    pub fn new(login: LoginResult) -> Self {
        Session {
            user_id: login.id,
            device_id: String::from("temp"), // TODO
            export_key: login.export_key,
            session_key: login.session_key,
            db_key: Vec::new(), // TODO
        }
    }
}

pub fn device_exists(user_id: &str, device_id: &str) -> bool {
    let has_key = storage::db_key_exists(user_id, device_id);
    let has_db = storage::db_exists();

    has_key && has_db
}
