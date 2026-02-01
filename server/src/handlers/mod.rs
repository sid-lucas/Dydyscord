use axum::{http::StatusCode};

pub mod auth;
pub mod users;

pub async fn root() -> (StatusCode, &'static str) {
    (StatusCode::OK, "Dydyscord Server is running!")
}