use base64::Engine;
use dotenv::dotenv;
use opaque_ke::ServerSetup;
use redis::Client as RedisClient;
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

use secrecy::SecretSlice;

use crate::handler::auth::opaque::DefaultCipherSuite as OpaqueCiphersuite;

#[derive(Clone)]
pub struct ServerState {
    pool: PgPool,
    redis: ConnectionManager,
    opaque_setup: Arc<ServerSetup<OpaqueCiphersuite>>,
    pepper: Arc<SecretSlice<u8>>,
    jwt_key: Arc<SecretSlice<u8>>,
}

impl ServerState {
    // Constructor
    pub async fn new() -> Self {
        dotenv().ok();

        let pool = Self::init_postgres().await;
        let redis = Self::init_redis().await;
        let opaque_setup = Self::init_opaque();
        let pepper = Self::init_pepper();
        let jwt_key = Self::init_jwt_key();

        ServerState {
            pool,
            redis,
            opaque_setup,
            pepper,
            jwt_key,
        }
    }

    // Getter
    pub fn pool(&self) -> PgPool {
        self.pool.clone()
    }
    pub fn redis(&self) -> ConnectionManager {
        self.redis.clone()
    }
    pub fn opaque_setup(&self) -> Arc<ServerSetup<OpaqueCiphersuite>> {
        self.opaque_setup.clone()
    }
    pub fn pepper(&self) -> Arc<SecretSlice<u8>> {
        self.pepper.clone()
    }
    pub fn jwt_key(&self) -> Arc<SecretSlice<u8>> {
        self.jwt_key.clone()
    }

    // Init
    async fn init_postgres() -> PgPool {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database")
    }

    async fn init_redis() -> ConnectionManager {
        let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

        let client = RedisClient::open(redis_url).expect("invalid redis url");

        client
            .get_connection_manager()
            .await
            .expect("cannot connect to redis")
    }

    fn init_opaque() -> Arc<ServerSetup<OpaqueCiphersuite>> {
        let server_setup_b64 =
            std::env::var("OPAQUE_SERVER_SETUP").expect("OPAQUE_SERVER_SETUP must be set");

        let server_setup_bytes = base64::engine::general_purpose::STANDARD
            .decode(server_setup_b64)
            .expect("OPAQUE_SERVER_SETUP invalid base64");

        let server_setup = ServerSetup::<OpaqueCiphersuite>::deserialize(&server_setup_bytes)
            .expect("Failed to deserialize OPAQUE_SERVER_SETUP");

        Arc::new(server_setup)
    }

    fn init_pepper() -> Arc<SecretSlice<u8>> {
        let pepper_hex = std::env::var("SERVER_PEPPER").expect("SERVER_PEPPER must be set");

        let pepper_bytes = hex::decode(pepper_hex).expect("SERVER_PEPPER invalid hex");

        if pepper_bytes.len() != 64 {
            panic!("SERVER_PEPPER must be 64 bytes");
        }

        Arc::new(pepper_bytes.into())
    }

    fn init_jwt_key() -> Arc<SecretSlice<u8>> {
        let jwt_key = std::env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY must be set");

        let jwt_key_bytes = hex::decode(jwt_key).expect("SERVER_PEPPER invalid hex");

        match jwt_key_bytes.len() {
            32 | 48 | 64 => {}
            _ => panic!("JWT_SECRET_KEY must be 32, 48, or 64 bytes"),
        }

        Arc::new(jwt_key_bytes.into())
    }
}
