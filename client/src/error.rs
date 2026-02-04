use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("could not read input")]
    Input,

    #[error("could not reach server")]
    Network,

    #[error("username already exists")]
    UsernameTaken,

    #[error("server error")]
    Server,

    #[error("invalid server response")]
    InvalidResponse,

    #[error("bad request sent to server")]
    BadRequest,

    #[error("username or password is incorrect")]
    LoginFailed,

    #[error("unauthorized access")]
    Unauthorized,
}
