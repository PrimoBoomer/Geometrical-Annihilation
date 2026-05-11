#![forbid(unsafe_code)]

use futures_util::stream::StreamExt;
use log::trace;
use thiserror::Error;
use tokio::net::TcpListener;
use tungstenite::Message;

use crate::protocol::ClientAction;

#[derive(Error, Debug)]
enum Error {
    #[error("Bad protocol received")]
    BadProtocol,
    #[error("Connection: {0}")]
    ConnectionFailed(String),
    #[error("Read: {0}")]
    ReadFailed(String),
}

type Result<T> = std::result::Result<T, crate::Error>;

mod protocol {
    use serde_derive::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub(crate) enum ClientAction {
        Authentication { nickname: String },
    }
}

async fn run() -> Result<()> {
    let listenner = TcpListener::bind("127.0.0.1:8080")
        .await
        .map_err(|err| Error::ConnectionFailed(err.to_string()))?;

    loop {
        if let Ok((socket, addr)) = listenner.accept().await {
            tokio::spawn(async move {
                process_socket(socket, addr).await;
            });
        }
    }
}

async fn process_socket(socket: tokio::net::TcpStream, addr: std::net::SocketAddr) -> Result<()> {
    trace!("Connected: {addr}");
    let websocket = tokio_tungstenite::accept_async(socket)
        .await
        .map_err(|err| Error::ConnectionFailed(err.to_string()))?;
    Ok(process_websocket(websocket, addr).await?)
}

async fn process_websocket(
    websocket: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    addr: std::net::SocketAddr,
) -> Result<()> {
    trace!("Upgraded: {addr}");
    let (_writer, mut reader) = websocket.split();

    loop {
        if let Some(maybe_message) = reader.next().await {
            let message = maybe_message.map_err(|err| Error::ReadFailed(err.to_string()))?;
            process_message(message, addr).await?;
        } else {
            return Err(Error::ReadFailed("Nothing to read".to_string()));
        }
    }
}

async fn process_message(message: tungstenite::Message, addr: std::net::SocketAddr) -> Result<()> {
    trace!("Message: {addr}");
    if let Message::Text(text) = message {
        let client_action: ClientAction =
            serde_json::from_str(text.as_str()).map_err(|_err| Error::BadProtocol)?;
        Ok(())
    } else {
        Err(Error::BadProtocol)
    }
}
