use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{broadcast, RwLock};

use geometrical_annihilation::net::accept_loop;
use geometrical_annihilation::tick::spawn_tick_loop;
use geometrical_annihilation::world::GameState;

const BIND_ADDR: &str = "127.0.0.1:8080";
const BROADCAST_CAPACITY: usize = 64;

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();

    let addr: SocketAddr = BIND_ADDR.parse()?;
    let state = Arc::new(RwLock::new(GameState::default()));
    let (tx, _) = broadcast::channel::<Arc<str>>(BROADCAST_CAPACITY);

    spawn_tick_loop(state.clone(), tx.clone());
    accept_loop(addr, state, tx).await
}
