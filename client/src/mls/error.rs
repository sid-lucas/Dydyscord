use thiserror::Error;

#[derive(Debug, Error)]
pub enum MlsError {
    #[error("could not migrate provider's database")]
    Migration,

    #[error("could not create signature keys")]
    SignatureKeysCreate,

    #[error("could not store signature keys")]
    SignatureKeysStore,

    #[error("could not create key package")]
    KeyPackageCreate,
}
