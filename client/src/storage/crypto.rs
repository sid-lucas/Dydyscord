use crate::error::AppError;
use crate::storage::error::StorageError;
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use secrecy::{ExposeSecret, SecretSlice};

pub fn generate_wrapped_db_key(
    export_key: &SecretSlice<u8>,
) -> Result<(SecretSlice<u8>, Vec<u8>), AppError> {
    // Generate a new db_key (32 bytes)
    let db_key: SecretSlice<u8> = rand::random::<[u8; 32]>().to_vec().into();

    // Get the first 32 bytes of export_key to wrap db_key
    let bytes = export_key.expose_secret();
    if bytes.len() < 32 {
        return Err(StorageError::ExportKeyLength.into());
    }
    let wrap_key: SecretSlice<u8> = bytes[..32].to_vec().into();

    // Wrap (encrypt) db_key with wrap_key
    let wrapped = encrypt_db_key(&wrap_key, &db_key)?;

    Ok((db_key, wrapped))
}

pub fn unwrap_db_key(
    export_key: &SecretSlice<u8>,
    wrapped: &[u8],
) -> Result<SecretSlice<u8>, AppError> {
    // Get the first 32 bytes of export_key to unwrap db_key
    let bytes = export_key.expose_secret();
    if bytes.len() < 32 {
        return Err(StorageError::ExportKeyLength.into());
    }
    let wrap_key: SecretSlice<u8> = bytes[..32].to_vec().into();

    // Unwrap (decrypt) db_key with wrap_key
    let db_key = decrypt_db_key(&wrap_key, wrapped)?;

    // Check that the resulting db_key is exactly 32 bytes
    if db_key.expose_secret().len() != 32 {
        return Err(StorageError::DbKeyLength.into());
    }

    Ok(db_key)
}

pub fn encrypt_db_key(
    wrap_key: &SecretSlice<u8>,
    db_key: &SecretSlice<u8>,
) -> Result<Vec<u8>, AppError> {
    // Create a random nonce
    let mut rng = OsRng;
    let nonce = Aes256Gcm::generate_nonce(&mut rng);

    // Encrypt db_key with wrap_key
    let cipher = Aes256Gcm::new_from_slice(wrap_key.expose_secret())
        .map_err(|_| StorageError::Encrypt)?;
    let ciphertext = cipher
        .encrypt(&nonce, db_key.expose_secret())
        .map_err(|_| StorageError::Encrypt)?;

    // Create wrap (envelope): version || nonce || ciphertext (tag included in ct)
    let mut wrapped = Vec::with_capacity(1 + 12 + ciphertext.len());
    wrapped.push(1u8);
    wrapped.extend_from_slice(nonce.as_slice());
    wrapped.extend_from_slice(&ciphertext);

    Ok(wrapped)
}

pub fn decrypt_db_key(
    wrap_key: &SecretSlice<u8>,
    wrapped: &[u8],
) -> Result<SecretSlice<u8>, AppError> {
    // Check minimum wrap (envelope) size
    if wrapped.len() < 1 + 12 {
        return Err(StorageError::EnvelopeLength.into());
    }

    // Check wrap version
    if wrapped[0] != 1u8 {
        return Err(StorageError::EnvelopeVersion.into());
    }

    // Extract nonce and ciphertext
    let nonce_bytes = &wrapped[1..13];
    let nonce = Nonce::from_slice(nonce_bytes);
    let ciphertext = &wrapped[13..];

    // Decrypt db_key with wrap_key
    let cipher = Aes256Gcm::new_from_slice(wrap_key.expose_secret())
        .map_err(|_| StorageError::Decrypt)?;
    let db_key_vec = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| StorageError::Decrypt)?;

    // Convert to SecretSlice
    let db_key: SecretSlice<u8> = db_key_vec.into();

    Ok(db_key)
}
