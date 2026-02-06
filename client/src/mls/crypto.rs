use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use once_cell::sync::OnceCell;

// OnceCell car permet de définir une valeur global initialisée unde fois et accessible partout en lecture
// static mut : ne serait pas safe / risque de race condition
// var globale avec mutex : lourd et inutile si valeur change pas
// stocker dans EncryptedCodec : impossible car Codec impose fonctions statiques
static TEMP_KEY: OnceCell<[u8; 32]> = OnceCell::new();

fn init_codec_key(key: [u8; 32]) {
    let _ = TEMP_KEY.set(key);
}

pub fn wrap_db_key(export_key: &[u8; 32], db_key: &[u8; 32]) -> Result<Vec<u8>, ()> {
    encrypt_with_key(export_key, db_key)
}

pub fn unwrap_db_key(export_key: &[u8; 32], wrapped: &[u8]) -> Result<[u8; 32], ()> {
    let plain = decrypt_with_key(export_key, wrapped)?;
    if plain.len() != 32 {
        return Err(());
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&plain);
    Ok(out)
}

pub fn encrypt_with_key(key_bytes: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, ()> {
    // Création du nonce aléatoire
    let mut rng = OsRng;
    let nonce = Aes256Gcm::generate_nonce(&mut rng);

    // Chiffrement avec la TEMP_KEY
    let cipher = Aes256Gcm::new(key_bytes.into());
    let ciphertext = cipher.encrypt(&nonce, plaintext).map_err(|_| ())?;

    // envelope: version || nonce || ciphertext (tag inclus dans ct)
    let mut out = Vec::with_capacity(1 + 12 + ciphertext.len());
    out.push(1u8);
    out.extend_from_slice(nonce.as_slice());
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

pub fn decrypt_with_key(key_bytes: &[u8; 32], envelope: &[u8]) -> Result<Vec<u8>, ()> {
    // Vérification taille minimale
    if envelope.len() < 1 + 12 {
        return Err(());
    }

    // Vérification version
    if envelope[0] != 1u8 {
        return Err(());
    }

    // Extraction nonce et ciphertext
    let nonce_bytes = &envelope[1..13];
    let ciphertext = &envelope[13..];

    // Création cipher
    let cipher = Aes256Gcm::new(key_bytes.into());

    // Déchiffrement
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|_| ())?;

    Ok(plaintext)
}
