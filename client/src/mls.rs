use openmls::prelude::{tls_codec::*, *};
use openmls_basic_credential::SignatureKeyPair;
use openmls_rust_crypto::{OpenMlsRustCrypto, RustCrypto};
use openmls_sqlite_storage::{Connection, SqliteStorageProvider};
use openmls_traits::OpenMlsProvider;

use crate::error::ClientError;
use crate::mls::storage::CBORCodec;

mod crypto;
pub mod storage;
pub mod test;

pub struct MyProvider {
    crypto: RustCrypto,
    rand: RustCrypto,
    storage: SqliteStorageProvider<CBORCodec, Connection>,
}

impl OpenMlsProvider for MyProvider {
    type CryptoProvider = RustCrypto;
    type RandProvider = RustCrypto;
    type StorageProvider = SqliteStorageProvider<CBORCodec, Connection>;

    fn storage(&self) -> &Self::StorageProvider {
        &self.storage
    }
    fn crypto(&self) -> &Self::CryptoProvider {
        &self.crypto
    }
    fn rand(&self) -> &Self::RandProvider {
        &self.rand
    }
}

pub fn prepare_provider(db_key: &[u8; 32], user_id: &str) -> Result<MyProvider, ClientError> {
    let conn = storage::open_sqlcipher(db_key, user_id)?;

    let mut storage = SqliteStorageProvider::<CBORCodec, _>::new(conn);

    storage
        .run_migrations()
        .map_err(|_| ClientError::Internal)?;

    Ok(MyProvider {
        crypto: RustCrypto::default(),
        rand: RustCrypto::default(),
        storage,
    })
}

// A helper to create and store credentials.
/*
fn generate_credential_with_key(
    identity: Vec<u8>,
    credential_type: CredentialType,
    signature_algorithm: SignatureScheme,
    provider: &impl OpenMlsProvider,
) -> (CredentialWithKey, SignatureKeyPair) {
    let credential = BasicCredential::new(identity);
    let signature_keys =
        SignatureKeyPair::new(signature_algorithm).expect("Error generating a signature key pair.");

    // Store the signature key into the key store so OpenMLS has access
    // to it.
    signature_keys
        .store(provider.storage())
        .expect("Error storing signature keys in key store.");

    (
        CredentialWithKey {
            credential: credential.into(),
            signature_key: signature_keys.public().into(),
        },
        signature_keys,
    )
}
     */

// A helper to create key package bundles.
/*
fn generate_key_package(
    ciphersuite: Ciphersuite,
    provider: &impl OpenMlsProvider,
    signer: &SignatureKeyPair,
    credential_with_key: CredentialWithKey,
) -> KeyPackageBundle {
    // Create the key package
    KeyPackage::builder()
        .build(ciphersuite, provider, signer, credential_with_key)
        .unwrap()
}
        */
