use axum::{
    Router,
    routing::{get, post},
};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;

mod handlers;
mod models;
mod opaque;

const SERVER_ADDR: &str = "0.0.0.0:3000";

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // Load .env
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let app = Router::new()
        .route("/", get(handlers::root))
        .route("/users", post(handlers::create_user))
        .route("/register/start", post(handlers::register_start))
        .route("/register/finish", post(handlers::register_finish))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind(SERVER_ADDR).await.unwrap();
    println!("🚀 Listening on {}", SERVER_ADDR);
    let _ = axum::serve(listener, app).await;
}
