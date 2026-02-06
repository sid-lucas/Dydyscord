use crate::error::ClientError;
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
    pub fn new(login: LoginResult) -> Result<Self, ClientError> {
        Ok(Session {
            user_id: login.id,
            device_id: String::from("temp"), // TODO
            export_key: login.export_key,
            session_key: login.session_key,
            db_key: Vec::new(), // TODO
        })
    }
}
