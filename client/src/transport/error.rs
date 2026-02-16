use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum TransportError {
    // --- Transport / IO ---
    #[error("network error")]
    Network,

    // --- HTTP protocol ---
    #[error("unexpected status code")]
    UnexpectedStatus,

    #[error("invalid or malformed response body")]
    InvalidResponse,

    // --- Standard HTTP statuses ---
    #[error("bad request")]
    BadRequest,

    #[error("unauthorized")]
    Unauthorized,

    #[error("conflict")]
    Conflict,

    #[error("internal server error")]
    Internal,

    // --- Domain logic ---
    #[error("username already exists")]
    UsernameTaken,

    #[error("username or password is incorrect")]
    LoginFailed,
}
