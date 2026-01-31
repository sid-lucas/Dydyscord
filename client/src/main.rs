use crate::user::User;
use inquire::{InquireError, Select, Text};
use inquire_derive::Selectable;
use openmls::prelude::{tls_codec::*, *};
use openmls_basic_credential::SignatureKeyPair;
use openmls_rust_crypto::OpenMlsRustCrypto;
use std::fmt;
mod user;

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
    CreateAccount,
    ConnectToServer,
}

impl fmt::Display for Choice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Choice::CreateAccount => write!(f, "Create account"),
            Choice::ConnectToServer => write!(f, "Connect to server"),
        }
    }
}

fn create_user() -> Option<User> {
    let username = Text::new("Enter your username:").prompt();
    match username {
        Ok(name) => Some(User::new(name)),
        Err(_) => None,
    }
}

fn main() {
    // Define ciphersuite ...
    let ciphersuite = Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519;
    // ... and the crypto provider to use.
    let provider = &OpenMlsRustCrypto::default();

    loop {
        let answer = Choice::select("Choose an option:")
            .prompt()
            .expect("An error occurred");

        match answer {
            Choice::CreateAccount => {
                let user = create_user();
                match user {
                    Some(u) => println!("Created user: {}", u),
                    None => println!("Failed to create user."),
                }
            }
            Choice::ConnectToServer => {
                println!("Connecting to server...");
                // Further connection logic would go here.
            }
        }
    }
}
