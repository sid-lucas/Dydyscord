use std::os::unix::fs::PermissionsExt;
use std::{env, fs, path::PathBuf};

use base64::Engine;
use openmls_sqlite_storage::Connection;
use rusqlite::{OptionalExtension, params};
use secrecy::{ExposeSecret, SecretSlice};

use crate::config::constant;
use crate::error::AppError;
use crate::storage::{crypto, error::StorageError};
use crate::transport::http;

// TODO Maybe split into multiple files, to discuss

pub fn open_sqlcipher(db_key: &SecretSlice<u8>, user_id: &str) -> Result<Connection, StorageError> {
    let db_path = ensure_db(user_id, constant::DB_EXTENSION)?;
    let conn = Connection::open(db_path).map_err(|_| StorageError::DatabaseConnect)?;

    let key_string = base64::engine::general_purpose::STANDARD.encode(db_key.expose_secret());

    conn.pragma_update(None, "key", &key_string)
        .map_err(|_| StorageError::DatabaseRead)?;

    Ok(conn)
}

pub fn ensure_app_dir() -> Result<PathBuf, StorageError> {
    let home = env::var("HOME").expect("HOME not set"); // TODO : CONST

    // Path to the app folder
    let mut path_app_dir = PathBuf::from(home);
    let dir_name = format!(".{}", constant::APP_NAME);
    path_app_dir.push(dir_name);

    // Create the app folder if it does not exist
    if !path_app_dir.exists() {
        fs::create_dir_all(&path_app_dir).map_err(|_| StorageError::AppDirectoryCreate)?;
    }
    // Ensure restrictive permissions are applied even if the folder already exists
    fs::set_permissions(&path_app_dir, fs::Permissions::from_mode(0o700))
        .map_err(|_| StorageError::AppDirectoryPermissions)?;

    // Return the path to the app dir
    Ok(path_app_dir)
}

pub fn ensure_db(user_id: &str, extension: &str) -> Result<PathBuf, StorageError> {
    let mut path_db_file = ensure_app_dir()?;

    // Path to the sqlite file (.db or .key depending on extension)
    let file_name = format!("{user_id}{extension}");
    path_db_file.push(file_name);

    // Create the db file if it does not exist
    if !path_db_file.exists() {
        fs::File::create(&path_db_file).map_err(|_| StorageError::StorageFileCreate)?;
    }
    // Ensure restrictive permissions are applied even if the file already exists
    fs::set_permissions(&path_db_file, fs::Permissions::from_mode(0o600))
        .map_err(|_| StorageError::StorageFilePermissions)?;
    Ok(path_db_file)
}

pub fn file_path(user_id: &str, extension: &str) -> PathBuf {
    let home = env::var("HOME").expect("HOME not set"); // TODO : CONST
    let path = PathBuf::from(home).join(format!(".{}", constant::APP_NAME));
    let file_name = format!("{user_id}{extension}");
    path.join(file_name)
}

pub fn file_exists(user_id: &str, extension: &str) -> bool {
    let home = env::var("HOME").expect("HOME not set"); // TODO : CONST
    let path_dir = PathBuf::from(home).join(format!(".{}", constant::APP_NAME));
    let path_file = file_path(user_id, extension);
    path_dir.exists() && path_file.exists()
}

pub fn purge_storage(user_id: &str) {
    let db_path = file_path(user_id, constant::DB_EXTENSION);
    let key_path = file_path(user_id, constant::DB_KEY_EXTENSION);

    if db_path.exists() {
        let _ = fs::remove_file(db_path);
    }
    if key_path.exists() {
        let _ = fs::remove_file(key_path);
    }
}

pub fn get_db_key(
    user_id: &str,
    export_key: &SecretSlice<u8>,
) -> Result<SecretSlice<u8>, AppError> {
    // If <user_id>.key exists, try to decode+decrypt
    if file_exists(user_id, constant::DB_KEY_EXTENSION) {
        let key_path = file_path(user_id, constant::DB_KEY_EXTENSION);
        // TODO: Possible race condition between file_exists() and read_to_string(), but ignored
        let wrapped_b64 =
            fs::read_to_string(&key_path).map_err(|_| StorageError::StorageFileRead)?;
        let wrapped_b64 = wrapped_b64.trim();

        let wrapped = base64::engine::general_purpose::STANDARD
            .decode(wrapped_b64)
            .map_err(|_| StorageError::DbKeyDecode)?;

        let db_key = crypto::unwrap_db_key(export_key, &wrapped)?;

        return Ok(db_key);
    }

    // Otherwise generate db_key and wrap it
    let (db_key, wrapped) = crypto::generate_wrapped_db_key(export_key)?;
    let wrapped_b64 = base64::engine::general_purpose::STANDARD.encode(wrapped);

    // Ensure the .key file exists
    let key_path = ensure_db(user_id, constant::DB_KEY_EXTENSION)?;

    // Write the encrypted db_key into the file
    fs::write(&key_path, wrapped_b64).map_err(|_| StorageError::DbKeyStore)?;

    Ok(db_key)
}

fn ensure_app_state_table(conn: &Connection) -> Result<(), AppError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_state (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )
    .map_err(|_| StorageError::DatabaseSchema)?;
    Ok(())
}

pub fn store_device_id(
    db_key: &SecretSlice<u8>,
    user_id: &str,
    device_id: &str,
) -> Result<(), AppError> {
    let conn = open_sqlcipher(db_key, user_id)?;

    ensure_app_state_table(&conn)?;

    conn.execute(
        "INSERT OR REPLACE INTO app_state (key, value) VALUES ('device_id', ?1)",
        params![device_id],
    )
    .map_err(|_| StorageError::DatabaseQuery)?;

    Ok(())
}

pub fn read_device_id(db_key: &SecretSlice<u8>, user_id: &str) -> Result<Option<String>, AppError> {
    let conn = open_sqlcipher(db_key, user_id)?;

    ensure_app_state_table(&conn)?;

    let device_id: Option<String> = conn
        .query_row(
            "SELECT value FROM app_state WHERE key = 'device_id' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| StorageError::DatabaseQuery)?;

    Ok(device_id)
}

pub fn store_signature_pub_key(
    db_key: &SecretSlice<u8>,
    user_id: &str,
    signature_public_key_b64: &str,
) -> Result<(), AppError> {
    let conn = open_sqlcipher(db_key, user_id)?;
    ensure_app_state_table(&conn)?;

    conn.execute(
        "INSERT OR REPLACE INTO app_state (key, value) VALUES ('signature_public_key', ?1)",
        params![signature_public_key_b64],
    )
    .map_err(|_| StorageError::DatabaseQuery)?;

    Ok(())
}

pub fn read_signature_pub_key(db_key: &SecretSlice<u8>, user_id: &str) -> Result<String, AppError> {
    let conn = open_sqlcipher(db_key, user_id)?;
    ensure_app_state_table(&conn)?;

    let pub_key: Option<String> = conn
        .query_row(
            "SELECT value FROM app_state WHERE key = 'signature_public_key' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .map_err(|_| StorageError::DatabaseQuery)?;

    pub_key.ok_or(StorageError::DatabaseQuery.into())
}

// Check whether a db + db_key already exist (device exists)
// or whether one is missing (device does not exist and purge on inconsistency)
pub fn reconcile_device_storage(user_id: &str) -> bool {
    let has_db = file_exists(user_id, constant::DB_EXTENSION);
    let has_key = file_exists(user_id, constant::DB_KEY_EXTENSION);

    // If db is present but not db_key -> consider db corrupted/lost, so purge
    if !has_db || !has_key {
        purge_storage(user_id);
    }

    has_db && has_key
}

pub fn init_device_storage(
    user_id: &str,
    export_key: &SecretSlice<u8>,
) -> Result<(String, SecretSlice<u8>, bool), AppError> {
    // Check for db/key files and purge if there is an issue
    let has_storage = reconcile_device_storage(&user_id.to_string());

    // Retrieve/Create the db encryption key
    let db_key = get_db_key(&user_id.to_string(), export_key)?;

    // If files are OK, try to read and retrieve the stored device_id
    let device_id = if has_storage {
        match read_device_id(&db_key, user_id) {
            Ok(id) => id,
            Err(_) => None, // No device_id could be retrieved -> considered corrupted
        }
    } else {
        None
    };

    let mut is_new_device = false;

    let device_id = match device_id {
        Some(id) => {
            http::get_device()?; // TODO : Check if the device_id is still valid on the server, if not consider the device as new and re-register
            id
        }
        None => {
            // New device detected, initialize with server
            is_new_device = true;

            // Retrieve the new device_id and Session token
            let device_id = http::create_device()?;

            // Store device_id in local db
            store_device_id(&db_key, user_id, device_id.as_str())?;

            device_id
        }
    };

    Ok((device_id, db_key, is_new_device))
}
