use base64::Engine;
use dotenv::dotenv;
use opaque_ke::ServerSetup;
use redis::Client as RedisClient;
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

use secrecy::SecretSlice;

use crate::opaque::DefaultCipherSuite as OpaqueCiphersuite;

#[derive(Clone)]
pub struct ServerState {
    pub pool: PgPool,
    pub redis: ConnectionManager,
    pub opaque_setup: Arc<ServerSetup<OpaqueCiphersuite>>,
    pub pepper: Arc<SecretSlice<u8>>,
}

pub async fn init_server_state() -> ServerState {
    dotenv().ok();

    let pool = setup_postgres().await;

    let redis = setup_redis().await;

    let pepper = setup_pepper();

    let opaque_setup = setup_opaque();

    ServerState {
        pool,
        redis,
        opaque_setup,
        pepper,
    }
}

async fn setup_postgres() -> PgPool {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    pool
}

async fn setup_redis() -> ConnectionManager {
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

    let client = RedisClient::open(redis_url).expect("invalid redis url");

    client
        .get_connection_manager()
        .await
        .expect("cannot connect to redis")
}

fn setup_pepper() -> Arc<SecretSlice<u8>> {
    let pepper_hex = std::env::var("SERVER_PEPPER").expect("SERVER_PEPPER must be set");

    let pepper_bytes = hex::decode(pepper_hex).expect("SERVER_PEPPER invalid hex");
    if pepper_bytes.len() != 64 {
        panic!("SERVER_PEPPER must be 64 bytes");
    }

    Arc::new(pepper_bytes.into())
}

fn setup_opaque() -> Arc<ServerSetup<OpaqueCiphersuite>> {
    let server_setup_b64 =
        std::env::var("OPAQUE_SERVER_SETUP").expect("OPAQUE_SERVER_SETUP must be set");
    let server_setup_bytes = base64::engine::general_purpose::STANDARD
        .decode(server_setup_b64)
        .expect("OPAQUE_SERVER_SETUP invalid base64");
    let server_setup = ServerSetup::<OpaqueCiphersuite>::deserialize(&server_setup_bytes)
        .expect("Failed to deserialize OPAQUE_SERVER_SETUP");
    Arc::new(server_setup)
}
