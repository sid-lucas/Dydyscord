use axum::http::StatusCode;

pub async fn root() -> (StatusCode, &'static str) {
    (StatusCode::OK, "Dydyscord Server is running!")
}
