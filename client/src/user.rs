use openmls::prelude::{CredentialWithKey, KeyPackage};
use std::fmt;
use uuid::Uuid;

pub struct User {
    username: String,
    devices: Vec<Device>,
}

impl User {
    pub fn new(username: String) -> User {
        Self {
            username,
            devices: Vec::new(),
        }
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.username)
    }
}

pub struct Device {
    uuid: Uuid,
    credential_with_key: CredentialWithKey,
    key_packages: Vec<KeyPackage>,
}
