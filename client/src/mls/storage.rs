use base64::Engine;
use openmls_sqlite_storage::Connection;
use std::os::unix::fs::PermissionsExt;
use std::{env, fs, path::PathBuf};

use crate::constants;
use crate::error::ClientError;
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

pub fn open_sqlcipher(db_key: &[u8; 32]) -> Result<Connection, ClientError> {
    let db_path = ensure_db();
    let conn = Connection::open(db_path).map_err(|_| ClientError::Internal)?;

    let key_string = base64::engine::general_purpose::STANDARD.encode(db_key);
    conn.pragma_update(None, "key", &key_string)
        .map_err(|_| ClientError::Internal)?;

    Ok(conn)
}

pub fn ensure_db() -> PathBuf {
    let home = env::var("HOME").expect("HOME not set");

    // Chemin jusqu'au dossier de l'app
    let mut db = PathBuf::from(home);
    db.push(constants::APP_FOLDER);

    // Créer le dossier de l'app si non existant
    if !db.exists() {
        fs::create_dir_all(&db).expect("Failed to create dir");
        fs::set_permissions(&db, fs::Permissions::from_mode(0o700)).unwrap();
    }

    // Chemin jusqu'au fichier db sqlite
    db.push(constants::DB_FILE);

    // Créer le fichier db si non existant
    if !db.exists() {
        fs::File::create(&db).expect("Failed to create db file");
        fs::set_permissions(&db, fs::Permissions::from_mode(0o600)).unwrap();
    }

    db
}

pub fn db_exists() -> bool {
    let home = env::var("HOME").expect("HOME not set");
    let dir = PathBuf::from(home).join(constants::APP_FOLDER);
    let db = dir.join(constants::DB_FILE);
    dir.exists() && db.exists()
}

#[derive(Debug)]
pub enum DbKeyStatus {
    Present,
    Missing,
    Unavailable(String),
}

pub fn db_key_status(user_id: &str) -> DbKeyStatus {
    let account = user_id.to_string();
    let entry = match keyring::Entry::new(constants::KEYRING_SERVICE_NAME, &account) {
        Ok(entry) => entry,
        Err(err) => return DbKeyStatus::Unavailable(err.to_string()),
    };

    match entry.get_password() {
        Ok(_) => DbKeyStatus::Present,
        Err(keyring::Error::NoEntry) => DbKeyStatus::Missing,
        Err(err) => DbKeyStatus::Unavailable(err.to_string()),
    }
}

pub fn db_key_exists(user_id: &str) -> bool {
    matches!(db_key_status(user_id), DbKeyStatus::Present)
}

pub fn purge_db() {
    // TODO : Attention, pas de gestion d'erreur ici
    let db_path = ensure_db();
    fs::remove_file(&db_path).expect("device.db not found.");
}

pub fn get_or_create_db_key(user_id: &str, export_key: &[u8]) -> Result<[u8; 32], ClientError> {
    let account = user_id.to_string();
    let entry = keyring::Entry::new(constants::KEYRING_SERVICE_NAME, &account)
        .map_err(|_| ClientError::Keyring)?;

    // Essayer de récupérer la db_key si existe dans la keychain
    match entry.get_password() {
        Ok(wrapped_b64) => {
            let wrapped = base64::engine::general_purpose::STANDARD
                .decode(wrapped_b64)
                .map_err(|_| ClientError::Internal)?;
            let db_key =
                crypto::unwrap_db_key(export_key, &wrapped).map_err(|_| ClientError::Internal)?;
            return Ok(db_key);
        }
        Err(keyring::Error::NoEntry) => {
            // Si n'existe pas, continue en dessous pour l'ajouter dans la keychain
        }
        Err(_) => return Err(ClientError::Keyring),
    }

    // Sinon créer + wrap + store dans la keychain
    let db_key: [u8; 32] = rand::random();
    let wrapped = crypto::wrap_db_key(export_key, &db_key).map_err(|_| ClientError::Internal)?;
    let wrapped_b64 = base64::engine::general_purpose::STANDARD.encode(wrapped);

    entry
        .set_password(&wrapped_b64)
        .map_err(|_| ClientError::Keyring)?;

    Ok(db_key)
}
