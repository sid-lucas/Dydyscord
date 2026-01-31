use openmls::prelude::{CredentialWithKey, KeyPackage};
use uuid::Uuid;

struct User {
    username: String,
    devices: Vec<Device>
}

struct Device {
    uuid: Uuid,
    credential_with_key: CredentialWithKey,
    key_packages: Vec<KeyPackage>
}