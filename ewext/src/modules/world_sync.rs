use crate::WorldSync;
use crate::modules::{Module, ModuleCtx};
use noita_api::noita::world::ParticleWorldState;
use shared::world_sync::ProxyToWorldSync;
impl Module for WorldSync {
    fn on_world_update(&mut self, _ctx: &mut ModuleCtx) -> eyre::Result<()> {
        std::hint::black_box(unsafe {
            self.particle_world_state.assume_init_ref().encode_world()
        })?;
        std::hint::black_box(unsafe {
            self.particle_world_state.assume_init_ref().decode_world()
        })?;
        //TODO
        Ok(())
    }
}
impl WorldSync {
    pub fn handle_remote(&mut self, msg: ProxyToWorldSync) -> eyre::Result<()> {
        match msg {
            ProxyToWorldSync::Updates(updates) => {
                for _chunk in updates {
                    //TODO
                }
            }
        }
        Ok(())
    }
}
trait WorldData {
    unsafe fn encode_world(&self) -> eyre::Result<()>;
    unsafe fn decode_world(&self) -> eyre::Result<()>;
}
impl WorldData for ParticleWorldState {
    unsafe fn encode_world(&self) -> eyre::Result<()> {
        //TODO
        Ok(())
    }
    unsafe fn decode_world(&self) -> eyre::Result<()> {
        //TODO
        Ok(())
    }
}
