use crate::{
    auth::error::AuthError, mls::error::MlsError, storage::error::StorageError,
    transport::error::TransportError, ui::error::UiError,
};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    Auth(AuthError),
    Storage(StorageError),
    Transport(TransportError),
    Ui(UiError),
    Mls(MlsError),
}

impl From<MlsError> for AppError {
    fn from(err: MlsError) -> Self {
        AppError::Mls(err)
    }
}

impl From<StorageError> for AppError {
    fn from(err: StorageError) -> Self {
        AppError::Storage(err)
    }
}

impl From<TransportError> for AppError {
    fn from(err: TransportError) -> Self {
        AppError::Transport(err)
    }
}

impl From<UiError> for AppError {
    fn from(err: UiError) -> Self {
        AppError::Ui(err)
    }
}

impl From<AuthError> for AppError {
    fn from(err: AuthError) -> Self {
        AppError::Auth(err)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Auth(e) => write!(f, "Auth error: {}", e.0),
            AppError::Storage(e) => write!(f, "Storage error: {}", e),
            AppError::Transport(e) => write!(f, "Transport error: {}", e),
            AppError::Ui(e) => write!(f, "UI error: {}", e),
            AppError::Mls(e) => write!(f, "MLS error: {}", e),
        }
    }
}
