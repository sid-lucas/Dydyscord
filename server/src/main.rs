use crate::config::server::{self};
use axum::{
    Router, middleware,
    routing::{get, post},
};
use config::constant;
use handler::auth;
use std::sync::Arc;

mod config;
mod database;
mod handler;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let server_state = server::ServerState::new().await;

    let app = Router::new()
        // Routes protégées par un JWT Access:
        // None
        // Routes protégées par un JWT Refresh:
        // None
        // Routes protégées par un JWT Auth:
        // TODO Add route qui améliore notre JWT Auth en JWT Refresh
        .route(
            "/device",
            post(handler::auth::device::create_device).layer(middleware::from_fn_with_state(
                server_state.clone(),
                handler::auth::jwt::verify_jwt_auth,
            )),
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
