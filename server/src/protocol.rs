use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type Rgb = [u8; 3];
pub type Vec2 = [f32; 2];

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Hello { name: Option<String> },
    Move { target: Vec2 },
    Respawn,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage<'a> {
    Welcome {
        player_id: Uuid,
        primary: Rgb,
        outline: Rgb,
        world: Vec2,
        spawn: Vec2,
    },
    Snapshot {
        tick: u64,
        players: &'a [PlayerSnapshot],
    },
}

#[derive(Debug, Serialize, Clone)]
pub struct PlayerSnapshot {
    pub id: Uuid,
    pub pos: Vec2,
    pub target: Option<Vec2>,
    pub primary: Rgb,
    pub outline: Rgb,
}
