use bimap::BiHashMap;
use eyre::OptionExt;
use noita_api::{
    game_print, lua::LuaState, AIAttackComponent, AdvancedFishAIComponent, AnimalAIComponent,
    CameraBoundComponent, CharacterDataComponent, EntityID, PhysicsAIComponent, VelocityComponent,
};
use rustc_hash::FxHashMap;
use shared::des::{EntityInfo, EntitySpawnInfo, EntityUpdate, Gid, Lid, ProjectileFired};

use crate::serialize::deserialize_entity;

pub(crate) static DES_TAG: &str = "ew_des";

struct EntityEntryPair {
    last: Option<EntityInfo>,
    current: EntityInfo,
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
    entity_infos: FxHashMap<Lid, EntityInfo>,
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
                current: EntityInfo {
                    entity_data: EntitySpawnInfo::Filename(filename),
                    x,
                    y,
                    vx: 0.0,
                    vy: 0.0,
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
            if let Some(vel) = entity.try_get_first_component::<VelocityComponent>(None)? {
                let (vx, vy) = vel.m_velocity()?;
                current.vx = vx;
                current.vy = vy;
            }
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

            if current.vx != last.vx || current.vy != last.vy {
                res.push(EntityUpdate::SetVelocity(current.vx, current.vy));
                last.vx = current.vx;
                last.vy = current.vy;
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

    pub(crate) fn lid_by_entity(&self, entity: EntityID) -> Option<Lid> {
        self.tracked.get_by_right(&entity).copied()
    }
}

impl RemoteDiffModel {
    pub(crate) fn apply_diff(&mut self, diff: &[EntityUpdate]) {
        let mut current_lid = Lid(0);
        for entry in diff {
            match entry {
                EntityUpdate::CurrentEntity(lid) => current_lid = *lid,
                EntityUpdate::Init(entity_entry) => {
                    self.entity_infos.insert(current_lid, entity_entry.clone());
                }
                EntityUpdate::SetPosition(x, y) => {
                    let Some(ent_data) = self.entity_infos.get_mut(&current_lid) else {
                        continue;
                    };
                    ent_data.x = *x;
                    ent_data.y = *y;
                }
                EntityUpdate::SetVelocity(vx, vy) => {
                    let Some(entity_info) = self.entity_infos.get_mut(&current_lid) else {
                        continue;
                    };
                    entity_info.vx = *vx;
                    entity_info.vy = *vy;
                }
                EntityUpdate::RemoveEntity(lid) => {
                    if let Some((_, entity)) = self.tracked.remove_by_left(lid) {
                        entity.kill();
                    }
                    self.entity_infos.remove(&lid);
                }
            }
        }
    }

    pub(crate) fn apply_entities(&mut self) -> eyre::Result<()> {
        for (gid, entity_info) in &self.entity_infos {
            match self.tracked.get_by_left(gid) {
                Some(entity) => {
                    entity.set_position(entity_info.x, entity_info.y)?;
                    if let Some(vel) = entity.try_get_first_component::<VelocityComponent>(None)? {
                        vel.set_m_velocity((entity_info.vx, entity_info.vy))?;
                    }
                    if let Some(vel) =
                        entity.try_get_first_component::<CharacterDataComponent>(None)?
                    {
                        vel.set_m_velocity((entity_info.vx, entity_info.vy))?;
                    }
                }
                None => {
                    let entity = self.spawn_entity_by_data(&entity_info.entity_data)?;
                    game_print("Spawned remote entity");
                    self.remove_unnecessary_components(entity)?;
                    entity.add_tag(DES_TAG)?;
                    self.tracked.insert(*gid, entity);
                }
            }
        }

        Ok(())
    }

    fn spawn_entity_by_data(&self, entity_data: &EntitySpawnInfo) -> eyre::Result<EntityID> {
        match entity_data {
            EntitySpawnInfo::Filename(filename) => EntityID::load(filename, None, None),
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

    pub(crate) fn spawn_projectiles(&self, projectiles: &[ProjectileFired]) {
        for projectile in projectiles {
            let Ok(deserialized) = deserialize_entity(
                &projectile.serialized,
                projectile.position.0,
                projectile.position.1,
            ) else {
                game_print("uhh something went wrong when spawning projectile: deserialize");
                continue;
            };
            let Some(&shooter_entity) = self.tracked.get_by_left(&projectile.shooter_lid) else {
                continue;
            };

            game_print(format!(
                "gsp {shooter_entity:?} {deserialized:?} {:?} {:?}",
                projectile.position, projectile.target,
            ));

            // TODO hangs at this point
            let _ = noita_api::raw::game_shoot_projectile(
                shooter_entity.raw() as i32,
                projectile.position.0 as f64,
                projectile.position.1 as f64,
                projectile.target.0 as f64,
                projectile.target.1 as f64,
                deserialized.raw() as i32,
                None,
                None,
            );
        }
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
