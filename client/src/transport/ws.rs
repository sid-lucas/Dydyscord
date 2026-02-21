use futures_util::{SinkExt, StreamExt};
use http::header::COOKIE;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::config::constant;
use crate::transport::api;

pub fn start_background() {
    let session_cookie = match api::session_cookie_header() {
        Some(c) => c,
        None => {
            eprintln!("WS: no session cookie, not connecting"); // TODO Handle better
            return;
        }
    };

    let ws_url = ws_url_from_http(constant::SERVER_URL);

    // TODO : REprendre ici
    let res = connect_ws(ws_url.clone(), session_cookie.clone());
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        match rt.block_on(res) {
            Ok(_) => println!("WS ended cleanly"),
            Err(e) => eprintln!("WS error: {e}"),
        }
    });
}

fn ws_url_from_http(http_url: &str) -> String {
    let base = http_url.trim_end_matches('/');
    let ws_base = if base.starts_with("https://") {
        base.replacen("https://", "wss://", 1)
    } else {
        base.replacen("http://", "ws://", 1)
    };
    format!("{}/ws", ws_base)
}

pub async fn connect_ws(
    ws_url: String,
    session_cookie: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // Build request with cookie header
    let mut req = ws_url.into_client_request()?;
    req.headers_mut().insert(COOKIE, session_cookie.parse()?);

    let (ws_stream, resp) = connect_async(req).await?;

    println!("WS status: {}", resp.status());

    let (mut ws_tx, mut ws_rx) = ws_stream.split();

    // Optionnel: envoyer un "hello"
    ws_tx.send(Message::Text("hello".into())).await?;

    // Boucle de réception
    while let Some(msg) = ws_rx.next().await {
        match msg? {
            Message::Text(txt) => {
                // ex: {"type":"welcome_ready"}
                // si welcome_ready -> fetch_welcome()
                // Fetch welcome for current session
                println!("Salut a tous");
            }
            Message::Binary(bin) => {
                // ex: welcome OpenMLS bytes
                // appliquer directement
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    Ok(())
}
