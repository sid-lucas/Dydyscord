use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use openmls_rust_crypto::RustCrypto;
use openmls_sqlite_storage::{Connection, SqliteStorageProvider};
use openmls_traits::OpenMlsProvider;
use openmls_traits::storage::StorageProvider;

use crate::mls::storage::EncryptedCodec;
use crate::mls::{MyProvider, crypto, storage};

pub fn test() -> Result<(), String> {
    crypto::init_codec_key([42u8; 32]); // Clé fixe pour test

    let db_path = storage::ensure_localdb_path();
    let conn = Connection::open(db_path).map_err(|e| format!("open db: {e:?}"))?;

    let mut storage = SqliteStorageProvider::<EncryptedCodec, _>::new(conn);
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
    let stored: Option<KeyPackageBundle> =
        provider
            .storage()
            .key_package(&hash_ref)
            .map_err(|e| format!("read key package: {e:?}"))?;
    let stored = stored.ok_or("read key package: not found".to_string())?;

    if stored.key_package() != key_package_bundle.key_package() {
        return Err("read key package: mismatch".to_string());
    }

    Ok(())
}
