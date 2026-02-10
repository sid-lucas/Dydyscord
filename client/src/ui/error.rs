use thiserror::Error;

#[derive(Debug, Error)]
pub enum UiError {
    #[error("could not read username")]
    Username,
    #[error("could not read password")]
    Password,
}
