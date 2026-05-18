use rand::Rng;

use crate::protocol::Vec2;
use crate::world::{GameState, WORLD_SIZE};

const MIN_SPAWN_DISTANCE: f32 = 800.0;
const MAX_ATTEMPTS: u32 = 64;

pub fn find_spawn<R: Rng + ?Sized>(state: &GameState, rng: &mut R) -> Vec2 {
    let half = WORLD_SIZE * 0.5;
    let min_sq = MIN_SPAWN_DISTANCE * MIN_SPAWN_DISTANCE;
    let mut best: Option<(Vec2, f32)> = None;
    for _ in 0..MAX_ATTEMPTS {
        let candidate: Vec2 = [
            rng.gen_range(-half..half),
            rng.gen_range(-half..half),
        ];
        let nearest_sq = state
            .players
            .values()
            .map(|p| {
                let dx = p.pos[0] - candidate[0];
                let dy = p.pos[1] - candidate[1];
                dx * dx + dy * dy
            })
            .fold(f32::INFINITY, f32::min);
        if nearest_sq >= min_sq {
            return candidate;
        }
        match best {
            Some((_, b)) if b >= nearest_sq => {}
            _ => best = Some((candidate, nearest_sq)),
        }
    }
    best.map(|(p, _)| p).unwrap_or([0.0, 0.0])
}
