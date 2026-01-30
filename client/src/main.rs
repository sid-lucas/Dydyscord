use inquire::{InquireError, Select, Text};
use inquire_derive::Selectable;
use openmls::prelude::{tls_codec::*, *};
use openmls_basic_credential::SignatureKeyPair;
use openmls_rust_crypto::OpenMlsRustCrypto;
use std::fmt;

// A helper to create and store credentials.
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

// A helper to create key package bundles.
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

#[derive(Debug, Copy, Clone, Selectable)]
enum Choice {
    CreateMember,
    CreateKeyPackage,
}

impl fmt::Display for Choice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Choice::CreateMember => write!(f, "Create member"),
            Choice::CreateKeyPackage => write!(f, "Create key package"),
        }
    }
}

impl Choice {
    fn execute(&self) {
        match self {
            Choice::CreateMember => println!("Create member"),
            Choice::CreateKeyPackage => println!("Create key package"),
        }
    }
}

fn main() {
    // Define ciphersuite ...
    let ciphersuite = Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519;
    // ... and the crypto provider to use.
    let provider = &OpenMlsRustCrypto::default();

    let answer = Choice::select("Choose an option:").prompt();

    match answer {
        Ok(choice) => choice.execute(),
        Err(_) => println!("There was an error, please try again"),
    }
}
