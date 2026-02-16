use axum::{
    Router, middleware,
    routing::{get, post},
};
use config::constant;

use crate::config::server::{self};

mod config;
mod database;
mod handler;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let server_state = server::ServerState::new().await;

    let app = Router::new()
        // Routes protected by a Session JWT:
        .route(
            "/test/session",
            get(handler::root::root).layer(middleware::from_fn_with_state(
                server_state.clone(),
                handler::jwt::verify_jwt_session,
            )),
        )
        .route(
            "/device/{id}/keypackages",
            post(handler::mls::update_key_packages).layer(middleware::from_fn_with_state(
                server_state.clone(),
                handler::jwt::verify_jwt_session,
            )),
        )
        .route(
            "/user/keypackage", // TODO : Change route name?
            post(handler::mls::get_keypackage_from_username).layer(middleware::from_fn_with_state(
                server_state.clone(),
                handler::jwt::verify_jwt_session,
            )),
        )
        .route(
            "/welcome",
            post(handler::mls::store_welcome).layer(middleware::from_fn_with_state(
                server_state.clone(),
                handler::jwt::verify_jwt_session,
            )),
        )
        .route(
            "/welcome",
            get(handler::mls::fetch_welcome).layer(middleware::from_fn_with_state(
                server_state.clone(),
                handler::jwt::verify_jwt_session,
            )),
        )
        // Routes protected by an Auth JWT:
        .route(
            "/device",
            post(handler::auth::device::create_device).layer(middleware::from_fn_with_state(
                server_state.clone(),
                handler::jwt::verify_jwt_auth,
            )),
        )
        .route(
            "/device",
            get(handler::auth::device::get_device).layer(middleware::from_fn_with_state(
                server_state.clone(),
                handler::jwt::verify_jwt_auth,
            )),
        )
        // Open routes:
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
