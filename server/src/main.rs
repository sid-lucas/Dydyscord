use axum::{
    Router,
    routing::{get, post},
};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use opaque_ke::ServerSetup;

// ca c'est le truc ou jsp si faut le déplacer, il est dans opaque/mod.rs
use crate::opaque::OpaqueCiphersuite;

mod handlers;
mod models;
mod opaque;

const SERVER_ADDR: &str = "0.0.0.0:3000";

#[derive(Clone)]
pub struct ServerState {
    pub pool: PgPool,
    pub opaque_setup: Arc<ServerSetup<OpaqueCiphersuite>>,
}


#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // Load .env
    dotenv().ok();

    // Setup database connection pool
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // init état du serveur
    let server_state = ServerState {
        pool,
        opaque_setup: Arc::new(opaque::make_server_setup()),
    };

    let app = Router::new()
        .route("/", get(handlers::root))
        .route("/users", post(handlers::create_user))
        .route("/register/start", post(handlers::register_start))
        .route("/register/finish", post(handlers::register_finish))
        .with_state(server_state);

    let listener = tokio::net::TcpListener::bind(SERVER_ADDR).await.unwrap();
    println!("🚀 Listening on {}", SERVER_ADDR);
    let _ = axum::serve(listener, app).await;
}
