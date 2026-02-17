use secrecy::SecretSlice;

use crate::auth::error::AuthError;
use crate::auth::opaque::LoginResult;
use crate::error::AppError;
use crate::mls::provider::{self, MyProvider};

pub struct Session {
    user_id: String,
    device_id: Option<String>,
    export_key: SecretSlice<u8>,
    session_key: SecretSlice<u8>,
    db_key: Option<SecretSlice<u8>>,
    provider: Option<MyProvider>,
}

impl Session {
    pub fn new(login: LoginResult) -> Self {
        Session {
            user_id: login.id.to_string(),
            device_id: None,
            export_key: login.export_key.into(),
            session_key: login.session_key.into(),
            db_key: None,
            provider: None,
        }
    }

    // Setter
    pub fn set_device_id(&mut self, id: &str) {
        self.device_id = Some(id.to_string());
    }

    pub fn set_db_key(&mut self, db_key: SecretSlice<u8>) {
        self.db_key = Some(db_key);
    }

    pub fn set_provider(&mut self) -> Result<(), AppError> {
        let Some(db_key) = self.db_key.as_ref() else {
            return Err(AuthError::SessionDbKeyUnset.into());
        };
        self.provider = Some(provider::prepare_provider(db_key, self.user_id())?);
        Ok(())
    }

    // Getter
    pub fn user_id(&self) -> &str {
        self.user_id.as_str()
    }

    pub fn device_id(&self) -> Option<&str> {
        self.device_id.as_deref()
    }

    pub fn export_key(&self) -> &SecretSlice<u8> {
        &self.export_key
    }

    pub fn session_key(&self) -> &SecretSlice<u8> {
        &self.session_key
    }

    pub fn db_key(&self) -> Option<&SecretSlice<u8>> {
        self.db_key.as_ref()
    }

    pub fn provider(&self) -> Option<&MyProvider> {
        self.provider.as_ref()
    }
}
