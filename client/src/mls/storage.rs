use std::os::unix::fs::PermissionsExt;
use std::{env, fs, path::PathBuf};

use crate::error::ClientError;

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

const APP_FOLDER: &str = ".dydyscord";
const DB_FILE: &str = "device.db";

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

pub fn ensure_db() -> PathBuf {
    let home = env::var("HOME").expect("HOME not set");

    // Chemin jusqu'au dossier de l'app
    let mut db = PathBuf::from(home);
    db.push(APP_FOLDER);

    // Créer le dossier de l'app si non existant
    if !db.exists() {
        fs::create_dir_all(&db).expect("Failed to create dir");
        fs::set_permissions(&db, fs::Permissions::from_mode(0o700)).unwrap();
    }

    // Chemin jusqu'au fichier db sqlite
    db.push(DB_FILE);

    // Créer le fichier db si non existant
    if !db.exists() {
        fs::File::create(&db).expect("Failed to create db file");
        fs::set_permissions(&db, fs::Permissions::from_mode(0o600)).unwrap();
    }

    db
}

pub fn db_exists() -> bool {
    let home = env::var("HOME").expect("HOME not set");
    let dir = PathBuf::from(home).join(APP_FOLDER);
    let db = dir.join(DB_FILE);
    dir.exists() && db.exists()
}

pub fn db_key_exists(user_id: &str, device_id: &str) -> bool {
    let account = format!("{user_id}:{device_id}");
    keyring::Entry::new("dydyscord", &account)
        .and_then(|e| e.get_password())
        .is_ok()
}
