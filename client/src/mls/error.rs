use thiserror::Error;

#[derive(Debug, Error)]
pub enum MlsError {
    #[error("could not migrate provider's database")]
    Migration,

    #[error("could not create signature keys")]
    SignatureKeysCreate,

    #[error("could not store signature keys")]
    SignatureKeysStore,

    #[error("could not retrieve signature keys")]
    SignatureKeysRead,

    #[error("could not decode public signature key")]
    PubKeyDecode,

    #[error("could not create key package")]
    KeyPackageCreate,

    #[error("could not deserialise key package")]
    KeyPackageDeserialize,

    #[error("key package is invalid")]
    KeyPackageInvalid,

    #[error("could not create a new group")]
    GroupCreate,

    #[error("could not join a group")]
    GroupJoin,

    #[error("could not add member into group")]
    AddMembers,

    #[error("could not merge pending commit")]
    MergePendingCommit,

    #[error("could not serialize welcome message")]
    WelcomeSerialize,

    #[error("could not decode welcome message")]
    WelcomeDecode,

    #[error("could not deserialize MLS message")]
    WelcomeDeserialize,

    #[error("could not create staged join from welcome")]
    StagedWelcomeCreate,
}
