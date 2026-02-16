use base64::Engine;
use once_cell::sync::OnceCell;
use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use openmls_rust_crypto::RustCrypto;
use openmls_sqlite_storage::{Connection, SqliteStorageProvider};
use openmls_traits::OpenMlsProvider;
use openmls_traits::storage::StorageProvider;

use crate::mls::storage::CBORCodec;
use crate::mls::{MyProvider, crypto, storage};

// OnceCell because it allows defining a global value initialized once and readable everywhere
// static mut: would not be safe / risk of race condition
// global var with mutex: heavy and unnecessary if the value does not change
// store in EncryptedCodec: impossible because Codec enforces static functions
static EXPORT_KEY: OnceCell<[u8; 32]> = OnceCell::new();

fn init_codec_key(key: [u8; 32]) {
    let _ = EXPORT_KEY.set(key);
}

fn test() -> Result<(), String> {
    init_codec_key([42u8; 32]); // Fixed key for test

    let user_id = "ahahah";
    let db_path = storage::ensure_db(user_id);
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

    // 1) SignatureKeyPair (stored via internal StorageId)
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

    // 2) KeyPackageBundle (stored automatically by build)
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
