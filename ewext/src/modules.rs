use eyre::Ok;

pub(crate) mod entity_sync;

pub(crate) struct ModuleCtx {}

pub(crate) trait Module {
    // fn init() -> Self;
    fn on_world_update(&mut self, _ctx: &mut ModuleCtx) -> eyre::Result<()> {
        Ok(())
    }
}
