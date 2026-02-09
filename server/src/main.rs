use crate::config::init_server_state;
use axum::{
    Router, middleware,
    routing::{get, post},
};
use dotenv::dotenv;

mod api;
mod config;
mod constants;
mod database;
mod opaque;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    dotenv().ok();
    let secret = std::env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY must be set");
    constants::JWT_SECRET_KEY
        .set(secret)
        .expect("JWT_SECRET_KEY already set");

    let server_state = init_server_state().await;

    let app = Router::new()
        // Routes protégées par authentification (nécessite login):
        .route(
            "/device",
            post(api::).layer(middleware::from_fn(api::jwt::verify_jwt_auth)),
        )
        // Routes ouvertes :
        .route("/", get(api::root))
        .route("/register/start", post(api::auth::register_start))
        .route("/register/finish", post(api::auth::register_finish))
        .route("/login/start", post(api::auth::login_start))
        .route("/login/finish", post(api::auth::login_finish))
        .with_state(server_state);

    let listener = tokio::net::TcpListener::bind(constants::SERVER_ADDR)
        .await
        .unwrap();
    println!("🚀 Listening on {}", constants::SERVER_ADDR);
    let _ = axum::serve(listener, app).await;
}
