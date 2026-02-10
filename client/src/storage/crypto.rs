use crate::error::AppError;
use crate::storage::error::StorageError;
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use hkdf::Hkdf;
use sha2::Sha256;

pub fn wrap_db_key(export_key: &[u8], db_key: &[u8; 32]) -> Result<Vec<u8>, AppError> {
    let wrap_key = derive_wrap_key(export_key)?;
    encrypt_with_key(&wrap_key, db_key)
}

pub fn unwrap_db_key(export_key: &[u8], wrapped: &[u8]) -> Result<[u8; 32], AppError> {
    let wrap_key = derive_wrap_key(export_key)?;
    let plain = decrypt_with_key(&wrap_key, wrapped)?;
    if plain.len() != 32 {
        return Err(StorageError("unwrapped key length invalid").into());
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&plain);
    Ok(out)
}

fn derive_wrap_key(export_key: &[u8]) -> Result<[u8; 32], AppError> {
    // TODO : Vérifier... dériver wrap_key avec export_key pcq export_key fait 64bytes et il faut une clé de 32bytes pour chiffrer db_key
    let hk = Hkdf::<Sha256>::new(None, export_key);
    let mut out = [0u8; 32];
    hk.expand(b"db-key-wrap", &mut out)
        .map_err(|_| StorageError("wrap key derive"))?;
    Ok(out)
}

pub fn encrypt_with_key(key_bytes: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, AppError> {
    // Création du nonce aléatoire
    let mut rng = OsRng;
    let nonce = Aes256Gcm::generate_nonce(&mut rng);

    // Chiffrement avec la TEMP_KEY
    let cipher = Aes256Gcm::new(key_bytes.into());
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|_| StorageError("wrap key encryption"))?;

    // envelope: version || nonce || ciphertext (tag inclus dans ct)
    let mut out = Vec::with_capacity(1 + 12 + ciphertext.len());
    out.push(1u8);
    out.extend_from_slice(nonce.as_slice());
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

pub fn decrypt_with_key(key_bytes: &[u8; 32], envelope: &[u8]) -> Result<Vec<u8>, AppError> {
    // Vérification taille minimale
    if envelope.len() < 1 + 12 {
        return Err(StorageError("envelope length").into());
    }

    // Vérification version
    if envelope[0] != 1u8 {
        return Err(StorageError("envelope version").into());
    }

    // Extraction nonce et ciphertext
    let nonce_bytes = &envelope[1..13];
    let ciphertext = &envelope[13..];

    // Création cipher
    let cipher = Aes256Gcm::new(key_bytes.into());

    // Déchiffrement
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| StorageError("wrap key decryption"))?;

    Ok(plaintext)
}
