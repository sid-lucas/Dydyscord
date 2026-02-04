use crate::config::init_server_state;
use axum::{
    Router,
    routing::{get, post},
};

mod config;
mod database;
mod api;
mod opaque;

const SERVER_ADDR: &str = "0.0.0.0:3000";

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let server_state = init_server_state().await;

    let app = Router::new()
        .route("/", get(api::root))
        .route("/register/start", post(api::auth::register_start))
        .route("/register/finish", post(api::auth::register_finish))
        .route("/login/start", post(api::auth::login_start))
        .route("/login/finish", post(api::auth::login_finish))
        .with_state(server_state);

    let listener = tokio::net::TcpListener::bind(SERVER_ADDR).await.unwrap();
    println!("🚀 Listening on {}", SERVER_ADDR);
    let _ = axum::serve(listener, app).await;
}
