use std::mem;

use bimap::BiHashMap;
use eyre::OptionExt;
use noita_api::{
    game_print, AIAttackComponent, AdvancedFishAIComponent, AnimalAIComponent,
    CameraBoundComponent, CharacterDataComponent, DamageModelComponent, EntityID,
    ExplodeOnDamageComponent, ItemPickUpperComponent, PhysicsAIComponent, PhysicsBody2Component,
    VelocityComponent,
};
use rustc_hash::FxHashMap;
use shared::{
    des::{
        EntityInfo, EntitySpawnInfo, EntityUpdate, FullEntityData, Gid, Lid, ProjectileFired,
        UpdatePosition, AUTHORITY_RADIUS,
    },
    WorldPos,
};

use crate::{modules::ModuleCtx, serialize::deserialize_entity};

pub(crate) static DES_TAG: &str = "ew_des";

struct EntityEntryPair {
    last: Option<EntityInfo>,
    current: EntityInfo,
    gid: Gid,
}

pub(crate) struct LocalDiffModel {
    next_lid: Lid,
    tracked: BiHashMap<Lid, EntityID>,
    entity_entries: FxHashMap<Lid, EntityEntryPair>,
    pending_removal: Vec<Lid>,
    pending_authority: Vec<FullEntityData>,
    authority_radius: f32,
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
            pending_authority: Vec::new(),

            authority_radius: AUTHORITY_RADIUS,
        }
    }
}

impl LocalDiffModel {
    fn alloc_lid(&mut self) -> Lid {
        let ret = self.next_lid;
        self.next_lid.0 += 1;
        ret
    }

    pub(crate) fn track_entity(&mut self, entity: EntityID, gid: Gid) -> eyre::Result<Lid> {
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
                    hp: 1.0,
                },
                gid,
            },
        );
        Ok(lid)
    }

    pub(crate) fn reset_diff_encoding(&mut self) {
        for (_, entry_pair) in &mut self.entity_entries {
            entry_pair.last = None;
        }
    }

    pub(crate) fn update(&mut self) -> eyre::Result<()> {
        for entity_data in mem::take(&mut self.pending_authority) {
            let entity = spawn_entity_by_data(
                &entity_data.data,
                entity_data.pos.x as f32,
                entity_data.pos.y as f32,
            )?;
            self.track_entity(entity, entity_data.gid)?;
        }
        Ok(())
    }

    pub(crate) fn update_tracked_entities(&mut self, ctx: &mut ModuleCtx) -> eyre::Result<()> {
        let (cam_x, cam_y) = noita_api::raw::game_get_camera_pos()?;
        let cam_x = cam_x as f32;
        let cam_y = cam_y as f32;
        for (
            &lid,
            EntityEntryPair {
                last: _,
                current,
                gid,
            },
        ) in &mut self.entity_entries
        {
            let entity = self
                .tracked
                .get_by_left(&lid)
                .ok_or_eyre("Expected to find a corresponding entity")?;
            if !entity.is_alive() {
                self.pending_removal.push(lid);

                ctx.net.send(&shared::NoitaOutbound::DesToProxy(
                    shared::des::DesToProxy::DeleteEntity(*gid),
                ))?;

                continue;
            }
            let (x, y) = entity.position()?;
            current.x = x;
            current.y = y;

            // Check if entity went out of range, remove and release authority if it did.
            if (x - cam_x).powi(2) + (y - cam_y).powi(2) > self.authority_radius.powi(2) {
                ctx.net.send(&shared::NoitaOutbound::DesToProxy(
                    shared::des::DesToProxy::UpdatePositions(vec![UpdatePosition {
                        gid: *gid,
                        pos: WorldPos::from_f32(x, y),
                    }]),
                ))?;
                ctx.net.send(&shared::NoitaOutbound::DesToProxy(
                    shared::des::DesToProxy::ReleaseAuthority(*gid),
                ))?;
                game_print("Released authority over entity");

                self.pending_removal.push(lid);

                entity.kill();
                continue;
            }

            if let Some(vel) = entity.try_get_first_component::<VelocityComponent>(None)? {
                let (vx, vy) = vel.m_velocity()?;
                current.vx = vx;
                current.vy = vy;
            }
            if let Some(damage) = entity.try_get_first_component::<DamageModelComponent>(None)? {
                let hp = damage.hp()?;
                current.hp = hp as f32;
            }
        }
        Ok(())
    }

    pub(crate) fn make_diff(&mut self) -> Vec<EntityUpdate> {
        let mut res = Vec::new();
        for (
            &lid,
            EntityEntryPair {
                last,
                current,
                gid: _,
            },
        ) in &mut self.entity_entries
        {
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

            if current.hp != last.hp {
                res.push(EntityUpdate::SetHp(current.hp));
                last.hp = current.hp;
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

    pub(crate) fn got_authority(&mut self, full_entity_data: shared::des::FullEntityData) {
        self.pending_authority.push(full_entity_data);
    }

    pub(crate) fn full_entity_data_for(&self, lid: Lid) -> Option<FullEntityData> {
        let entry_pair = self.entity_entries.get(&lid)?;
        Some(FullEntityData {
            gid: entry_pair.gid,
            pos: WorldPos::from_f32(entry_pair.current.x, entry_pair.current.y),
            data: entry_pair.current.entity_data.clone(),
        })
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
                EntityUpdate::SetHp(hp) => {
                    let Some(entity_info) = self.entity_infos.get_mut(&current_lid) else {
                        continue;
                    };
                    entity_info.hp = *hp;
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
        for (lid, entity_info) in &self.entity_infos {
            match self.tracked.get_by_left(lid) {
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
                    if let Some(damage) =
                        entity.try_get_first_component::<DamageModelComponent>(None)?
                    {
                        let current_hp = damage.hp()? as f32;
                        if current_hp > entity_info.hp {
                            noita_api::raw::entity_inflict_damage(
                                entity.raw() as i32,
                                (current_hp - entity_info.hp) as f64,
                                "CURSE".into(),
                                "hp sync".into(),
                                "NONE".into(),
                                0.0,
                                0.0,
                                None,
                                None,
                                None,
                                None,
                            )?;
                        }
                    }
                }
                None => {
                    let entity = spawn_entity_by_data(
                        &entity_info.entity_data,
                        entity_info.x,
                        entity_info.y,
                    )?;
                    game_print("Spawned remote entity");
                    self.init_remote_entity(entity)?;
                    self.tracked.insert(*lid, entity);
                }
            }
        }

        Ok(())
    }

    /// Modifies a newly spawned entity so it can be synced properly.
    /// Removes components that shouldn't be on entities that were replicated from a remote,
    /// generally because they interfere with things we're supposed to sync.
    fn init_remote_entity(&self, entity: EntityID) -> eyre::Result<()> {
        entity.remove_all_components_of_type::<CameraBoundComponent>()?;
        entity.remove_all_components_of_type::<AnimalAIComponent>()?;
        entity.remove_all_components_of_type::<PhysicsAIComponent>()?;
        entity.remove_all_components_of_type::<AdvancedFishAIComponent>()?;
        entity.remove_all_components_of_type::<AIAttackComponent>()?;
        entity.remove_all_components_of_type::<ItemPickUpperComponent>()?;

        entity.add_tag(DES_TAG)?;
        entity.add_tag("polymorphable_NOT")?;
        if let Some(damage) = entity.try_get_first_component::<DamageModelComponent>(None)? {
            damage.set_wait_for_kill_flag_on_death(true)?;
        }

        for pb2 in entity.iter_all_components_of_type::<PhysicsBody2Component>(None)? {
            pb2.set_destroy_body_if_entity_destroyed(true)?;
        }

        for expl in entity.iter_all_components_of_type::<ExplodeOnDamageComponent>(None)? {
            expl.set_explode_on_damage_percent(0.0)?;
            expl.set_explode_on_death_percent(0.0)?;
            expl.set_physics_body_modified_death_probability(0.0)?;
        }

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

fn spawn_entity_by_data(entity_data: &EntitySpawnInfo, x: f32, y: f32) -> eyre::Result<EntityID> {
    match entity_data {
        EntitySpawnInfo::Filename(filename) => {
            EntityID::load(filename, Some(x as f64), Some(y as f64))
        }
    }
}
