use std::sync::{Arc, Mutex};

use rustc_hash::FxHashMap;

use crate::net::omni::OmniPeerId;

use super::world_model::ChunkCoord;

#[derive(Default, Clone, Copy)]
pub struct PlayerInfo {
    pub x: f64,
    pub y: f64,
}

#[derive(Default)]
struct WorldInfoInner {
    players: FxHashMap<OmniPeerId, PlayerInfo>,
}

#[derive(Default)]
pub struct WorldInfo {
    inner: Arc<Mutex<WorldInfoInner>>,
}

impl WorldInfo {
    fn with_inner<T>(&self, f: impl FnOnce(&mut WorldInfoInner) -> T) -> T {
        let mut inner = self.inner.lock().unwrap();
        f(&mut inner)
    }

    pub(in crate::net) fn update_player_pos(&self, peer_id: OmniPeerId, x: f64, y: f64) {
        self.with_inner(|inner| {
            let info = inner.players.entry(peer_id).or_default();
            info.x = x;
            info.y = y;
        })
    }

    pub fn with_player_infos(&self, mut f: impl FnMut(OmniPeerId, PlayerInfo)) {
        self.with_inner(|inner| {
            for (id, info) in &inner.players {
                f(*id, *info)
            }
        })
    }
}
