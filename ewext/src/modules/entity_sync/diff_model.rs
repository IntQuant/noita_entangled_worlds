use bimap::BiHashMap;
use noita_api::{
    AIAttackComponent, AdvancedFishAIComponent, AnimalAIComponent, CameraBoundComponent, EntityID,
    PhysicsAIComponent,
};
use rustc_hash::FxHashMap;
use shared::des::{EntityData, EntityUpdate, Gid};

struct EntityEntry {
    entity_data: EntityData,
    x: f32,
    y: f32,
}

#[derive(Default)]
pub(crate) struct LocalDiffModel {}

#[derive(Default)]
pub(crate) struct RemoteDiffModel {
    tracked: BiHashMap<Gid, EntityID>,
    entity_entries: FxHashMap<Gid, EntityEntry>,
}

impl EntityEntry {
    fn new(entity_data: EntityData) -> Self {
        Self {
            entity_data,
            x: 0.0,
            y: 0.0,
        }
    }
}

impl LocalDiffModel {
    pub(crate) fn reset_diff_encoding(&mut self) {
        todo!();
    }

    pub(crate) fn make_diff(&mut self) -> Vec<EntityUpdate> {
        todo!()
    }
}

impl RemoteDiffModel {
    pub(crate) fn apply_diff(&mut self, diff: &[EntityUpdate]) {
        let mut current_gid = 0;
        for entry in diff {
            match entry {
                EntityUpdate::CurrentEntity(gid) => current_gid = *gid,
                EntityUpdate::EntityData(entity_data) => {
                    self.entity_entries
                        .insert(current_gid, EntityEntry::new(entity_data.clone()));
                }
                EntityUpdate::SetPosition(x, y) => {
                    let Some(ent_data) = self.entity_entries.get_mut(&current_gid) else {
                        continue;
                    };
                    ent_data.x = *x;
                    ent_data.y = *y;
                }
                EntityUpdate::RemoveEntity(gid) => {
                    if let Some((_, entity)) = self.tracked.remove_by_left(gid) {
                        entity.kill();
                    }
                    self.entity_entries.remove(&gid);
                }
            }
        }
    }

    pub(crate) fn apply_entities(&mut self) -> eyre::Result<()> {
        for (gid, entity_entry) in &self.entity_entries {
            match self.tracked.get_by_left(gid) {
                Some(entity) => {
                    entity.set_position(entity_entry.x, entity_entry.y)?;
                }
                None => {
                    let entity = self.spawn_entity_by_data(&entity_entry.entity_data)?;
                    self.remove_unnecessary_components(entity)?;
                    self.tracked.insert(*gid, entity);
                }
            }
        }

        Ok(())
    }

    fn spawn_entity_by_data(&self, entity_data: &EntityData) -> eyre::Result<EntityID> {
        match entity_data {
            EntityData::Filename(filename) => EntityID::load(filename, None, None),
        }
    }

    /// Removes components that shouldn't be on entities that were replicated from a remote,
    /// generally because they interfere with things we're supposed to sync.
    fn remove_unnecessary_components(&self, entity: EntityID) -> eyre::Result<()> {
        entity.remove_all_components_of_type::<CameraBoundComponent>()?;
        entity.remove_all_components_of_type::<AnimalAIComponent>()?;
        entity.remove_all_components_of_type::<PhysicsAIComponent>()?;
        entity.remove_all_components_of_type::<AdvancedFishAIComponent>()?;
        entity.remove_all_components_of_type::<AIAttackComponent>()?;
        Ok(())
    }
}

impl Drop for RemoteDiffModel {
    fn drop(&mut self) {
        // Cleanup all entities tracked by this model.
        for ent in self.tracked.right_values() {
            ent.kill();
        }
    }
}
