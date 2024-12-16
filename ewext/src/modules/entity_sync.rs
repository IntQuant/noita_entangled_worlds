use std::num::NonZero;

use eyre::Context;
use noita_api::EntityID;

use super::Module;

struct TrackedEntity {
    /// 64 bit globally unique id. Assigned randomly, should only have 50% chance of collision with 2^32 entities at once.
    gid: u64,
    entity: EntityID,
}

pub(crate) struct EntitySync {
    /// Last entity we stopped looking for new tracked entities at.
    look_current_entity: EntityID,
    /// List of entities that we have authority over.
    tracked: Vec<TrackedEntity>,
}

impl Default for EntitySync {
    fn default() -> Self {
        Self {
            look_current_entity: EntityID(NonZero::new(1).unwrap()),
            tracked: Vec::new(),
        }
    }
}

impl EntitySync {
    fn should_be_tracked(&mut self, entity: EntityID) -> eyre::Result<bool> {
        Ok(entity.has_tag("enemy"))
    }

    /// Looks for newly spawned entities that might need to be tracked.
    fn look_for_tracked(&mut self) -> eyre::Result<()> {
        let max_entity = EntityID::max_in_use()?;
        for i in (self.look_current_entity.next()?)..=max_entity {
            let entity = EntityID::try_from(i)?;
            if !entity.is_alive() {
                continue;
            }
            if self.should_be_tracked(entity)? {
                println!("Tracking {entity:?}");
                self.tracked.push(TrackedEntity {
                    gid: rand::random(),
                    entity,
                });
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
