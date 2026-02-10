use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("could not connect to local database")]
    DatabaseConnection,

    #[error("could not retrieve data from database")]
    DatabaseRetrieval,

    #[error("could not read database key")]
    KeyRead,

    #[error("could not decode database key")]
    KeyDecode,

    #[error("could not unwrap database key")]
    KeyUnwrap,
}
