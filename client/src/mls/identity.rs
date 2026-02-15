use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use openmls_traits::OpenMlsProvider;
use serde::Serialize;

use crate::config::constant;
use crate::error::AppError;
use crate::mls::error::MlsError;
use crate::transport::http;
use openmls_rust_crypto::OpenMlsRustCrypto;

// A helper to create and store credentials.
fn generate_credential_with_key(
    identity: &str,
) -> Result<(CredentialWithKey, SignatureKeyPair), AppError> {
    // Create the credential with the identity (device_id)
    let identity = identity.as_bytes().to_vec();
    let credential = BasicCredential::new(identity);
    let provider = &OpenMlsRustCrypto::default();

    // Create the associated signature key pair
    let signature_keys = SignatureKeyPair::new(constant::OPENMLS_CIPHERSUITE.signature_algorithm())
        .map_err(|_| MlsError::SignatureKeysCreate)?;

    // Store the pair in the account-associated db, so OpenMLS has access to it
    signature_keys
        .store(provider.storage())
        .map_err(|_| MlsError::SignatureKeysStore)?;

    Ok((
        CredentialWithKey {
            credential: credential.into(),
            signature_key: signature_keys.public().into(),
        },
        signature_keys,
    ))
}

// A helper to create key package bundles.
fn generate_key_package(
    signer: &SignatureKeyPair,
    credential_with_key: CredentialWithKey,
) -> Result<KeyPackageBundle, AppError> {
    // Create the key package
    let provider = &OpenMlsRustCrypto::default();
    let key_package = KeyPackage::builder()
        .build(
            constant::OPENMLS_CIPHERSUITE,
            provider,
            signer,
            credential_with_key,
        )
        .map_err(|_| MlsError::KeyPackageCreate)?;

    Ok(key_package)
}

pub fn init_openmls(is_new_device: bool, device_id: String) -> Result<(), AppError> {
    // if new device: create the necessary openmls elements and put them in the db

    if is_new_device {
        // Create the credential and signature keys
        let (credential_with_key, signature_keys) = generate_credential_with_key(&device_id)?;

        // Create 100 key package bundles
        let mut key_packages = Vec::new();
        for _ in 0..100 {
            let key_package = generate_key_package(&signature_keys, credential_with_key.clone())?;
            key_packages.push(key_package);
        }
        // serialize the key packages and send them to the server to be stored in the db
        http::send_key_packages(device_id, key_packages)?;
    }

    Ok(())
}
