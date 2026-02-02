use axum::{
    Router,
    routing::{get, post},
};
use dotenv::dotenv;
use opaque_ke::ServerSetup;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use redis::aio::ConnectionManager;
use redis::Client as RedisClient;
use std::sync::Arc;

use crate::opaque::OpaqueCiphersuite;

mod database;
mod handlers;
mod opaque;

const SERVER_ADDR: &str = "0.0.0.0:3000";

// TODO: vérifier si ça clone vraiment
// TODO: ARC???
#[derive(Clone)]
pub struct ServerState {
    pub pool: PgPool,
    pub redis: ConnectionManager,
    pub opaque_setup: Arc<ServerSetup<OpaqueCiphersuite>>,
    pub pepper: [u8; 64], // TODO: Je sais pas si c'est une bonne pratique d'avoir le pepper en mémoire comme ça,
                          // Ou si on devrait le recup du .env à chaque fois..?
}

async fn build_redis(redis_url: &str) -> ConnectionManager {
    let client = RedisClient::open(redis_url).expect("invalid redis url");
    client
        .get_connection_manager()
        .await
        .expect("cannot connect to redis")
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // Load .env
    dotenv().ok();

    // TODO FAIRE UNE FONCTION SETUP PROPRE POUR CHACUN :

    // Setup database connection pool
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Setup redis connection pool
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let redis = build_redis(&redis_url).await;

    // Setup pepper
    let pepper = hex::decode(std::env::var("SERVER_PEPPER").expect("SERVER_PEPPER must be set"))
        .expect("SERVER_PEPPER invalid hex");

    // init état du serveur
    let server_state = ServerState {
        pool,
        redis,
        opaque_setup: Arc::new(opaque::make_server_setup()),
        pepper: pepper.try_into().expect("SERVER_PEPPER must be 64 bytes"),
    };

    let app = Router::new()
        .route("/", get(handlers::root))
        .route("/register/start", post(handlers::auth::register_start))
        .route("/register/finish", post(handlers::auth::register_finish))
        .route("/login/start", post(handlers::auth::login_start))
        .route("/login/finish", post(handlers::auth::login_finish))
        .with_state(server_state);

    let listener = tokio::net::TcpListener::bind(SERVER_ADDR).await.unwrap();
    println!("🚀 Listening on {}", SERVER_ADDR);
    let _ = axum::serve(listener, app).await;
}
