use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("could not reach server")]
    Network,

    #[error("server error")]
    Server,

    #[error("invalid server response")]
    InvalidResponse,

    #[error("bad request sent to server")]
    BadRequest,

    #[error("unauthorized access")]
    Unauthorized,

    #[error("internal error")]
    Internal,

    #[error("username already exists")]
    UsernameTaken,

    #[error("username or password is incorrect")]
    LoginFailed,
}
