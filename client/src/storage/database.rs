use base64::Engine;
use openmls_sqlite_storage::Connection;
use rusqlite::{OptionalExtension, params};
use secrecy::{ExposeSecret, SecretSlice};
use std::os::unix::fs::PermissionsExt;
use std::{env, fs, path::PathBuf};

use crate::config::constant;
use crate::error::AppError;
use crate::storage::{crypto, error::StorageError};
use crate::transport::http;

// TODO Peut etre découper en plusieurs fichiers, à discuter

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

    // Chemin jusqu'au dossier de l'app
    let mut path_app_dir = PathBuf::from(home);
    let dir_name = format!(".{}", constant::APP_NAME);
    path_app_dir.push(dir_name);

    // Créer le dossier de l'app si non existant
    if !path_app_dir.exists() {
        fs::create_dir_all(&path_app_dir).map_err(|_| StorageError::AppDirectoryCreate)?;
    }
    // S'assure d'appliquer permissions restrictives même si le dossier existe déjà
    fs::set_permissions(&path_app_dir, fs::Permissions::from_mode(0o700))
        .map_err(|_| StorageError::AppDirectoryPermissions)?;

    // Retourne le chemin jusqu'à l'app dir
    Ok(path_app_dir)
}

pub fn ensure_db(user_id: &str, extension: &str) -> Result<PathBuf, StorageError> {
    let mut path_db_file = ensure_app_dir()?;

    // Chemin jusqu'au fichier sqlite (.db ou .key dépendant de l'extension)
    let file_name = format!("{user_id}{extension}");
    path_db_file.push(file_name);

    // Créer le fichier db si non existant
    if !path_db_file.exists() {
        fs::File::create(&path_db_file).map_err(|_| StorageError::StorageFileCreate)?;
    }
    // S'assure d'appliquer permissions restrictives même si le fichier existe déjà
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
    // Si <user_id>.key existe, essayer de decoder+déchiffrer
    if file_exists(user_id, constant::DB_KEY_EXTENSION) {
        let key_path = file_path(user_id, constant::DB_KEY_EXTENSION);
        // TODO: Possible race condition entre le file_exists() et le read_to_string(), mais ignorée
        let wrapped_b64 =
            fs::read_to_string(&key_path).map_err(|_| StorageError::StorageFileRead)?;
        let wrapped_b64 = wrapped_b64.trim();

        let wrapped = base64::engine::general_purpose::STANDARD
            .decode(wrapped_b64)
            .map_err(|_| StorageError::DbKeyDecode)?;

        let db_key = crypto::unwrap_db_key(export_key, &wrapped)?;

        return Ok(db_key);
    }

    // Sinon générer la db_key et la wrap
    let (db_key, wrapped) = crypto::generate_wrapped_db_key(export_key)?;
    let wrapped_b64 = base64::engine::general_purpose::STANDARD.encode(wrapped);

    // S'assurer de l'existence du fichier .key
    let key_path = ensure_db(user_id, constant::DB_KEY_EXTENSION)?;

    // Inscrire la db_key chiffrée dans le fichier
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

// Check si existe deja une db + db_key (device existe)
// ou si l'un manque (device non existant et purge si incohérence)
pub fn reconcile_device_storage(user_id: &str) -> bool {
    let has_db = file_exists(user_id, constant::DB_EXTENSION);
    let has_key = file_exists(user_id, constant::DB_KEY_EXTENSION);

    // Si db présente mais pas la db_key -> considère la db comme corrompue/perdue donc purge
    if !has_db || !has_key {
        purge_storage(user_id);
    }

    has_db && has_key
}

pub fn init_device_storage(
    user_id: &str,
    export_key: &SecretSlice<u8>,
) -> Result<(String, SecretSlice<u8>, bool), AppError> {
    // Vérifie la présence des fichiers db/key et purge si problème
    let has_storage = reconcile_device_storage(&user_id.to_string());

    // Récupèration/Création de la clé de chiffrement de la db
    let db_key = get_db_key(&user_id.to_string(), export_key)?;

    // Si fichiers OK, tente de lire et récup le device_id stocké
    let device_id = if has_storage {
        match read_device_id(&db_key, user_id) {
            Ok(id) => id,
            Err(_) => None, // Aucun device_id n'a pu être récup -> considéré corrompue
        }
    } else {
        None
    };

    let mut is_new_device = false;

    let device_id = match device_id {
        Some(id) => {
            // Device connu, requête au serveur pour obtenir JWT Refresh au nom du device_id
            // TODO http::upgrade_jwt() pour obtenir JWT Refresh
            id
        }
        None => {
            // Nouveau device détecté, initialisation avec serveur
            is_new_device = true;

            // Récupération du nouveau device_id et du JWT Refresh
            let device_id = http::create_device()?;

            // Stockage du device_id dans db locale
            store_device_id(&db_key, user_id, device_id.as_str())?;

            device_id
        }
    };

    Ok((device_id, db_key, is_new_device))
}
