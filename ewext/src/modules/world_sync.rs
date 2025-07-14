use crate::WorldSync;
use crate::modules::{Module, ModuleCtx};
use noita_api::noita::world::ParticleWorldState;
use shared::NoitaOutbound;
use shared::world_sync::{ChunkCoord, NoitaWorldUpdate, ProxyToWorldSync, WorldSyncToProxy};
impl Module for WorldSync {
    fn on_world_update(&mut self, ctx: &mut ModuleCtx) -> eyre::Result<()> {
        let mut update = NoitaWorldUpdate {
            coord: ChunkCoord(0, 0),
            runs: vec![],
        };
        unsafe {
            self.particle_world_state
                .assume_init_ref()
                .encode_world(ChunkCoord(0, 0), &mut update)?
        };
        let msg = NoitaOutbound::WorldSyncToProxy(WorldSyncToProxy::Updates(vec![update]));
        ctx.net.send(&msg)?;
        Ok(())
    }
}
impl WorldSync {
    pub fn handle_remote(&mut self, msg: ProxyToWorldSync) -> eyre::Result<()> {
        match msg {
            ProxyToWorldSync::Updates(updates) => {
                for chunk in updates {
                    unsafe {
                        self.particle_world_state
                            .assume_init_ref()
                            .decode_world(chunk)?
                    }
                }
            }
        }
        Ok(())
    }
}
trait WorldData {
    unsafe fn encode_world(
        &self,
        coord: ChunkCoord,
        chunk: &mut NoitaWorldUpdate,
    ) -> eyre::Result<()>;
    unsafe fn decode_world(&self, chunk: NoitaWorldUpdate) -> eyre::Result<()>;
}
impl WorldData for ParticleWorldState {
    unsafe fn encode_world(
        &self,
        coord: ChunkCoord,
        chunk: &mut NoitaWorldUpdate,
    ) -> eyre::Result<()> {
        chunk.coord = coord;
        let runs = &mut chunk.runs;
        runs.clear();
        Ok(())
    }
    unsafe fn decode_world(&self, chunk: NoitaWorldUpdate) -> eyre::Result<()> {
        std::hint::black_box(chunk);
        //TODO
        Ok(())
    }
}
