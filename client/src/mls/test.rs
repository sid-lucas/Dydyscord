use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use openmls_rust_crypto::RustCrypto;
use openmls_sqlite_storage::{Connection, SqliteStorageProvider};
use openmls_traits::OpenMlsProvider;
use openmls_traits::storage::StorageProvider;

use crate::mls::storage::{CBORCodec, EncryptedCodec};
use crate::mls::{MyProvider, crypto, storage};

use once_cell::sync::OnceCell;

// OnceCell car permet de définir une valeur global initialisée unde fois et accessible partout en lecture
// static mut : ne serait pas safe / risque de race condition
// var globale avec mutex : lourd et inutile si valeur change pas
// stocker dans EncryptedCodec : impossible car Codec impose fonctions statiques
static EXPORT_KEY: OnceCell<[u8; 32]> = OnceCell::new();

pub fn init_codec_key(key: [u8; 32]) {
    let _ = EXPORT_KEY.set(key);
}

pub fn test() -> Result<(), String> {
    init_codec_key([42u8; 32]); // Clé fixe pour test

    let db_path = storage::ensure_localdb_path();
    let conn = Connection::open(db_path).map_err(|e| format!("open db: {e:?}"))?;

    let key_bytes = EXPORT_KEY.get().ok_or("export key not set")?;
    let key_string = base64::engine::general_purpose::STANDARD.encode(key_bytes);

    conn.pragma_update(None, "key", &key_string)
        .map_err(|e| format!("pragma key: {e:?}"))?;

    let mut storage = SqliteStorageProvider::<CBORCodec, _>::new(conn);
    storage
        .run_migrations()
        .map_err(|e| format!("run migrations: {e:?}"))?;

    let provider = MyProvider {
        crypto: RustCrypto::default(),
        rand: RustCrypto::default(),
        storage,
    };

    let ciphersuite = Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519;

    // 1) SignatureKeyPair (stocké via StorageId interne)
    let signer = SignatureKeyPair::new(ciphersuite.signature_algorithm())
        .map_err(|e| format!("generate signer: {e:?}"))?;
    signer
        .store(provider.storage())
        .map_err(|e| format!("store signer: {e:?}"))?;

    let loaded = SignatureKeyPair::read(
        provider.storage(),
        signer.public(),
        signer.signature_scheme(),
    )
    .ok_or("read signer: not found".to_string())?;
    if loaded.public() != signer.public() {
        return Err("read signer: public key mismatch".to_string());
    }

    // 2) KeyPackageBundle (stocké automatiquement par build)
    let credential = BasicCredential::new(b"device-1".to_vec());
    let key_package_bundle = KeyPackage::builder()
        .build(
            ciphersuite,
            &provider,
            &signer,
            CredentialWithKey {
                credential: credential.into(),
                signature_key: signer.public().into(),
            },
        )
        .map_err(|e| format!("build key package: {e:?}"))?;

    let hash_ref = key_package_bundle
        .key_package()
        .hash_ref(provider.crypto())
        .map_err(|e| format!("hash ref: {e:?}"))?;
    let stored: Option<KeyPackageBundle> = provider
        .storage()
        .key_package(&hash_ref)
        .map_err(|e| format!("read key package: {e:?}"))?;
    let stored = stored.ok_or("read key package: not found".to_string())?;

    if stored.key_package() != key_package_bundle.key_package() {
        return Err("read key package: mismatch".to_string());
    }

    Ok(())
}
