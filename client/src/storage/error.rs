use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("could not unwrap db key")]
    UnwrapDbKey,

    #[error("could not derive wrap key")]
    DeriveWrapKey,

    #[error("could not encrypt with wrap key")]
    EncryptWithWrapKey,

    #[error("could not decrypt with wrap key")]
    DecryptWithWrapKey,

    #[error("invalid envelope length")]
    InvalidEnvelopeLength,

    #[error("invalid envelope version")]
    InvalidEnvelopeVersion,

    #[error("could not serialize CBOR codec")]
    SerializeCborCodec,

    #[error("could not deserialize CBOR codec")]
    DeserializeCborCodec,

    #[error("could not connect to database")]
    ConnectDatabase,

    #[error("could not read database")]
    ReadDatabase,

    #[error("could not create app directory")]
    CreateAppDirectory,

    #[error("could not set app directory permissions")]
    SetAppDirectoryPermissions,

    #[error("could not create storage file")]
    CreateStorageFile,

    #[error("could not set storage file permissions")]
    SetStorageFilePermissions,

    #[error("could not read db key from file")]
    ReadDbKeyFile,

    #[error("could not decode db key")]
    DecodeDbKey,

    #[error("could not wrap db key")]
    WrapDbKey,

    #[error("could not store db key")]
    StoreDbKey,

    #[error("device_id was not found in database")]
    DeviceIdMissing,
}
