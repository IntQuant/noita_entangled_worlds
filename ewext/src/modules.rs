use eyre::Ok;

use crate::net::NetManager;

pub(crate) mod entity_sync;

pub(crate) struct ModuleCtx<'a> {
    pub(crate) net: &'a mut NetManager,
}

pub(crate) trait Module {
    // fn init() -> Self;
    fn on_world_init(&mut self, _ctx: &mut ModuleCtx) -> eyre::Result<()> {
        Ok(())
    }

    fn on_world_update(&mut self, _ctx: &mut ModuleCtx) -> eyre::Result<()> {
        Ok(())
    }
}
