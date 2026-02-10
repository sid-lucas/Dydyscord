use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("could not connect to local database")]
    DatabaseConnection,

    #[error("could not read database (invalid key or corrupted file)")]
    DatabaseRetrieval,

    #[error("could not read database key")]
    KeyRead,

    #[error("could not write database key")]
    KeyWrite,

    #[error("could not decode database key")]
    KeyDecode,

    #[error("could not unwrap database key")]
    KeyUnwrap,

    #[error("could not wrap database key")]
    KeyWrap,

    #[error("could not create application directory in home")]
    DirCreate,

    #[error("could not set restrictive permissions on application directory")]
    DirPermission,

    #[error("could not create storage file in application directory")]
    FileCreate,

    #[error("could not set restrictive permissions on storage file")]
    FilePermission,

    #[error("could not serialize/deserialize")]
    CodecSerde,

    #[error("sqlite migration failed")]
    Migration,
}
