use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ===== OPAQUE =====

#[derive(Serialize, Deserialize, Debug)]
pub struct OpaqueRegisterStartRequest {
    pub username: String,
    pub request_b64: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpaqueRegisterStartResponse {
    pub response_b64: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpaqueRegisterFinishRequest {
    pub username: String,
    pub request_b64: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpaqueLoginStartRequest {
    pub username: String,
    pub request_b64: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpaqueLoginStartResponse {
    pub user_id: Uuid,
    pub response_b64: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpaqueLoginFinishRequest {
    pub user_id: Uuid,
    pub request_b64: String,
}

// ===== DEVICE / WELCOME =====

#[derive(Serialize, Deserialize, Debug)]
pub struct DeviceKeyPackage {
    pub device_id: Uuid,
    pub key_package: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WelcomeStoreRequest {
    pub device_ids: Vec<Uuid>,
    pub welcome_b64: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WelcomeFetchResponse {
    pub welcome_b64: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserKeyPackageRequest {
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyPackagesUploadRequest {
    pub key_packages: Vec<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateDeviceResponse {
    pub device_id: Uuid,
}
