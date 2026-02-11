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
    use secrecy::ExposeSecret;

    // Génère une nouvelle db_key (32 bytes)
    let db_key: SecretSlice<u8> = rand::random::<[u8; 32]>().to_vec().into();

    // Récupère les 32 premiers bytes de l'export_key pour wrap la db_key
    let bytes = export_key.expose_secret();
    if bytes.len() < 32 {
        return Err(StorageError::ExportKeyLength.into());
    }
    let wrap_key: SecretSlice<u8> = bytes[..32].to_vec().into();

    // Wrap (chiffre) la db_key avec la wrap_key
    let wrapped = encrypt_db_key(&wrap_key, &db_key)?;

    Ok((db_key, wrapped))
}

pub fn unwrap_db_key(
    export_key: &SecretSlice<u8>,
    wrapped: &[u8],
) -> Result<SecretSlice<u8>, AppError> {
    // Récupère les 32 premiers bytes de l'export_key pour unwrap la db_key
    let bytes = export_key.expose_secret();
    if bytes.len() < 32 {
        return Err(StorageError::ExportKeyLength.into());
    }
    let wrap_key: SecretSlice<u8> = bytes[..32].to_vec().into();

    // Unwrap (déchiffre) la db_key avec la wrap_key
    let db_key = decrypt_db_key(&wrap_key, wrapped)?;

    // Vérifie que la db_key obtenue fait bien 32 bytes
    if db_key.expose_secret().len() != 32 {
        return Err(StorageError::DbKeyLength.into());
    }

    Ok(db_key)
}

pub fn encrypt_db_key(
    wrap_key: &SecretSlice<u8>,
    db_key: &SecretSlice<u8>,
) -> Result<Vec<u8>, AppError> {
    // Création du nonce aléatoire
    let mut rng = OsRng;
    let nonce = Aes256Gcm::generate_nonce(&mut rng);

    // Chiffrement la db_key avec la wrap_key
    let cipher = Aes256Gcm::new_from_slice(wrap_key.expose_secret())
        .map_err(|_| StorageError::EncryptWithWrapKey)?;
    let ciphertext = cipher
        .encrypt(&nonce, db_key.expose_secret())
        .map_err(|_| StorageError::EncryptWithWrapKey)?;

    // Création wrap (envelope): version || nonce || ciphertext (tag inclus dans ct)
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
    // Vérification taille minimale du wrap (envelope)
    if wrapped.len() < 1 + 12 {
        return Err(StorageError::EnvelopeLength.into());
    }

    // Vérification version du wrap
    if wrapped[0] != 1u8 {
        return Err(StorageError::EnvelopeVersion.into());
    }

    // Extraction nonce et ciphertext
    let nonce_bytes = &wrapped[1..13];
    let nonce = Nonce::from_slice(nonce_bytes);
    let ciphertext = &wrapped[13..];

    // Déchiffrement de la db_key avec la wrap_key
    let cipher = Aes256Gcm::new_from_slice(wrap_key.expose_secret())
        .map_err(|_| StorageError::DecryptWithWrapKey)?;
    let db_key_vec = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| StorageError::DecryptWithWrapKey)?;

    // Conversion en SecretSlice
    let db_key: SecretSlice<u8> = db_key_vec.into();

    Ok(db_key)
}
