use std::sync::{Arc, Mutex};

use crate::net::omni::OmniPeerId;
use rustc_hash::FxHashMap;
use shared::WorldPos;

#[derive(Default)]
struct WorldInfoInner {
    players: FxHashMap<OmniPeerId, WorldPos>,
}

#[derive(Default)]
pub struct WorldInfo {
    inner: Arc<Mutex<WorldInfoInner>>,
}
impl WorldInfo {
    pub(crate) fn clear_positions(&self) {
        self.with_inner(|inner| inner.players.clear())
    }

    pub(crate) fn dist(&self, from: OmniPeerId, to: OmniPeerId) -> Option<(u64, f32)> {
        self.with_inner(|inner| {
            inner
                .players
                .get(&from)
                .and_then(|f| inner.players.get(&to).map(|t| f.dist(t)))
        })
    }

    fn with_inner<T>(&self, f: impl FnOnce(&mut WorldInfoInner) -> T) -> T {
        let mut inner = self.inner.lock().unwrap();
        f(&mut inner)
    }

    pub(in crate::net) fn update_player_pos(&self, peer_id: OmniPeerId, x: i32, y: i32) {
        self.with_inner(|inner| {
            let info = inner.players.entry(peer_id).or_default();
            info.x = x;
            info.y = y;
        })
    }

    pub fn with_player_infos(&self, mut f: impl FnMut(OmniPeerId, WorldPos)) {
        self.with_inner(|inner| {
            for (id, info) in &inner.players {
                f(*id, *info)
            }
        })
    }
}
