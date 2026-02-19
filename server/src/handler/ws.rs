use axum::{
    extract::{
        Extension, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::config::server::ServerState;
use crate::handler::jwt::Claims;

pub async fn establish_conn(
    State(state): State<ServerState>,
    Extension(claims): Extension<Claims>,
    ws: WebSocketUpgrade,
) -> Result<Response, StatusCode> {
    // retrieve the device_id in the JWT
    let device_id = Uuid::parse_str(claims.sub()).map_err(|_| StatusCode::BAD_REQUEST)?;
    println!("WS connected for device_id={device_id}");
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, device_id)))
}

async fn handle_socket(socket: WebSocket, state: ServerState, device_id: Uuid) {
    let (mut ws_tx, mut ws_rx) = socket.split();

    // Internal channel to push msg to this socket
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // Link the channel to the device_id in the Dashmap
    state.sockets.insert(device_id.to_string(), tx);

    // task writer (server -> client)
    let writer = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_tx.send(msg).await.is_err() {
                break;
            }
        }
    });

    // reader (client -> server)
    while let Some(Ok(msg)) = ws_rx.next().await {
        match msg {
            Message::Text(_txt) => {
                // handle si besoin (ping custom, ack, etc.)
            }
            Message::Binary(_bin) => {
                // handle si besoin
            }
            Message::Close(_) => {
                println!("WS closed for device_id={device_id}");
                break;
            }
            _ => {}
        }
    }

    // cleanup
    state.sockets.remove(&device_id.to_string());
    writer.abort();
}
