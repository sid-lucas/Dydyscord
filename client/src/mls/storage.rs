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
