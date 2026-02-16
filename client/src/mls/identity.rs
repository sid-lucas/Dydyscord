use base64::Engine;
use common::{KeyPackagesUploadRequest, UserKeyPackageRequest, WelcomeStoreRequest};
use openmls::prelude::tls_codec::Serialize as TlsSerialize;
use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use openmls_rust_crypto::OpenMlsRustCrypto;
use openmls_traits::OpenMlsProvider;
use secrecy::SecretSlice;

use crate::config::constant;
use crate::error::AppError;
use crate::mls::error::MlsError;
use crate::mls::provider::MyProvider;
use crate::storage;
use crate::transport::http;

// TODO Rename file, does not correspond to what it does

// A helper to create and store credentials.
fn generate_credential_with_key(
    identity: &str,
) -> Result<(CredentialWithKey, SignatureKeyPair), AppError> {
    // Create the credential with the identity (device_id)
    let identity = identity.as_bytes().to_vec();
    let credential = BasicCredential::new(identity);

    // Create the associated signature key pair
    let signature_keys = SignatureKeyPair::new(constant::OPENMLS_CIPHERSUITE.signature_algorithm())
        .map_err(|_| MlsError::SignatureKeysCreate)?;

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

// A helper to retrieve the signature key pair and credential from current user
fn load_signer_and_credential(
    provider: &MyProvider,
    device_id: &str,
    signature_public_key: Vec<u8>,
) -> Result<(SignatureKeyPair, CredentialWithKey), AppError> {
    let scheme = constant::OPENMLS_CIPHERSUITE.signature_algorithm();

    let signer = SignatureKeyPair::read(provider.storage(), &signature_public_key, scheme)
        .ok_or(MlsError::SignatureKeysRead)?;

    let credential = BasicCredential::new(device_id.as_bytes().to_vec());
    let credential_with_key = CredentialWithKey {
        credential: credential.into(),
        signature_key: signer.public().into(),
    };

    Ok((signer, credential_with_key))
}

pub fn init_openmls(
    db_key: &SecretSlice<u8>,
    user_id: &str,
    device_id: &str,
    provider: &MyProvider,
    is_new_device: bool,
) -> Result<(), AppError> {
    // if new device: create the necessary openmls elements and put them in the db

    if is_new_device {
        // Create the credential and signature keys
        let (credential_with_key, signature_keys) = generate_credential_with_key(&device_id)?;

        // Store the private_key in the account-associated db, so OpenMLS has access to it
        signature_keys
            .store(provider.storage())
            .map_err(|_| MlsError::SignatureKeysStore)?;

        // Store the public signature key
        let pubkey_b64 = base64::engine::general_purpose::STANDARD.encode(signature_keys.public());
        storage::database::store_signature_pub_key(db_key, user_id, &pubkey_b64)?;

        // Create 100 key package
        let mut kp_bytes = Vec::with_capacity(100);
        for _ in 0..100 {
            let kp_bundle = generate_key_package(&signature_keys, credential_with_key.clone())?;
            let bytes = kp_bundle
                .key_package()
                .tls_serialize_detached()
                .map_err(|_| MlsError::KeyPackageCreate)?;
            kp_bytes.push(bytes);
        }
        // serialize the key packages and send them to the server to be stored in the db
        http::send_key_packages(
            device_id,
            KeyPackagesUploadRequest {
                key_packages: kp_bytes,
            },
        )?;
    }

    Ok(())
}

pub fn init_group(
    db_key: &SecretSlice<u8>,
    user_id: &str,
    device_id: &str,
    provider: &MyProvider,
    user_to_add: &str,
) -> Result<(), AppError> {
    let response = http::create_group(UserKeyPackageRequest {
        username: user_to_add.to_string(),
    })?;

    // Retrieve the public signature key
    let pubkey_b64 = storage::database::read_signature_pub_key(db_key, user_id)?;
    let signature_pubkey = base64::engine::general_purpose::STANDARD
        .decode(pubkey_b64)
        .map_err(|_| MlsError::PubKeyDecode)?;

    // Retrieve the signer (with the public signature key) and the credential
    let (signer, credential_with_key) =
        load_signer_and_credential(provider, device_id, signature_pubkey)?;

    // TODO: set to false if we want privacy-first and send ratchet tree out-of-band
    let cfg = MlsGroupCreateConfig::builder()
        .use_ratchet_tree_extension(true)
        .build();

    // Create the group with the user information.
    let mut new_group = MlsGroup::new(provider, &signer, &cfg, credential_with_key)
        .map_err(|_| MlsError::GroupCreate)?;

    // Deserialize and verify keypackage(s) received
    let mut key_packages = Vec::with_capacity(response.len());
    for dk in &response {
        let kp_in = KeyPackageIn::tls_deserialize_exact_bytes(&dk.key_package)
            .map_err(|_| MlsError::KeyPackageDeserialize)?;
        let kp = kp_in
            .validate(provider.crypto(), ProtocolVersion::Mls10)
            .map_err(|_| MlsError::KeyPackageInvalid)?;
        key_packages.push(kp);
    }

    // Add members (1 keypackage per device)
    let (commit_msg, welcome_msg, group_info) = new_group
        .add_members(provider, &signer, &key_packages)
        .map_err(|_| MlsError::AddMembers)?;

    // Apply the pending commit locally to make the group operational.
    new_group
        .merge_pending_commit(provider)
        .map_err(|_| MlsError::MergePendingCommit)?;

    // Serialize the Welcome message so it can be sent to the server for delivery.
    let welcome_bytes = welcome_msg
        .tls_serialize_detached()
        .map_err(|_| MlsError::WelcomeSerialize)?;
    // Encode in base64
    let welcome_b64 = base64::engine::general_purpose::STANDARD.encode(welcome_bytes);

    // Retrieve all the device_id that have been had to the group
    let device_ids = response.iter().map(|dk| dk.device_id).collect();

    println!("Sending welcome to serveur...");

    http::send_welcome(WelcomeStoreRequest {
        device_ids,
        welcome_b64,
    })?;

    Ok(())
}

pub fn fetch_welcome(provider: &MyProvider) -> Result<(), AppError> {
    let response = http::fetch_welcome()?;

    // List of the groups joined with the fetched welcome messages
    let mut groups: Vec<MlsGroup> = Vec::new();

    for item in response {
        // Decode the welcomes
        let welcome_bytes = base64::engine::general_purpose::STANDARD
            .decode(item.welcome_b64)
            .map_err(|_| MlsError::WelcomeDecode)?;

        // Deserialize the welcomes
        let mls_message_in = MlsMessageIn::tls_deserialize_exact_bytes(&welcome_bytes)
            .map_err(|_| MlsError::WelcomeDeserialize)?;

        // ... and inspect the message
        let welcome = match mls_message_in.extract() {
            MlsMessageBodyIn::Welcome(welcome) => welcome,
            // We know it's a welcome message, so we ignore all other cases
            _ => unreachable!("Unexpected message type."),
        };

        // Build a staged join for the group in order to inspect the welcome
        let staged_join = StagedWelcome::new_from_welcome(
            provider,
            &MlsGroupJoinConfig::default(),
            welcome,
            // TODO: if ratchet_tree_extension is false, we have to provide the public ratchet tree here
            None,
        )
        .map_err(|_| MlsError::StagedWelcomeCreate)?;

        // Finally : join the group
        let joined_group = staged_join
            .into_group(provider)
            .map_err(|_| MlsError::GroupJoin)?;
        groups.push(joined_group);

        // TODO : Stocker nous même le group_id dans une table créé par nos soins dans SQLite
        // Ce group_id nous permet de rerecupérer les info mls du group pour send message...
    }
    Ok(())
}
