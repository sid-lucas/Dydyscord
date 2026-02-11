use thiserror::Error;

#[derive(Debug, Error)]
pub enum MlsError {
    #[error("could not migrate provider's database")]
    Migration,
}
