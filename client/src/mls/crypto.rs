use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use once_cell::sync::OnceCell;

// OnceCell car permet de définir une valeur global initialisée unde fois et accessible partout en lecture
// static mut : ne serait pas safe / risque de race condition
// var globale avec mutex : lourd et inutile si valeur change pas
// stocker dans EncryptedCodec : impossible car Codec impose fonctions statiques
static EXPORT_KEY: OnceCell<[u8; 32]> = OnceCell::new();

pub fn init_codec_key(key: [u8; 32]) {
    let _ = EXPORT_KEY.set(key);
}

pub fn encrypt_aes_gcm(plaintext: &[u8]) -> Result<Vec<u8>, ()> {
    // Conversion de l'export_key
    let k = EXPORT_KEY.get().ok_or(())?;
    let key: &Key<Aes256Gcm> = k.into();

    // Création du nonce aléatoire
    let mut rng = OsRng;
    let nonce = Aes256Gcm::generate_nonce(&mut rng);

    // Chiffrement avec la export_key
    let cipher = Aes256Gcm::new(key);
    let ciphertext = cipher.encrypt(&nonce, plaintext).map_err(|_| ())?;

    // envelope: version || nonce || ciphertext (tag inclus dans ct)
    let mut out = Vec::with_capacity(1 + 12 + ciphertext.len());
    out.push(1u8);
    out.extend_from_slice(nonce.as_slice());
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

pub fn decrypt_aes_gcm(envelope: &[u8]) -> Result<Vec<u8>, ()> {
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

    // Récupération de la clé
    let k = EXPORT_KEY.get().ok_or(())?;
    let key: &Key<Aes256Gcm> = k.into();

    // Création cipher
    let cipher = Aes256Gcm::new(key);

    // Déchiffrement
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|_| ())?;

    Ok(plaintext)
}
