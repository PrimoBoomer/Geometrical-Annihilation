#![forbid(unsafe_code)]

use futures_util::stream::StreamExt;
use log::trace;
use tokio::net::TcpListener;
mod protocol {
    enum PlayerAction {
        Authentication { nickname: String },
    }
}

async fn run() -> anyhow::Result<()> {
    let listenner = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        if let Ok((socket, addr)) = listenner.accept().await {
            tokio::spawn(async move {
                process_socket(socket, addr).await;
            });
        }
    }
}

async fn process_socket(socket: tokio::net::TcpStream, addr: std::net::SocketAddr) {
    trace!("Connection from {addr}");
    if let Ok(websocket) = tokio_tungstenite::accept_async(socket).await {
        process_websocket(websocket, addr).await;
    } else {
        return;
    }
}

async fn process_websocket(
    websocket: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    addr: std::net::SocketAddr,
) {
    let (_writer, mut reader) = websocket.split();

    loop {
        if let Some(Ok(message)) = reader.next().await {
            process_message(message).await;
        } else {
            return;
        }
    }
}

async fn process_message(message: tungstenite::Message) {
    if let Message::Text(text) {

    } else {
        return;
    }
}
