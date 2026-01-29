use axum::{
    routing::get,
    Router,
};

use tokio::{
    net::TcpListener,
};

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Test" }));

    // run our app with hyper, listening globally on port 3000
    let listener = TcpListener::bind("localhost:2727").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}