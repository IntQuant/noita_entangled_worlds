use bimap::BiHashMap;
use eyre::OptionExt;
use noita_api::{
    game_print, AIAttackComponent, AdvancedFishAIComponent, AnimalAIComponent,
    CameraBoundComponent, EntityID, PhysicsAIComponent,
};
use rustc_hash::FxHashMap;
use shared::des::{EntityData, EntityEntry, EntityUpdate, Gid, Lid};

pub(crate) static DES_TAG: &str = "ew_des";

struct EntityEntryPair {
    last: Option<EntityEntry>,
    current: EntityEntry,
}

pub(crate) struct LocalDiffModel {
    next_lid: Lid,
    tracked: BiHashMap<Lid, EntityID>,
    entity_entries: FxHashMap<Lid, EntityEntryPair>,
    pending_removal: Vec<Lid>,
}

#[derive(Default)]
pub(crate) struct RemoteDiffModel {
    tracked: BiHashMap<Lid, EntityID>,
    entity_entries: FxHashMap<Lid, EntityEntry>,
}

impl Default for LocalDiffModel {
    fn default() -> Self {
        Self {
            next_lid: Lid(0),
            tracked: Default::default(),
            entity_entries: Default::default(),
            pending_removal: Vec::with_capacity(16),
        }
    }
}

impl LocalDiffModel {
    fn alloc_lid(&mut self) -> Lid {
        let ret = self.next_lid;
        self.next_lid.0 += 1;
        ret
    }

    pub(crate) fn track_entity(&mut self, entity: EntityID) -> eyre::Result<()> {
        let lid = self.alloc_lid();
        entity.remove_all_components_of_type::<CameraBoundComponent>()?;
        entity.add_tag(DES_TAG)?;

        self.tracked.insert(lid, entity);
        // TODO: handle other types of entities, like items.
        let filename = entity.filename()?;
        let (x, y) = entity.position()?;
        self.entity_entries.insert(
            lid,
            EntityEntryPair {
                last: None,
                current: EntityEntry {
                    entity_data: EntityData::Filename(filename),
                    x,
                    y,
                },
            },
        );
        Ok(())
    }

    pub(crate) fn reset_diff_encoding(&mut self) {
        for (_, entry_pair) in &mut self.entity_entries {
            entry_pair.last = None;
        }
    }

    pub(crate) fn update_tracked_entities(&mut self) -> eyre::Result<()> {
        for (&lid, EntityEntryPair { last: _, current }) in &mut self.entity_entries {
            let entity = self
                .tracked
                .get_by_left(&lid)
                .ok_or_eyre("Expected to find a corresponding entity")?;
            if !entity.is_alive() {
                self.pending_removal.push(lid);
                continue;
            }
            let (x, y) = entity.position()?;
            current.x = x;
            current.y = y;
        }
        Ok(())
    }

    pub(crate) fn make_diff(&mut self) -> Vec<EntityUpdate> {
        let mut res = Vec::new();
        for (&lid, EntityEntryPair { last, current }) in &mut self.entity_entries {
            res.push(EntityUpdate::CurrentEntity(lid));
            let Some(last) = last.as_mut() else {
                res.push(EntityUpdate::Init(current.clone()));
                *last = Some(current.clone());
                continue;
            };
            let mut had_any_delta = false;
            if current.x != last.x || current.y != last.y {
                res.push(EntityUpdate::SetPosition(current.x, current.y));
                last.x = current.x;
                last.y = current.y;
                had_any_delta = true;
            }

            // Remove the CurrentEntity thing because it's not necessary.
            if !had_any_delta {
                res.pop();
            }
        }
        for lid in self.pending_removal.drain(..) {
            res.push(EntityUpdate::RemoveEntity(lid));
            // "Untrack" entity
            self.tracked.remove_by_left(&lid);
            self.entity_entries.remove(&lid);
        }
        res
    }
}

impl RemoteDiffModel {
    pub(crate) fn apply_diff(&mut self, diff: &[EntityUpdate]) {
        let mut current_lid = Lid(0);
        for entry in diff {
            match entry {
                EntityUpdate::CurrentEntity(lid) => current_lid = *lid,
                EntityUpdate::Init(entity_entry) => {
                    self.entity_entries
                        .insert(current_lid, entity_entry.clone());
                }
                EntityUpdate::SetPosition(x, y) => {
                    let Some(ent_data) = self.entity_entries.get_mut(&current_lid) else {
                        continue;
                    };
                    ent_data.x = *x;
                    ent_data.y = *y;
                }
                EntityUpdate::RemoveEntity(lid) => {
                    if let Some((_, entity)) = self.tracked.remove_by_left(lid) {
                        entity.kill();
                    }
                    self.entity_entries.remove(&lid);
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
                    game_print("Spawned remote entity");
                    self.remove_unnecessary_components(entity)?;
                    entity.add_tag(DES_TAG)?;
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
