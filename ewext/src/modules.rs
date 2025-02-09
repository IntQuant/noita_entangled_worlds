use bimap::BiHashMap;
use eyre::Ok;
use noita_api::EntityID;
use rustc_hash::{FxHashMap, FxHashSet};
use shared::des::Gid;
use shared::PeerId;

use crate::{my_peer_id, net::NetManager};

pub(crate) mod entity_sync;

pub(crate) struct ModuleCtx<'a> {
    pub(crate) net: &'a mut NetManager,
    pub(crate) player_map: &'a mut BiHashMap<PeerId, EntityID>,
    pub(crate) fps_by_player: &'a mut FxHashMap<PeerId, u8>,
    pub(crate) sync_rate: usize,
    pub(crate) dont_spawn: &'a FxHashSet<Gid>,
}
impl ModuleCtx<'_> {
    pub(crate) fn locate_player_within_except_me(
        &self,
        x: f32,
        y: f32,
        radius: f32,
    ) -> eyre::Result<Option<PeerId>> {
        let mut res = None;
        for (peer, entity) in self.player_map.iter() {
            if *peer == my_peer_id() {
                continue;
            }
            let (ex, ey) = entity.position()?;
            if (x - ex).powi(2) + (y - ey).powi(2) < radius.powi(2) {
                res = Some(*peer);
                break;
            }
        }
        Ok(res)
    }
}

pub(crate) trait Module {
    // fn init() -> Self;
    fn on_world_init(&mut self, _ctx: &mut ModuleCtx) -> eyre::Result<()> {
        Ok(())
    }

    fn on_new_entity(&mut self, _entity: EntityID, _ctx: &mut ModuleCtx) -> eyre::Result<()> {
        Ok(())
    }

    fn on_world_update(&mut self, _ctx: &mut ModuleCtx) -> eyre::Result<()> {
        Ok(())
    }

    #[expect(clippy::too_many_arguments)]
    fn on_projectile_fired(
        &mut self,
        _ctx: &mut ModuleCtx,
        _shooter_id: Option<EntityID>,
        _projectile_id: Option<EntityID>,
        _initial_rng: i32,
        _position: (f32, f32),
        _target: (f32, f32),
        _multicast_index: i32,
    ) -> eyre::Result<()> {
        Ok(())
    }
}
