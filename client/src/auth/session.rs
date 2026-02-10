use crate::auth::opaque::LoginResult;
use crate::error::AppError;
use crate::mls::provider::{self, MyProvider};
use crate::storage::error::StorageError;
use uuid::Uuid;

pub enum AppState {
    LoggedOut,
    LoggedIn(Session),
}

pub struct Session {
    pub user_id: Uuid,
    pub device_id: Option<String>,
    pub export_key: Vec<u8>,
    pub session_key: Vec<u8>,
    pub db_key: Option<Vec<u8>>,
    pub provider: Option<MyProvider>,
}

impl Session {
    pub fn new(login: LoginResult) -> Self {
        Session {
            user_id: login.id,
            export_key: login.export_key,
            session_key: login.session_key,
            device_id: None,
            db_key: None,
            provider: None,
        }
    }

    pub fn set_provider(&mut self, db_key: &[u8; 32], user_id: &str) -> Result<(), AppError> {
        self.provider = Some(provider::prepare_provider(db_key, user_id)?);
        Ok(())
    }
}
