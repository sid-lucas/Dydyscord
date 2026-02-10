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

pub fn open_sqlcipher(db_key: &[u8; 32], user_id: &str) -> Result<Connection, ClientError> {
    let db_path = ensure_db(user_id);
    let conn = Connection::open(db_path).map_err(|_| ClientError::Internal)?;

    let key_string = base64::engine::general_purpose::STANDARD.encode(db_key);
    conn.pragma_update(None, "key", &key_string)
        .map_err(|_| ClientError::Internal)?;

    Ok(conn)
}

pub fn ensure_db(user_id: &str) -> PathBuf {
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
    let extension = constants::DB_EXTENSION;
    let file_name = format!("{user_id}{extension}");
    db.push(file_name);

    // Créer le fichier db si non existant
    if !db.exists() {
        fs::File::create(&db).expect("Failed to create db file");
        fs::set_permissions(&db, fs::Permissions::from_mode(0o600)).unwrap();
    }

    db
}

pub fn file_path(user_id: &str, extension: &str) -> PathBuf {
    let home = env::var("HOME").expect("HOME not set");
    let path = PathBuf::from(home).join(constants::APP_FOLDER);
    let file_name = format!("{user_id}{extension}");
    path.join(file_name)
}

pub fn file_exists(user_id: &str, extension: &str) -> bool {
    let home = env::var("HOME").expect("HOME not set");
    let path_dir = PathBuf::from(home).join(constants::APP_FOLDER);
    let path_file = file_path(user_id, extension);
    path_dir.exists() && path_file.exists()
}

pub fn purge_storage(user_id: &str) {
    // TODO : Attention, pas de gestion d'erreur ici
    let db_path = file_path(user_id, constants::DB_EXTENSION);
    let key_path = file_path(user_id, constants::DB_KEY_EXTENSION);

    if db_path.exists() {
        let _ = fs::remove_file(db_path);
    }
    if key_path.exists() {
        let _ = fs::remove_file(key_path);
    }
}

pub fn get_db_key(user_id: &str, export_key: &[u8]) -> Result<[u8; 32], ClientError> {
    // Si <user_id>.key existe, essayer de decoder+déchiffrer
    if file_exists(user_id, constants::DB_KEY_EXTENSION) {
        let key_path = file_path(user_id, constants::DB_KEY_EXTENSION);
        let wrapped_b64 = fs::read_to_string(&key_path).map_err(|_| ClientError::Internal)?;
        let wrapped_b64 = wrapped_b64.trim();

        let wrapped = base64::engine::general_purpose::STANDARD
            .decode(wrapped_b64)
            .map_err(|_| ClientError::Internal)?;

        let db_key =
            crypto::unwrap_db_key(export_key, &wrapped).map_err(|_| ClientError::Internal)?;

        return Ok(db_key);
    }

    // Sinon créer + wrap + store dans le fichier <user_id>.key
    let db_key: [u8; 32] = rand::random();
    let wrapped = crypto::wrap_db_key(export_key, &db_key).map_err(|_| ClientError::Internal)?;
    let wrapped_b64 = base64::engine::general_purpose::STANDARD.encode(wrapped);

    let key_path = file_path(user_id, constants::DB_KEY_EXTENSION);
    if let Some(parent) = key_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|_| ClientError::Internal)?;
            fs::set_permissions(parent, fs::Permissions::from_mode(0o700))
                .map_err(|_| ClientError::Internal)?;
        }
    }
    fs::write(&key_path, wrapped_b64).map_err(|_| ClientError::Internal)?;
    fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600))
        .map_err(|_| ClientError::Internal)?;

    Ok(db_key)
}
