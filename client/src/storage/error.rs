use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    // Crypto
    #[error("could not encrypt with wrap key")]
    Encrypt,

    #[error("could not decrypt with wrap key")]
    Decrypt,

    #[error("invalid db_key length (has to be 32 bytes)")]
    DbKeyLength,

    #[error("invalid export_key length (has to be 32+ bytes)")]
    ExportKeyLength,

    #[error("invalid envelope length (has to be 13+ bytes)")]
    EnvelopeLength,

    #[error("invalid envelope version")]
    EnvelopeVersion,

    // Codec / serialization
    #[error("could not serialize CBOR codec")]
    CborCodecSerialize,

    #[error("could not deserialize CBOR codec")]
    CborCodecDeserialize,

    // Database / SQLite
    #[error("could not connect to database")]
    DatabaseConnect,

    #[error("could not read database")]
    DatabaseRead,

    #[error("could not create schema")]
    DatabaseSchema,

    #[error("could not perfom query on database")]
    DatabaseQuery,

    // Fichiers / permissions
    #[error("could not create app directory")]
    AppDirectoryCreate,

    #[error("could not set app directory permissions")]
    AppDirectoryPermissions,

    #[error("could not create storage file")]
    StorageFileCreate,

    #[error("could not set storage file permissions")]
    StorageFilePermissions,

    #[error("could not read db key from file")]
    StorageFileRead,

    #[error("could not decode db key")]
    DbKeyDecode,

    #[error("could not store db key")]
    DbKeyStore,

    // App state
    #[error("device_id was not found in database")]
    DeviceIdMissing, // TODO unused
}
