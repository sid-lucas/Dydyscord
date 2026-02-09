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

    pub fn set_provider(&mut self, db_key: &[u8; 32]) -> Result<(), ClientError> {
        self.provider = Some(mls::prepare_provider(db_key)?);
        Ok(())
    }
}

// Check si existe deja une db + db_key (device existe)
// ou si l'un manque (device non existant et purge si incohérence)
pub fn reconcile_device_storage(user_id: &str) -> bool {
    let has_key = storage::db_key_exists(user_id);
    let has_db = storage::db_exists();

    // Si db présente mais pas la db_key -> considère la db comme corrompue/perdue donc purge
    if has_db && !has_key {
        let _ = storage::purge_db();
    }

    //TODO : remove, Print de debug
    println!("has_key : {}", has_key);
    println!("has_db : {}", has_db);

    has_db && has_key
}
