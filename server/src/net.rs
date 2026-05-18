use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use rand::thread_rng;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, RwLock};
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

use crate::color::random_pair;
use crate::protocol::{ClientMessage, ServerMessage};
use crate::spawn::find_spawn;
use crate::world::{GameState, Player, WORLD_SIZE};

pub async fn accept_loop(
    addr: SocketAddr,
    state: Arc<RwLock<GameState>>,
    tx: broadcast::Sender<Arc<str>>,
) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    log::info!("listening on {addr}");
    loop {
        let (stream, peer) = listener.accept().await?;
        let state = state.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, peer, state, tx).await {
                log::warn!("connection {peer} closed: {e}");
            }
        });
    }
}

async fn handle_connection(
    stream: TcpStream,
    peer: SocketAddr,
    state: Arc<RwLock<GameState>>,
    tx: broadcast::Sender<Arc<str>>,
) -> Result<()> {
    let ws = tokio_tungstenite::accept_async(stream).await?;
    log::info!("ws upgraded for {peer}");
    let (mut sink, mut source) = ws.split();

    let player_id = Uuid::new_v4();
    let primary;
    let outline;
    let spawn_pos;
    {
        let mut s = state.write().await;
        let mut rng = thread_rng();
        let pair = random_pair(&mut rng);
        primary = pair.0;
        outline = pair.1;
        spawn_pos = find_spawn(&s, &mut rng);
        s.players.insert(
            player_id,
            Player {
                id: player_id,
                primary,
                outline,
                pos: spawn_pos,
                target: None,
            },
        );
    }
    log::info!("player {player_id} joined from {peer} at {spawn_pos:?}");

    let welcome = ServerMessage::Welcome {
        player_id,
        primary,
        outline,
        world: [WORLD_SIZE, WORLD_SIZE],
        spawn: spawn_pos,
    };
    sink.send(Message::Text(serde_json::to_string(&welcome)?.into()))
        .await?;

    let mut rx = tx.subscribe();
    let send_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(payload) => {
                    let text: String = payload.as_ref().to_owned();
                    if sink.send(Message::Text(text.into())).await.is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    log::warn!("subscriber lagged {n} messages");
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
        let _ = sink.close().await;
    });

    let read_result: Result<()> = async {
        while let Some(msg) = source.next().await {
            let msg = msg?;
            match msg {
                Message::Text(t) => {
                    let txt: &str = t.as_ref();
                    match serde_json::from_str::<ClientMessage>(txt) {
                        Ok(ClientMessage::Hello { .. }) => {}
                        Ok(ClientMessage::Move { target }) => {
                            let target = GameState::clamp_to_world(target);
                            let mut s = state.write().await;
                            if let Some(p) = s.players.get_mut(&player_id) {
                                p.target = Some(target);
                            }
                        }
                        Ok(ClientMessage::Respawn) => {
                            let mut s = state.write().await;
                            let mut rng = thread_rng();
                            let pos = find_spawn(&s, &mut rng);
                            drop(rng);
                            if let Some(p) = s.players.get_mut(&player_id) {
                                p.pos = pos;
                                p.target = None;
                            }
                        }
                        Err(e) => log::warn!("bad message from {peer}: {e}"),
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
        Ok(())
    }
    .await;

    {
        let mut s = state.write().await;
        s.players.remove(&player_id);
    }
    send_task.abort();
    log::info!("player {player_id} ({peer}) left");
    read_result
}
