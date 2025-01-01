use bimap::BiHashMap;
use eyre::Ok;
use noita_api::EntityID;
use shared::PeerId;

use crate::net::NetManager;

pub(crate) mod entity_sync;

pub(crate) struct ModuleCtx<'a> {
    pub(crate) net: &'a mut NetManager,
    pub(crate) player_map: &'a mut BiHashMap<PeerId, EntityID>,
}

pub(crate) trait Module {
    // fn init() -> Self;
    fn on_world_init(&mut self, _ctx: &mut ModuleCtx) -> eyre::Result<()> {
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
