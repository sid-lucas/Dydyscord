use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use openmls_traits::OpenMlsProvider;

use crate::config::constant;
use crate::error::AppError;
use crate::mls::error::MlsError;

// A helper to create and store credentials.
fn generate_credential_with_key(
    identity: &str,
    signature_algorithm: SignatureScheme,
    provider: &impl OpenMlsProvider,
) -> Result<(CredentialWithKey, SignatureKeyPair), AppError> {
    // Create the credential with the identity (device_id)
    let identity = identity.as_bytes().to_vec();
    let credential = BasicCredential::new(identity);

    // Create the associated signature key pair
    let signature_keys =
        SignatureKeyPair::new(signature_algorithm).map_err(|_| MlsError::SignatureKeysCreate)?;

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
    provider: &impl OpenMlsProvider,
    signer: &SignatureKeyPair,
    credential_with_key: CredentialWithKey,
) -> Result<KeyPackageBundle, AppError> {
    // Create the key package
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

pub fn init_openmls(is_new_device: bool) -> Result<(), AppError> {
    // if new device: create the necessary openmls elements and put them in the db

    // TODO

    Ok(())
}
