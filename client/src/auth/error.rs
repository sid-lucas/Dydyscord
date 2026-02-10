use thiserror::Error;

use std::fmt;

#[derive(Debug)]
pub struct AuthError {
    message: String,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AuthError {}
