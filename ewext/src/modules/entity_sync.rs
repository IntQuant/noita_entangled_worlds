use eyre::Context;
use noita_api::EntityID;

use super::Module;

pub(crate) struct EntitySync {
    look_current_entity: isize,
    /// List of entities that we have authority over.
    tracked: Vec<EntityID>,
}

impl Default for EntitySync {
    fn default() -> Self {
        Self {
            look_current_entity: 1,
            tracked: Vec::new(),
        }
    }
}

impl EntitySync {
    /// Looks for newly spawned entities that might need to be tracked.
    fn look_for_tracked(&mut self) -> eyre::Result<()> {
        let max_entity = noita_api::raw::entities_get_max_id()? as isize;
        for i in (self.look_current_entity + 1)..=max_entity {
            let ent = EntityID::try_from(i)?;
            if !ent.is_alive() {
                continue;
            }
        }

        self.look_current_entity = max_entity;
        Ok(())
    }
}

impl Module for EntitySync {
    fn on_world_update(&mut self, _ctx: &mut super::ModuleCtx) -> eyre::Result<()> {
        self.look_for_tracked()
            .wrap_err("Error in look_for_tracked")?;

        Ok(())
    }
}
