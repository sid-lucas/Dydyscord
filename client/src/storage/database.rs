use base64::Engine;
use openmls_sqlite_storage::Connection;
use std::os::unix::fs::PermissionsExt;
use std::{env, fs, path::PathBuf};

use crate::config::constant;
use crate::error::AppError;
use crate::storage::{crypto, error::StorageError};
use crate::transport::http;

pub struct CBORCodec;

impl Default for CBORCodec {
    fn default() -> Self {
        Self
    }
}

impl openmls_sqlite_storage::Codec for CBORCodec {
    type Error = AppError;

    fn to_vec<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, Self::Error> {
        let mut out = Vec::new();
        ciborium::ser::into_writer(value, &mut out)
            .map_err(|_| StorageError("CBOR codec serialize"))?;
        Ok(out)
    }

    fn from_slice<T: serde::de::DeserializeOwned>(slice: &[u8]) -> Result<T, Self::Error> {
        let mut input =
            ciborium::de::from_reader(slice).map_err(|_| StorageError("CBOR codec deserialize"))?;
        Ok(input)
    }
}

pub fn open_sqlcipher(db_key: &[u8; 32], user_id: &str) -> Result<Connection, StorageError> {
    let db_path = ensure_db(user_id, constant::DB_EXTENSION)?;
    let conn = Connection::open(db_path)
        .map_err(|_| StorageError("could not establish database sqlite connection"))?;

    let key_string = base64::engine::general_purpose::STANDARD.encode(db_key);
    conn.pragma_update(None, "key", &key_string)
        .map_err(|_| StorageError("could not read database (invalid key or corrupted file)"))?;
    Ok(conn)
}

pub fn ensure_app_dir() -> Result<PathBuf, StorageError> {
    let home = env::var("HOME").expect("HOME not set");

    // Chemin jusqu'au dossier de l'app
    let mut path_app_dir = PathBuf::from(home);
    let dir_name = format!(".{}", constant::APP_NAME);
    path_app_dir.push(dir_name);

    // Créer le dossier de l'app si non existant
    if !path_app_dir.exists() {
        fs::create_dir_all(&path_app_dir)
            .map_err(|_| StorageError("could not create directory app"))?;
    }
    // S'assure d'appliquer permissions restrictives même si le dossier existe déjà
    fs::set_permissions(&path_app_dir, fs::Permissions::from_mode(0o700))
        .map_err(|_| StorageError("could not set directory app permissions"))?;

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
        fs::File::create(&path_db_file)
            .map_err(|_| StorageError("could not create storage file"))?;
    }
    // S'assure d'appliquer permissions restrictives même si le fichier existe déjà
    fs::set_permissions(&path_db_file, fs::Permissions::from_mode(0o600))
        .map_err(|_| StorageError("could not set storage file permissions"))?;

    Ok(path_db_file)
}

pub fn file_path(user_id: &str, extension: &str) -> PathBuf {
    let home = env::var("HOME").expect("HOME not set");
    let path = PathBuf::from(home).join(format!(".{}", constant::APP_NAME));
    let file_name = format!("{user_id}{extension}");
    path.join(file_name)
}

pub fn file_exists(user_id: &str, extension: &str) -> bool {
    let home = env::var("HOME").expect("HOME not set");
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

pub fn get_db_key(user_id: &str, export_key: &[u8]) -> Result<[u8; 32], StorageError> {
    // Si <user_id>.key existe, essayer de decoder+déchiffrer
    if file_exists(user_id, constant::DB_KEY_EXTENSION) {
        let key_path = file_path(user_id, constant::DB_KEY_EXTENSION);
        // TODO: Possible race condition entre le file_exists() et le read_to_string(), mais ignorée
        let wrapped_b64 = fs::read_to_string(&key_path)
            .map_err(|_| StorageError("could not read db key file"))?;
        let wrapped_b64 = wrapped_b64.trim();

        let wrapped = base64::engine::general_purpose::STANDARD
            .decode(wrapped_b64)
            .map_err(|_| StorageError("could not decode db key"))?;

        let db_key = crypto::unwrap_db_key(export_key, &wrapped)
            .map_err(|_| StorageError("could not unwrap db key"))?;

        return Ok(db_key);
    }

    // Sinon créer clé + wrap
    let db_key: [u8; 32] = rand::random();
    let wrapped = crypto::wrap_db_key(export_key, &db_key)
        .map_err(|_| StorageError("could not wrap db key"))?;
    let wrapped_b64 = base64::engine::general_purpose::STANDARD.encode(wrapped);

    // S'assurer de l'existence du fichier .key
    let key_path = ensure_db(user_id, constant::DB_KEY_EXTENSION)?;

    // Inscrire la db_key chiffrée dans le fichier
    fs::write(&key_path, wrapped_b64)
        .map_err(|_| StorageError("could not write db key in file"))?;

    Ok(db_key)
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

    //TODO : remove, Print de debug
    println!("has_key : {}", has_key);
    println!("has_db : {}", has_db);

    has_db && has_key
}

pub fn initialize_device_storage(
    user_id: &str,
    export_key: &[u8],
) -> Result<([u8; 32], String), AppError> {
    // Reconcile + récupère si le device est reconnu avant potentielle init de la db
    let new_device = !reconcile_device_storage(&user_id.to_string());

    // Récupèration/Création de la clé de chiffrement de la db
    let db_key = get_db_key(&user_id.to_string(), &export_key)?;

    if new_device {
        // TODO : CREER LES TYPES OPENMLS NECESSAIRES ET STOCKER DANS LA DB LOCALE
        let device_id = http::create_device()?;
        Ok((db_key, device_id))
    } else {
        //
        // La c'est si le device est reconnu (a deja fait l'initialisation OpenMLS)

        // TODO : LIRE LES TYPES DE LA DB LOCALE

        Ok((db_key, "device_id_placeholder".to_string()))
    }
}
