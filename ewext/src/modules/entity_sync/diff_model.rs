use std::mem;

use bimap::BiHashMap;
use eyre::{Context, OptionExt};
use noita_api::serialize::{deserialize_entity, serialize_entity};
use noita_api::{
    game_print, AIAttackComponent, AdvancedFishAIComponent, AnimalAIComponent,
    CameraBoundComponent, CharacterDataComponent, DamageModelComponent, EntityID,
    ExplodeOnDamageComponent, ItemComponent, ItemCostComponent, ItemPickUpperComponent,
    LuaComponent, PhysData, PhysicsAIComponent, PhysicsBody2Component, SpriteComponent,
    VelocityComponent,
};
use rustc_hash::FxHashMap;
use shared::{
    des::{
        EntityInfo, EntityKind, EntitySpawnInfo, EntityUpdate, FullEntityData, Gid, Lid,
        PhysBodyInfo, ProjectileFired, UpdatePosition, AUTHORITY_RADIUS,
    },
    NoitaOutbound, PeerId, WorldPos,
};

use crate::{modules::ModuleCtx, my_peer_id, print_error};

use super::NetManager;

pub(crate) static DES_TAG: &str = "ew_des";
pub(crate) static DES_SCRIPTS_TAG: &str = "ew_des_lua";

struct EntityEntryPair {
    last: Option<EntityInfo>,
    current: EntityInfo,
    gid: Gid,
}

struct LocalDiffModelTracker {
    tracked: BiHashMap<Lid, EntityID>,
    pending_removal: Vec<Lid>,
    pending_authority: Vec<FullEntityData>,
    pending_localize: Vec<(Lid, PeerId)>,
    /// Stores pairs of entity killed and optionally the responsible entity.
    pending_death_notify: Vec<(EntityID, Option<EntityID>)>,
    authority_radius: f32,
}

pub(crate) struct LocalDiffModel {
    next_lid: Lid,
    entity_entries: FxHashMap<Lid, EntityEntryPair>,
    tracker: LocalDiffModelTracker,
}

#[derive(Default)]
pub(crate) struct RemoteDiffModel {
    tracked: BiHashMap<Lid, EntityID>,
    entity_infos: FxHashMap<Lid, EntityInfo>,
    /// Entities that we want to track again. Typically when we move authority locally from a different peer.
    backtrack: Vec<EntityID>,
    grab_request: Vec<Lid>,
    pending_remove: Vec<Lid>,
    pending_death_notify: Vec<(Lid, Option<PeerId>)>,
}

impl Default for LocalDiffModel {
    fn default() -> Self {
        Self {
            next_lid: Lid(0),
            entity_entries: Default::default(),
            tracker: LocalDiffModelTracker {
                tracked: Default::default(),
                pending_removal: Vec::with_capacity(16),
                pending_authority: Vec::new(),
                pending_localize: Vec::with_capacity(4),
                pending_death_notify: Vec::with_capacity(4),
                authority_radius: AUTHORITY_RADIUS,
            },
        }
    }
}

impl LocalDiffModelTracker {
    fn update_entity(
        &mut self,
        ctx: &mut ModuleCtx,
        gid: Gid,
        info: &mut EntityInfo,
        lid: Lid,
        cam_pos: (f32, f32),
    ) -> eyre::Result<()> {
        let entity = self.entity_by_lid(lid)?;

        if !entity.is_alive() {
            self.untrack_entity(ctx, gid, lid)?;
            return Ok(());
        }
        let item_and_was_picked = info.kind == EntityKind::Item && item_in_inventory(entity)?;
        if item_and_was_picked {
            self.temporary_untrack_item(ctx, gid, lid, entity)?;
            return Ok(());
        }

        let should_send_position = if info.kind == EntityKind::Item {
            if let Some(com) = entity.try_get_first_component::<ItemComponent>(None)? {
                !com.play_hover_animation()?
            } else {
                true
            }
        } else {
            true
        };

        let (x, y) = entity.position()?;
        if should_send_position {
            info.x = x;
            info.y = y;
        }

        // Check if entity went out of range, remove and release authority if it did.
        if (x - cam_pos.0).powi(2) + (y - cam_pos.1).powi(2) > self.authority_radius.powi(2) {
            self.release_authority(ctx, gid, lid)
                .wrap_err("Failed to release authority")?;
            return Ok(());
        }

        if let Some(vel) = entity.try_get_first_component::<VelocityComponent>(None)? {
            let (vx, vy) = vel.m_velocity()?;
            info.vx = vx;
            info.vy = vy;
        }
        if let Some(damage) = entity.try_get_first_component::<DamageModelComponent>(None)? {
            let hp = damage.hp()?;
            info.hp = hp as f32;
        }

        if check_all_phys_init(entity)? {
            info.phys = collect_phys_info(entity)?;
        }

        if let Some(item_cost) = entity.try_get_first_component::<ItemCostComponent>(None)? {
            info.cost = item_cost.cost()?;
        } else {
            info.cost = 0;
        }

        info.game_effects = entity.get_game_effects();

        info.current_stains = entity.get_current_stains();

        if let Ok(sprites) = entity.iter_all_components_of_type::<SpriteComponent>(None) {
            info.animations = sprites
                .map(|sprite| sprite.rect_animation().unwrap_or("".into()).to_string())
                .collect();
        } else {
            info.animations.clear()
        }

        Ok(())
    }

    fn untrack_entity(
        &mut self,
        ctx: &mut ModuleCtx<'_>,
        gid: Gid,
        lid: Lid,
    ) -> Result<(), eyre::Error> {
        self.pending_removal.push(lid);
        ctx.net.send(&NoitaOutbound::DesToProxy(
            shared::des::DesToProxy::DeleteEntity(gid),
        ))?;

        Ok(())
    }

    fn temporary_untrack_item(
        &mut self,
        ctx: &mut ModuleCtx<'_>,
        gid: Gid,
        lid: Lid,
        entity: EntityID,
    ) -> Result<(), eyre::Error> {
        self.untrack_entity(ctx, gid, lid)?;
        entity.remove_tag(DES_TAG)?;
        with_entity_scripts(entity, |luac| {
            luac.set_script_throw_item(
                "mods/quant.ew/files/system/entity_sync_helper/item_notify.lua".into(),
            )
        })?;
        Ok(())
    }

    fn entity_by_lid(&mut self, lid: Lid) -> eyre::Result<EntityID> {
        Ok(*self
            .tracked
            .get_by_left(&lid)
            .ok_or_eyre("Expected to find a corresponding entity")?)
    }

    fn release_authority(
        &mut self,
        ctx: &mut ModuleCtx<'_>,
        gid: Gid,
        lid: Lid,
    ) -> eyre::Result<()> {
        let entity = self.entity_by_lid(lid)?;
        let (x, y) = entity.position()?;
        ctx.net.send(&NoitaOutbound::DesToProxy(
            shared::des::DesToProxy::UpdatePositions(vec![UpdatePosition {
                gid,
                pos: WorldPos::from_f32(x, y),
            }]),
        ))?;
        ctx.net.send(&NoitaOutbound::DesToProxy(
            shared::des::DesToProxy::ReleaseAuthority(gid),
        ))?;
        self.pending_removal.push(lid);
        safe_entitykill(entity);
        Ok(())
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

        self.tracker.tracked.insert(lid, entity);

        let (x, y) = entity.position()?;

        let entity_kind = classify_entity(entity)?;
        let spawn_info = match entity_kind {
            EntityKind::Normal => EntitySpawnInfo::Filename(entity.filename()?),
            EntityKind::Item => EntitySpawnInfo::Serialized {
                serialized_at: noita_api::raw::game_get_frame_num()?,
                data: serialize_entity(entity)?,
            },
        };
        with_entity_scripts(entity, |scripts| {
            scripts.set_script_death(
                "mods/quant.ew/files/system/entity_sync_helper/death_notify.lua".into(),
            )
        })?;

        self.entity_entries.insert(
            lid,
            EntityEntryPair {
                last: None,
                current: EntityInfo {
                    spawn_info,
                    kind: entity_kind,
                    x,
                    y,
                    vx: 0.0,
                    vy: 0.0,
                    hp: 1.0, //TODO should be actual data?
                    phys: Vec::new(),
                    cost: 0,
                    game_effects: None,
                    current_stains: None,
                    animations: Vec::new(),
                },
                gid,
            },
        );

        Ok(lid)
    }

    pub(crate) fn track_and_upload_entity(
        &mut self,
        net: &mut NetManager,
        entity: EntityID,
        gid: Gid,
    ) -> eyre::Result<Lid> {
        let lid = self.track_entity(entity, gid)?;
        net.send(&NoitaOutbound::DesToProxy(
            shared::des::DesToProxy::InitOrUpdateEntity(
                self.full_entity_data_for(lid)
                    .ok_or_eyre("entity just began being tracked")?,
            ),
        ))?;
        Ok(lid)
    }

    pub(crate) fn reset_diff_encoding(&mut self) {
        for entry_pair in &mut self.entity_entries.values_mut() {
            entry_pair.last = None;
        }
    }

    pub(crate) fn update_pending_authority(&mut self) -> eyre::Result<()> {
        for entity_data in mem::take(&mut self.tracker.pending_authority) {
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
            if let Err(error) = self
                .tracker
                .update_entity(ctx, *gid, current, lid, (cam_x, cam_y))
                .wrap_err("Failed to update local entity")
            {
                print_error(error)?;
                self.tracker.untrack_entity(ctx, *gid, lid)?;
            }
        }
        Ok(())
    }

    pub(crate) fn make_diff(&mut self, ctx: &mut ModuleCtx) -> Vec<EntityUpdate> {
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

            if current.phys != last.phys {
                res.push(EntityUpdate::SetPhysInfo(current.phys.clone()));
                last.phys = current.phys.clone();
                had_any_delta = true;
            }
            
            if current.cost != last.cost {
                res.push(EntityUpdate::SetCost(current.cost));
                last.cost = current.cost;
                had_any_delta = true;
            }

            if current.current_stains != last.current_stains {
                res.push(EntityUpdate::SetStains(current.current_stains.clone()));
                last.current_stains = current.current_stains.clone();
                had_any_delta = true;
            }

            if current.game_effects != last.game_effects {
                res.push(EntityUpdate::SetGameEffects(current.game_effects.clone()));
                last.game_effects = current.game_effects.clone();
                had_any_delta = true;
            }

            if current.animations != last.animations {
                res.push(EntityUpdate::SetAnimations(current.animations.clone()));
                last.animations = current.animations.clone();
                had_any_delta = true;
            }

            // Remove the CurrentEntity thing because it's not necessary.
            if !had_any_delta {
                res.pop();
            }
        }
        for (lid, peer) in self.tracker.pending_localize.drain(..) {
            res.push(EntityUpdate::LocalizeEntity(lid, peer));
        }

        for (killed, responsible) in self.tracker.pending_death_notify.drain(..) {
            let responsible_peer = responsible
                .and_then(|ent| ctx.player_map.get_by_right(&ent))
                .copied();
            let Some(lid) = self.tracker.tracked.get_by_right(&killed).copied() else {
                continue;
            };
            res.push(EntityUpdate::KillEntity {
                lid,
                responsible_peer,
            });
        }

        for lid in self.tracker.pending_removal.drain(..) {
            res.push(EntityUpdate::RemoveEntity(lid));
            // "Untrack" entity
            self.tracker.tracked.remove_by_left(&lid);
            self.entity_entries.remove(&lid);
        }
        res
    }

    pub(crate) fn lid_by_entity(&self, entity: EntityID) -> Option<Lid> {
        self.tracker.tracked.get_by_right(&entity).copied()
    }

    pub(crate) fn got_authority(&mut self, full_entity_data: FullEntityData) {
        self.tracker.pending_authority.push(full_entity_data);
    }

    pub(crate) fn full_entity_data_for(&self, lid: Lid) -> Option<FullEntityData> {
        let entry_pair = self.entity_entries.get(&lid)?;
        Some(FullEntityData {
            gid: entry_pair.gid,
            pos: WorldPos::from_f32(entry_pair.current.x, entry_pair.current.y),
            data: entry_pair.current.spawn_info.clone(),
        })
    }

    pub(crate) fn entity_grabbed(&mut self, source: PeerId, lid: Lid) {
        let Some(info) = self.entity_entries.get(&lid) else {
            return;
        };
        if let Ok(entity) = self.tracker.entity_by_lid(lid) {
            if info.current.kind == EntityKind::Item {
                self.tracker.pending_localize.push((lid, source));
                safe_entitykill(entity);
                // "Untrack" entity
                self.tracker.tracked.remove_by_left(&lid);
                self.entity_entries.remove(&lid);
            } else {
                game_print("Tried to localize entity that's not an item");
            }
        }
    }

    pub(crate) fn death_notify(
        &mut self,
        entity_killed: EntityID,
        entity_responsible: Option<EntityID>,
    ) {
        self.tracker
            .pending_death_notify
            .push((entity_killed, entity_responsible))
    }
}

fn check_all_phys_init(entity: EntityID) -> eyre::Result<bool> {
    for phys_c in entity.iter_all_components_of_type::<PhysicsBody2Component>(None)? {
        if !phys_c.m_initialized()? {
            return Ok(false);
        }
    }

    Ok(true)
}

fn collect_phys_info(entity: EntityID) -> eyre::Result<Vec<Option<PhysBodyInfo>>> {
    let phys_bodies = noita_api::raw::physics_body_id_get_from_entity(entity, None)?;
    phys_bodies
        .into_iter()
        .map(|body| -> eyre::Result<Option<PhysBodyInfo>> {
            Ok(
                noita_api::raw::physics_body_id_get_transform(body)?.and_then(|data| {
                    let PhysData {
                        x,
                        y,
                        angle,
                        vx,
                        vy,
                        av,
                    } = data;
                    let (x, y) =
                        noita_api::raw::physics_pos_to_game_pos(x.into(), Some(y.into())).ok()?;

                    Some(PhysBodyInfo {
                        x: x as f32,
                        y: y as f32,
                        angle,
                        vx,
                        vy,
                        av,
                    })
                }),
            )
        })
        .collect::<eyre::Result<Vec<_>>>()
}

impl RemoteDiffModel {
    pub(crate) fn apply_diff(&mut self, diff: &[EntityUpdate]) {
        let mut current_lid = Lid(0);
        for entry in diff.iter().cloned() {
            match entry {
                EntityUpdate::CurrentEntity(lid) => current_lid = lid,
                EntityUpdate::Init(entity_entry) => {
                    self.entity_infos.insert(current_lid, entity_entry);
                }
                EntityUpdate::SetPosition(x, y) => {
                    let Some(ent_data) = self.entity_infos.get_mut(&current_lid) else {
                        continue;
                    };
                    ent_data.x = x;
                    ent_data.y = y;
                }
                EntityUpdate::SetVelocity(vx, vy) => {
                    let Some(entity_info) = self.entity_infos.get_mut(&current_lid) else {
                        continue;
                    };
                    entity_info.vx = vx;
                    entity_info.vy = vy;
                }
                EntityUpdate::SetHp(hp) => {
                    let Some(entity_info) = self.entity_infos.get_mut(&current_lid) else {
                        continue;
                    };
                    entity_info.hp = hp;
                }
                EntityUpdate::SetPhysInfo(vec) => {
                    let Some(entity_info) = self.entity_infos.get_mut(&current_lid) else {
                        continue;
                    };
                    entity_info.phys = vec.clone();
                }
                EntityUpdate::RemoveEntity(lid) => {
                    self.pending_remove.push(lid);
                }
                EntityUpdate::LocalizeEntity(lid, peer_id) => {
                    if let Some((_, entity)) = self.tracked.remove_by_left(&lid) {
                        if peer_id != my_peer_id() {
                            safe_entitykill(entity);
                        } else {
                            self.backtrack.push(entity);
                        }
                    }
                    self.entity_infos.remove(&lid);
                }
                EntityUpdate::KillEntity {
                    lid,
                    responsible_peer,
                } => {
                    self.pending_death_notify.push((lid, responsible_peer));
                }
                EntityUpdate::SetCost(cost) => {
                    let Some(entity_info) = self.entity_infos.get_mut(&current_lid) else {
                        continue;
                    };
                    entity_info.cost = cost;
                }
                EntityUpdate::SetStains(stains) => {
                    let Some(entity_info) = self.entity_infos.get_mut(&current_lid) else {
                        continue;
                    };
                    entity_info.current_stains = stains;
                }
                EntityUpdate::SetGameEffects(effects) => {
                    let Some(entity_info) = self.entity_infos.get_mut(&current_lid) else {
                        continue;
                    };
                    entity_info.game_effects = effects;
                }
                EntityUpdate::SetAnimations(animations) => {
                    let Some(entity_info) = self.entity_infos.get_mut(&current_lid) else {
                        continue;
                    };
                    entity_info.animations = animations;
                }
            }
        }
    }

    pub(crate) fn apply_entities(&mut self, ctx: &mut ModuleCtx) -> eyre::Result<()> {
        for (lid, entity_info) in &self.entity_infos {
            match self
                .tracked
                .get_by_left(lid)
                .and_then(|entity_id| entity_id.is_alive().then_some(*entity_id))
            {
                Some(entity) => {
                    if entity_info.kind == EntityKind::Item && item_in_inventory(entity)? {
                        self.grab_request.push(*lid);
                    }

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
                                "DAMAGE_CURSE".into(),
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

                    if !entity_info.phys.is_empty() && check_all_phys_init(entity)? {
                        let phys_bodies =
                            noita_api::raw::physics_body_id_get_from_entity(entity, None)?;
                        for (p, physics_body_id) in entity_info.phys.iter().zip(phys_bodies.iter())
                        {
                            let Some(p) = p else {
                                continue;
                            };
                            let (x, y) = noita_api::raw::game_pos_to_physics_pos(
                                p.x.into(),
                                Some(p.y.into()),
                            )?;
                            noita_api::raw::physics_body_id_set_transform(
                                *physics_body_id,
                                x,
                                y,
                                p.angle.into(),
                                p.vx.into(),
                                p.vy.into(),
                                p.av.into(),
                            )?;
                        }
                    }

                    if let Some(cost) = entity.try_get_first_component::<ItemCostComponent>(None)? {
                        cost.set_cost(entity_info.cost)?;
                    }

                    entity.set_game_effects(&entity_info.game_effects);

                    entity.set_current_stains(&entity_info.current_stains);

                    if let Ok(sprites) = entity.iter_all_components_of_type::<SpriteComponent>(None)
                    {
                        for (sprite, animation) in sprites.zip(entity_info.animations.iter()) {
                            sprite.set_rect_animation(animation.into())?;
                            sprite.set_next_rect_animation(animation.into())?;
                        }
                    }
                }
                None => {
                    let entity = spawn_entity_by_data(
                        &entity_info.spawn_info,
                        entity_info.x,
                        entity_info.y,
                    )?;
                    self.init_remote_entity(entity)?;
                    self.tracked.insert(*lid, entity);
                }
            }
        }

        let mut postpone_remove = Vec::new();

        for (lid, responsible) in self.pending_death_notify.drain(..) {
            let responsible_entity = responsible
                .and_then(|peer| ctx.player_map.get_by_left(&peer))
                .copied();
            let Some(entity) = self.tracked.get_by_left(&lid).copied() else {
                continue;
            };

            if let Some(explosion) =
                entity.try_get_first_component::<ExplodeOnDamageComponent>(None)?
            {
                explosion.set_explode_on_death_percent(1.0)?;
            }

            if let Some(damage) = entity.try_get_first_component::<DamageModelComponent>(None)? {
                damage.set_wait_for_kill_flag_on_death(false)?;
                damage.set_ui_report_damage(false)?;
                damage.set_hp(f64::MIN_POSITIVE)?;
                noita_api::raw::entity_inflict_damage(
                    entity.raw() as i32,
                    damage.hp()? + 0.1,
                    "DAMAGE_CURSE".into(),
                    "kill sync".into(),
                    "NONE".into(),
                    0.0,
                    0.0,
                    responsible_entity.map(|e| e.raw() as i32),
                    None,
                    None,
                    None,
                )?;
            }

            postpone_remove.push(lid);
        }

        for lid in self.pending_remove.drain(..) {
            if postpone_remove.contains(&lid) {
                continue;
            }
            if let Some((_, entity)) = self.tracked.remove_by_left(&lid) {
                safe_entitykill(entity);
            }
            self.entity_infos.remove(&lid);
        }

        self.pending_remove.extend_from_slice(&postpone_remove);

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

        if let Some(itemc) = entity.try_get_first_component::<ItemCostComponent>(None)? {
            itemc.set_stealable(false)?;
        }

        if let Ok(sprites) =
            entity.iter_all_components_of_type::<SpriteComponent>(Some("character".into()))
        {
            for sprite in sprites {
                sprite.0.remove_tag("character")?
            }
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

    pub(crate) fn drain_backtrack(&mut self) -> impl Iterator<Item = EntityID> + '_ {
        self.backtrack.drain(..)
    }

    pub(crate) fn drain_grab_request(&mut self) -> impl Iterator<Item = Lid> + '_ {
        self.grab_request.drain(..)
    }
}

fn item_in_inventory(entity: EntityID) -> Result<bool, eyre::Error> {
    Ok(entity.parent()? != entity)
}

impl Drop for RemoteDiffModel {
    fn drop(&mut self) {
        // Cleanup all entities tracked by this model.
        for ent in self.tracked.right_values() {
            safe_entitykill(*ent);
        }
    }
}

fn spawn_entity_by_data(entity_data: &EntitySpawnInfo, x: f32, y: f32) -> eyre::Result<EntityID> {
    match entity_data {
        EntitySpawnInfo::Filename(filename) => {
            EntityID::load(filename, Some(x as f64), Some(y as f64))
        }
        // TODO handle things like wand recharge time.
        EntitySpawnInfo::Serialized {
            serialized_at: _,
            data,
        } => deserialize_entity(data, x, y),
    }
}

pub(crate) fn entity_is_item(entity: EntityID) -> eyre::Result<bool> {
    Ok(entity
        .try_get_first_component_including_disabled::<ItemComponent>(None)?
        .is_some())
}

fn classify_entity(entity: EntityID) -> eyre::Result<EntityKind> {
    if entity_is_item(entity)? {
        return Ok(EntityKind::Item);
    }

    Ok(EntityKind::Normal)
}

fn with_entity_scripts<T>(
    entity: EntityID,
    f: impl FnOnce(LuaComponent) -> eyre::Result<T>,
) -> eyre::Result<T> {
    let component = entity
        .try_get_first_component(Some(DES_SCRIPTS_TAG.into()))
        .transpose()
        .unwrap_or_else(|| {
            let component = entity.add_component::<LuaComponent>()?;
            component.0.add_tag(DES_SCRIPTS_TAG)?;
            component.0.add_tag("enabled_in_inventory")?;
            component.0.add_tag("enabled_in_world")?;
            component.0.add_tag("enabled_in_hand")?;
            Ok(component)
        })?;
    f(component)
}

fn safe_entitykill(entity: EntityID) {
    // TODO
    entity.kill();
}
