use bimap::BiHashMap;
use eyre::Ok;
use noita_api::EntityID;
use rustc_hash::{FxHashMap, FxHashSet};
use shared::des::Gid;
use shared::{PeerId, WorldPos};

use crate::net::NetManager;

pub(crate) mod entity_sync;

pub(crate) struct ModuleCtx<'a> {
    pub(crate) net: &'a mut NetManager,
    pub(crate) player_map: &'a mut BiHashMap<PeerId, EntityID>,
    pub(crate) camera_pos: &'a mut FxHashMap<PeerId, WorldPos>,
    pub(crate) fps_by_player: &'a mut FxHashMap<PeerId, u8>,
    pub(crate) dont_spawn: &'a FxHashSet<Gid>,
}
impl ModuleCtx<'_> {
    pub(crate) fn locate_player_within_except_me(
        &self,
        x: i32,
        y: i32,
        radius: f32,
    ) -> eyre::Result<Option<PeerId>> {
        let mut res = None;
        let r = (radius.abs() as u64).pow(2);
        let mut dist = r;
        for (peer, entity) in self.player_map.iter() {
            if entity.has_tag("ew_client") {
                let (ex, ey) = entity.position()?;
                let (ex, ey) = (ex as i32, ey as i32);
                let pos = self
                    .camera_pos
                    .get(peer)
                    .cloned()
                    .unwrap_or(WorldPos { x: ex, y: ey });
                let (cx, cy) = (pos.x, pos.y);
                let d2 = (cx.abs_diff(ex) as u64).pow(2) + (cy.abs_diff(ey) as u64).pow(2);
                let d = (x.abs_diff(ex) as u64).pow(2) + (y.abs_diff(ey) as u64).pow(2);
                if d < dist && d2 < r {
                    res = Some(*peer);
                    dist = d;
                }
            }
        }
        if res.is_none() {
            for (peer, pos) in self.camera_pos.iter() {
                let (ex, ey) = (pos.x, pos.y);
                let d = (x.abs_diff(ex) as u64).pow(2) + (y.abs_diff(ey) as u64).pow(2);
                if d < dist {
                    res = Some(*peer);
                    dist = d;
                }
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

    fn on_new_entity(&mut self, _entity: EntityID, _kill: bool) -> eyre::Result<()> {
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
