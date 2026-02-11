use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("could not start opaque register")]
    OpaqueRegisterStart,

    #[error("could not finish opaque register")]
    OpaqueRegisterFinish,

    #[error("could not start opaque login")]
    OpaqueLoginStart,

    #[error("could not finish opaque login")]
    OpaqueLoginFinish,

    #[error("could not decode base64 opaque response")]
    OpaqueDecode,

    #[error("could not deserialize opaque response")]
    OpaqueDeserialize,

    #[error("tried to use an unset session db_key")]
    SessionDbKeyUnset,
}
