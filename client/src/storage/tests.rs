use super::crypto;
use super::error::StorageError;
use crate::error::AppError;
use secrecy::{ExposeSecret, SecretSlice};

// Helper
fn secret(bytes: &[u8]) -> SecretSlice<u8> {
    bytes.to_vec().into()
}

#[test]
fn crypto_unwrapped_equals_db_key() {
    // Valid export_key
    let export_key = secret(&[42u8; 64]);

    // Wrap db_key
    let (db_key, wrapped) =
        crypto::generate_wrapped_db_key(&export_key).expect("generate wrapped key failed");

    // Unwrap db_key
    let unwrapped = crypto::unwrap_db_key(&export_key, &wrapped).expect("unwrap failed");

    // Unwrapped_db_key and db_key should be equal
    assert_eq!(unwrapped.expose_secret(), db_key.expose_secret());
}

#[test]
fn crypto_export_key_too_short_to_wrap() {
    // Invalid export_key (too short <32 bytes)
    let export_key = secret(&[1u8; 31]);

    // Could not wrap with export_key
    let err = crypto::generate_wrapped_db_key(&export_key).unwrap_err();

    assert!(matches!(
        err,
        AppError::Storage(StorageError::ExportKeyLength)
    ));
}

#[test]
fn crypto_export_key_too_short_to_unwrap() {
    // Valid wrap
    let wrapped = vec![42u8; 32];

    // Invalid export_key (too short <32 bytes)
    let export_key = secret(&[1u8; 31]);

    // Could not unwrap with export_key
    let err = crypto::unwrap_db_key(&export_key, &wrapped).unwrap_err();

    assert!(matches!(
        err,
        AppError::Storage(StorageError::ExportKeyLength)
    ));
}

#[test]
fn crypto_envelope_too_short() {
    // Invalid wrap (too short <13 bytes)
    let wrapped = vec![1u8; 12];

    // Valid export_key
    let export_key = secret(&[42u8; 64]);

    // Could not unwrap
    let err = crypto::unwrap_db_key(&export_key, &wrapped).unwrap_err();

    assert!(matches!(
        err,
        AppError::Storage(StorageError::EnvelopeLength)
    ));
}

#[test]
fn crypto_envelope_bad_version() {
    // Invalid wrap (bad version, should be 1)
    let wrapped = vec![2u8; 13]; // version = 2

    // Valid export_key
    let export_key = secret(&[42u8; 64]);

    // Could not unwrap
    let err = crypto::unwrap_db_key(&export_key, &wrapped).unwrap_err();

    assert!(matches!(
        err,
        AppError::Storage(StorageError::EnvelopeVersion)
    ));
}
