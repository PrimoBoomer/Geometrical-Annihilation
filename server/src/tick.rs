use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{broadcast, RwLock};
use tokio::time::interval;

use crate::protocol::ServerMessage;
use crate::world::{GameState, TICK_DT};

pub fn spawn_tick_loop(state: Arc<RwLock<GameState>>, tx: broadcast::Sender<Arc<str>>) {
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs_f32(TICK_DT));
        log::info!("tick loop started ({} Hz)", (1.0 / TICK_DT).round());
        loop {
            ticker.tick().await;
            let payload: Option<Arc<str>> = {
                let mut s = state.write().await;
                s.advance();
                let players = s.snapshot();
                let msg = ServerMessage::Snapshot {
                    tick: s.tick,
                    players: &players,
                };
                match serde_json::to_string(&msg) {
                    Ok(j) => Some(Arc::<str>::from(j)),
                    Err(e) => {
                        log::error!("snapshot serialize failed: {e}");
                        None
                    }
                }
            };
            if let Some(payload) = payload {
                let _ = tx.send(payload);
            }
        }
    });
}
