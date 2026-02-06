use uuid::Uuid;

use crate::error::ClientError;
use crate::mls::{self, MyProvider, storage};
use crate::opaque::auth::LoginResult;

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

    pub fn set_provider(&mut self, db_key: &[u8; 32]) {
        // TODO : Voir si faut pas faire de gestion d'erreur au lieu du unwrap.
        self.provider = Some(mls::prepare_provider(db_key).unwrap());
    }
}

pub fn device_exists(user_id: &str) -> bool {
    let has_key = storage::db_key_exists(user_id);
    let has_db = storage::db_exists();

    has_key && has_db
}
