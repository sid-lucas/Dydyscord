use crate::config::server::{self};
use axum::{
    Router, middleware,
    routing::{get, post},
};
use config::{constant, server::ServerState};
use dotenv::dotenv;
use handler::{auth, root};

mod config;
mod database;
mod handler;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    dotenv().ok();
    let secret = std::env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY must be set");
    constant::JWT_SECRET_KEY
        .set(secret)
        .expect("JWT_SECRET_KEY already set");

    let server_state = server::init_server_state().await;

    let app = Router::new()
        // Routes protégées par authentification (nécessite login):
        .route(
            "/device",
            post(handler::auth::device::create_device)
                .layer(middleware::from_fn(handler::auth::jwt::verify_jwt_auth)),
        )
        // Routes ouvertes :
        .route("/", get(handler::root::root))
        .route(
            "/register/start",
            post(handler::auth::opaque::register_start),
        )
        .route(
            "/register/finish",
            post(handler::auth::opaque::register_finish),
        )
        .route("/login/start", post(handler::auth::opaque::login_start))
        .route("/login/finish", post(handler::auth::opaque::login_finish))
        .with_state(server_state);

    let listener = tokio::net::TcpListener::bind(constant::SERVER_ADDR)
        .await
        .unwrap();
    println!("🚀 Listening on {}", constant::SERVER_ADDR);
    let _ = axum::serve(listener, app).await;
}
