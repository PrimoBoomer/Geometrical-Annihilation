use std::collections::HashMap;

use uuid::Uuid;

use crate::protocol::{PlayerSnapshot, Rgb, Vec2};

pub const WORLD_SIZE: f32 = 10_000.0;
pub const COMMANDER_SPEED: f32 = 200.0;
pub const TICK_DT: f32 = 0.1;
pub const ARRIVAL_EPS: f32 = 1.0;

#[derive(Debug, Clone)]
pub struct Player {
    pub id: Uuid,
    pub primary: Rgb,
    pub outline: Rgb,
    pub pos: Vec2,
    pub target: Option<Vec2>,
}

#[derive(Debug, Default)]
pub struct GameState {
    pub players: HashMap<Uuid, Player>,
    pub tick: u64,
}

impl GameState {
    pub fn advance(&mut self) {
        let step = COMMANDER_SPEED * TICK_DT;
        for player in self.players.values_mut() {
            let Some(target) = player.target else { continue };
            let dx = target[0] - player.pos[0];
            let dy = target[1] - player.pos[1];
            let dist = (dx * dx + dy * dy).sqrt();
            if dist <= step.max(ARRIVAL_EPS) {
                player.pos = target;
                player.target = None;
            } else {
                let inv = step / dist;
                player.pos[0] += dx * inv;
                player.pos[1] += dy * inv;
            }
        }
        self.tick = self.tick.wrapping_add(1);
    }

    pub fn snapshot(&self) -> Vec<PlayerSnapshot> {
        self.players
            .values()
            .map(|p| PlayerSnapshot {
                id: p.id,
                pos: p.pos,
                target: p.target,
                primary: p.primary,
                outline: p.outline,
            })
            .collect()
    }

    pub fn clamp_to_world(p: Vec2) -> Vec2 {
        let half = WORLD_SIZE * 0.5;
        [p[0].clamp(-half, half), p[1].clamp(-half, half)]
    }
}
