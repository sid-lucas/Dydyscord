use crate::{
    mls::error::MlsError, storage::error::StorageError, transport::error::TransportError,
    ui::error::UiError,
};
use base64::DecodeError as Base64Error;
use opaque_ke::errors::ProtocolError as OpaqueError;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    Storage(StorageError),
    Transport(TransportError),
    Ui(UiError),
    Mls(MlsError),
    Opaque(OpaqueError),
    Base64Error(Base64Error),
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

impl From<OpaqueError> for AppError {
    fn from(err: OpaqueError) -> Self {
        AppError::Opaque(err)
    }
}

impl From<Base64Error> for AppError {
    fn from(err: Base64Error) -> Self {
        AppError::Opaque(OpaqueError::SerializationError)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Storage(e) => write!(f, "Storage error: {}", e),
            AppError::Transport(e) => write!(f, "Transport error: {}", e),
            AppError::Ui(e) => write!(f, "UI error: {}", e),
            AppError::Mls(e) => write!(f, "MLS error: {}", e),
            AppError::Opaque(e) => write!(f, "Opaque error: {}", e),
            AppError::Base64Error(e) => write!(f, "Base64 decoding error: {}", e),
        }
    }
}
