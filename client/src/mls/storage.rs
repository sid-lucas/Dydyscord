use std::os::unix::fs::PermissionsExt;
use std::{env, fs, path::PathBuf};

use crate::mls::crypto;

#[derive(thiserror::Error, Debug)]
pub enum CodecError {
    #[error("internal error")]
    InternalError,
    #[error("codec not initialized")]
    NotInitialized,
    #[error("serialize/deserialize error")]
    Serde,
    #[error("crypto error")]
    Crypto,
}

pub struct EncryptedCodec;

impl Default for EncryptedCodec {
    fn default() -> Self {
        Self
    }
}

// TODO, je crois qu'il faut adapter les map_err en utilisant l'enum CodecError
impl openmls_sqlite_storage::Codec for EncryptedCodec {
    type Error = CodecError;

    fn to_vec<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, Self::Error> {
        // Serialize en bytes avec ciborium
        let mut serialized = Vec::new();
        ciborium::ser::into_writer(value, &mut serialized).map_err(|_| CodecError::Serde)?;

        // Chiffre les bytes avec export_key de opaque
        let envelope =
            crypto::encrypt_aes_gcm(serialized.as_slice()).map_err(|_| CodecError::Crypto)?;

        Ok(envelope)
    }

    fn from_slice<T: serde::de::DeserializeOwned>(slice: &[u8]) -> Result<T, Self::Error> {
        // Déchiffre les bytes avec export_key de opaque
        let plaintext = crypto::decrypt_aes_gcm(slice).map_err(|_| CodecError::Crypto)?;

        // Deserialize avec ciborium
        let value =
            ciborium::de::from_reader(plaintext.as_slice()).map_err(|_| CodecError::Serde)?;

        Ok(value)
    }
}

pub struct CBORCodec;

impl Default for CBORCodec {
    fn default() -> Self {
        Self
    }
}

impl openmls_sqlite_storage::Codec for CBORCodec {
    type Error = CodecError;

    fn to_vec<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, Self::Error> {
        let mut out = Vec::new();
        ciborium::ser::into_writer(value, &mut out).map_err(|_| CodecError::Serde)?;
        Ok(out)
    }

    fn from_slice<T: serde::de::DeserializeOwned>(slice: &[u8]) -> Result<T, Self::Error> {
        ciborium::de::from_reader(slice).map_err(|_| CodecError::Serde)
    }
}

pub fn ensure_localdb_path() -> PathBuf {
    // chemin jusqu'au dossier
    let home = env::var("HOME").expect("HOME not set");

    let mut dir = PathBuf::from(home);
    dir.push(".dydyscord");

    // Créer le dossier de l'app si non existant
    if !dir.exists() {
        fs::create_dir_all(&dir).expect("Failed to create dir");
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
    }

    // chemin jusqu'au fichier db sqlite
    let mut db = dir.clone();
    db.push("state.db");
    db
}
