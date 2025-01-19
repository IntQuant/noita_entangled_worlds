use super::NetManager;
use crate::{modules::ModuleCtx, my_peer_id, print_error};
use bimap::BiHashMap;
use eyre::{Context, ContextCompat, OptionExt};
use noita_api::raw::{entity_create_new, raytrace_platforms};
use noita_api::serialize::{deserialize_entity, serialize_entity};
use noita_api::{
    game_print, AIAttackComponent, AbilityComponent, AdvancedFishAIComponent, AnimalAIComponent,
    AudioComponent, BossDragonComponent, BossHealthBarComponent, CameraBoundComponent,
    CharacterDataComponent, DamageModelComponent, EntityID, ExplodeOnDamageComponent,
    IKLimbComponent, IKLimbWalkerComponent, Inventory2Component, ItemComponent, ItemCostComponent,
    ItemPickUpperComponent, LaserEmitterComponent, LuaComponent, PhysData, PhysicsAIComponent,
    PhysicsBody2Component, SpriteComponent, StreamingKeepAliveComponent, VariableStorageComponent,
    VelocityComponent, WormComponent,
};
use rustc_hash::{FxHashMap, FxHashSet};
use shared::des::TRANSFER_RADIUS;
use shared::{
    des::{
        EntityInfo, EntityKind, EntitySpawnInfo, EntityUpdate, FullEntityData, Gid, Lid,
        PhysBodyInfo, ProjectileFired, UpdatePosition, AUTHORITY_RADIUS,
    },
    GameEffectData, NoitaOutbound, PeerId, WorldPos,
};
use std::mem;
use std::num::NonZero;
pub(crate) static DES_TAG: &str = "ew_des";
pub(crate) static DES_SCRIPTS_TAG: &str = "ew_des_lua";

struct EntityEntryPair {
    last: Option<EntityInfo>,
    current: EntityInfo,
    gid: Gid,
}

struct LocalDiffModelTracker {
    tracked: BiHashMap<Lid, EntityID>,
    give_gid: FxHashSet<Gid>,
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
impl LocalDiffModel {
    /*pub(crate) fn has_gid(&self, gid: Gid) -> bool {
        self.entity_entries.iter().any(|(_, e)| e.gid == gid)
    }*/
    pub(crate) fn find_by_gid(&self, gid: Gid) -> Option<EntityID> {
        self.entity_entries
            .iter()
            .find(|(_, e)| e.gid == gid)
            .map(|e| self.tracker.entity_by_lid(*e.0))?
            .ok()
    }
}
pub(crate) struct RemoteDiffModel {
    tracked: BiHashMap<Lid, EntityID>,
    entity_infos: FxHashMap<Lid, EntityInfo>,
    lid_to_gid: FxHashMap<Lid, Gid>,
    waiting_for_lid: FxHashMap<Gid, EntityID>,
    /// Entities that we want to track again. Typically when we move authority locally from a different peer.
    backtrack: Vec<EntityID>,
    grab_request: Vec<Lid>,
    pending_remove: Vec<Lid>,
    pending_death_notify: Vec<(Lid, Option<PeerId>)>,
    peer_id: PeerId,
}

impl RemoteDiffModel {
    pub fn new(peer_id: PeerId) -> Self {
        Self {
            tracked: Default::default(),
            entity_infos: Default::default(),
            lid_to_gid: Default::default(),
            waiting_for_lid: Default::default(),
            backtrack: Default::default(),
            grab_request: Default::default(),
            pending_remove: Default::default(),
            pending_death_notify: Default::default(),
            peer_id,
        }
    }
    /*pub fn has_gid(&self, gid: Gid) -> bool {
        self.lid_to_gid.iter().any(|(_, g)| *g == gid)
    }*/
    pub(crate) fn find_by_gid(&self, gid: Gid) -> Option<EntityID> {
        self.lid_to_gid
            .iter()
            .find(|(_, g)| **g == gid)
            .map(|l| self.tracked.get_by_left(l.0))?
            .copied()
    }
}

impl Default for LocalDiffModel {
    fn default() -> Self {
        Self {
            next_lid: Lid(0),
            entity_entries: Default::default(),
            tracker: LocalDiffModelTracker {
                tracked: Default::default(),
                give_gid: Default::default(),
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

        let should_send_position =
            if let Some(com) = entity.try_get_first_component::<ItemComponent>(None)? {
                !com.play_hover_animation()?
            } else {
                true
            };

        let (x, y) = entity.position()?;
        if should_send_position {
            (info.x, info.y) = (x, y);
        }

        let should_send_rotation =
            if let Some(com) = entity.try_get_first_component::<ItemComponent>(None)? {
                !com.play_spinning_animation()?
            } else {
                true
            };

        if should_send_rotation {
            info.r = entity.rotation()?
        }

        if let Some(inv) =
            entity.try_get_first_component_including_disabled::<Inventory2Component>(None)?
        {
            info.wand = if let Some(wand) = inv.m_actual_active_item()? {
                if let Some(gid) = wand
                    .iter_all_components_of_type_including_disabled::<VariableStorageComponent>(
                        None,
                    )?
                    .find_map(|var| {
                        if var.name().ok()? == "ew_gid_lid" {
                            Some(var.value_string().ok()?.parse::<u64>().ok()?)
                        } else {
                            None
                        }
                    })
                {
                    Some((Some(Gid(gid)), serialize_entity(wand)?))
                } else {
                    Some((None, serialize_entity(wand)?))
                }
            } else {
                None
            };
        }
        info.is_enabled = (entity.has_tag("boss_centipede")
            && entity
                .try_get_first_component::<BossHealthBarComponent>(Some(
                    "disabled_at_start".into(),
                ))?
                .is_some())
            || entity
                .try_get_first_component_including_disabled::<VariableStorageComponent>(None)?
                .iter()
                .any(|var| {
                    var.name().unwrap_or("".into()) == "active" && var.value_int().unwrap_or(0) == 1
                });

        if entity.has_tag("boss_wizard") {
            info.counter = entity
                .children(None)
                .iter()
                .filter_map(|ent| {
                    if ent.has_tag("touchmagic_immunity") {
                        let var = ent
                            .get_first_component_including_disabled::<VariableStorageComponent>(
                                None,
                            )
                            .ok()?;
                        Some(1 << var.value_int().ok()?)
                    } else {
                        None
                    }
                })
                .sum()
        /*} else if entity.filename()? == *"data/entities/buildings/wizardcave_gate.xml" {
        info.counter = entity
            .iter_all_components_of_type_including_disabled::<VariableStorageComponent>(None)?
            .find(|var| var.name().ok() == Some("egg_count".into()))
            .map(|var| var.value_int())
            .unwrap_or(Ok(0))? as u8*/
        } else if entity.has_tag("boss_dragon")
            && entity
                .iter_all_components_of_type::<LuaComponent>(None)?
                .any(|lua| {
                    lua.script_death().ok()
                        == Some("data/scripts/animals/boss_dragon_death.lua".into())
                })
        {
            info.counter = 1
        }

        info.limbs = entity
            .children(None)
            .iter()
            .filter_map(|ent| {
                if let Ok(limb) = ent.get_first_component::<IKLimbComponent>(None) {
                    limb.end_position().ok()
                } else {
                    None
                }
            })
            .collect();

        // Check if entity went out of range, remove and release authority if it did.
        let is_beyond_authority =
            (x - cam_pos.0).powi(2) + (y - cam_pos.1).powi(2) > self.authority_radius.powi(2);
        if is_beyond_authority {
            if info.is_global {
                if let Some(peer) = ctx.locate_player_within_except_me(x, y, TRANSFER_RADIUS)? {
                    self.transfer_authority_to(ctx, gid, lid, peer)
                        .wrap_err("Failed to transfer authority")?;
                    return Ok(());
                }
            } else {
                self.release_authority(ctx, gid, lid)
                    .wrap_err("Failed to release authority")?;
                return Ok(());
            }
        }

        if let Some(worm) = entity.try_get_first_component::<BossDragonComponent>(None)? {
            (info.vx, info.vy) = worm.m_target_vec()?;
        } else if let Some(worm) = entity.try_get_first_component::<WormComponent>(None)? {
            (info.vx, info.vy) = worm.m_target_vec()?;
        } else if let Some(vel) = entity.try_get_first_component::<VelocityComponent>(None)? {
            (info.vx, info.vy) = vel.m_velocity()?;
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

        info.game_effects = entity.get_game_effects().map(|e| {
            e.iter()
                .map(|(e, _)| e.clone())
                .collect::<Vec<GameEffectData>>()
        });

        info.current_stains = entity.get_current_stains()?;

        if let Ok(sprites) = entity.iter_all_components_of_type::<SpriteComponent>(None) {
            info.animations = sprites
                .filter_map(|sprite| {
                    let file = sprite.image_file().ok()?;
                    if file.ends_with(".xml") {
                        let text = noita_api::raw::mod_text_file_get_content(file).ok()?;
                        let mut split = text.split("name=\"");
                        split.next();
                        let data: Vec<&str> =
                            split.filter_map(|piece| piece.split("\"").next()).collect();
                        let animation = sprite.rect_animation().unwrap_or("".into());
                        Some(
                            data.iter()
                                .position(|name| name == &animation)
                                .unwrap_or(usize::MAX) as u16,
                        )
                    } else {
                        None
                    }
                })
                .collect();
        } else {
            info.animations.clear()
        }

        info.laser = None;
        if entity
            .try_get_first_component::<SpriteComponent>(Some("laser_sight".into()))?
            .is_some()
            && &entity.name()? != "$animal_turret"
        {
            let ai = entity.get_first_component::<AnimalAIComponent>(None)?;
            if let Some(target) = ai.m_greatest_prey()? {
                info.laser = ctx.player_map.get_by_right(&target).copied()
            }
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

    fn entity_by_lid(&self, lid: Lid) -> eyre::Result<EntityID> {
        Ok(*self
            .tracked
            .get_by_left(&lid)
            .ok_or_eyre("Expected to find a corresponding entity")?)
    }

    fn _release_authority_update_data(
        &mut self,
        ctx: &mut ModuleCtx<'_>,
        gid: Gid,
        lid: Lid,
    ) -> Result<EntityID, eyre::Error> {
        let entity = self.entity_by_lid(lid)?;
        let (x, y) = entity.position()?;
        ctx.net.send(&NoitaOutbound::DesToProxy(
            shared::des::DesToProxy::UpdatePositions(vec![UpdatePosition {
                gid,
                pos: WorldPos::from_f32(x, y),
            }]),
        ))?;
        Ok(entity)
    }

    fn release_authority(
        &mut self,
        ctx: &mut ModuleCtx<'_>,
        gid: Gid,
        lid: Lid,
    ) -> eyre::Result<()> {
        let entity = self._release_authority_update_data(ctx, gid, lid)?;
        ctx.net.send(&NoitaOutbound::DesToProxy(
            shared::des::DesToProxy::ReleaseAuthority(gid),
        ))?;
        self.pending_removal.push(lid);
        safe_entitykill(entity);
        Ok(())
    }

    fn transfer_authority_to(
        &mut self,
        ctx: &mut ModuleCtx<'_>,
        gid: Gid,
        lid: Lid,
        peer: PeerId,
    ) -> eyre::Result<()> {
        let entity = self._release_authority_update_data(ctx, gid, lid)?;
        ctx.net.send(&NoitaOutbound::DesToProxy(
            shared::des::DesToProxy::TransferAuthorityTo(gid, peer),
        ))?;
        self.pending_removal.push(lid);
        safe_entitykill(entity);
        Ok(())
    }
}

impl LocalDiffModel {
    pub(crate) fn give_gid(&mut self, gid: Gid) {
        self.tracker.give_gid.insert(gid);
    }
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
        entity
            .iter_all_components_of_type_including_disabled::<VariableStorageComponent>(None)?
            .for_each(|var| {
                if var.name().unwrap_or("".into()) == "ew_gid_lid" {
                    let _ = entity.remove_component(*var);
                }
            });
        let var = entity.add_component::<VariableStorageComponent>()?;
        var.set_name("ew_gid_lid".into())?;
        var.set_value_string(gid.0.to_string().into())?;
        var.set_value_int(i32::from_ne_bytes(lid.0.to_ne_bytes()))?;
        var.set_value_bool(true)?;

        if entity
            .try_get_first_component::<BossDragonComponent>(None)?
            .is_some()
            && entity
                .try_get_first_component::<StreamingKeepAliveComponent>(None)?
                .is_none()
        {
            entity.add_component::<StreamingKeepAliveComponent>()?;
        }

        let is_global = entity
            .try_get_first_component_including_disabled::<BossHealthBarComponent>(None)?
            .is_some()
            || entity
                .try_get_first_component::<StreamingKeepAliveComponent>(None)?
                .is_some();

        let drops_gold = entity
            .iter_all_components_of_type::<LuaComponent>(None)?
            .any(|lua| lua.script_death().ok() == Some("data/scripts/items/drop_money.lua".into()));

        self.entity_entries.insert(
            lid,
            EntityEntryPair {
                last: None,
                current: EntityInfo {
                    spawn_info,
                    kind: entity_kind,
                    x,
                    y,
                    r: 0.0,
                    vx: 0.0,
                    vy: 0.0,
                    hp: 1.0,
                    phys: Vec::new(),
                    cost: 0,
                    game_effects: None,
                    current_stains: 0,
                    animations: Vec::new(),
                    wand: None,
                    is_global,
                    drops_gold,
                    laser: Default::default(),
                    limbs: vec![],
                    is_enabled: false,
                    counter: 0,
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
        for (&lid, EntityEntryPair { last, current, gid }) in self.entity_entries.iter_mut() {
            res.push(EntityUpdate::CurrentEntity(lid));
            let Some(last) = last.as_mut() else {
                *last = Some(current.clone());
                res.push(EntityUpdate::Init(Box::from(current.clone()), *gid));
                continue;
            };
            let mut had_any_delta = false;

            fn diff<T: PartialEq + Clone>(
                current: &T,
                last: &mut T,
                update: EntityUpdate,
                res: &mut Vec<EntityUpdate>,
                had_any_delta: &mut bool,
            ) {
                if current != last {
                    res.push(update);
                    *last = current.clone();
                    *had_any_delta = true;
                }
            }
            if current.wand.clone().map(|(g, _)| g) != last.wand.clone().map(|(g, _)| g) {
                res.push(EntityUpdate::SetWand(current.wand.clone()));
                last.wand = current.wand.clone();
                had_any_delta = true;
            }
            diff(
                &(current.x, current.y),
                &mut (last.x, last.y),
                EntityUpdate::SetPosition(current.x, current.y),
                &mut res,
                &mut had_any_delta,
            );
            diff(
                &(current.vx, current.vy),
                &mut (last.vx, last.vy),
                EntityUpdate::SetVelocity(current.vx, current.vy),
                &mut res,
                &mut had_any_delta,
            );
            diff(
                &current.hp,
                &mut last.hp,
                EntityUpdate::SetHp(current.hp),
                &mut res,
                &mut had_any_delta,
            );
            diff(
                &current.phys,
                &mut last.phys,
                EntityUpdate::SetPhysInfo(current.phys.clone()),
                &mut res,
                &mut had_any_delta,
            );
            diff(
                &current.cost,
                &mut last.cost,
                EntityUpdate::SetCost(current.cost),
                &mut res,
                &mut had_any_delta,
            );
            diff(
                &current.current_stains,
                &mut last.current_stains,
                EntityUpdate::SetStains(current.current_stains),
                &mut res,
                &mut had_any_delta,
            );
            diff(
                &current.game_effects,
                &mut last.game_effects,
                EntityUpdate::SetGameEffects(current.game_effects.clone()),
                &mut res,
                &mut had_any_delta,
            );
            diff(
                &current.animations,
                &mut last.animations,
                EntityUpdate::SetAnimations(current.animations.clone()),
                &mut res,
                &mut had_any_delta,
            );
            diff(
                &current.laser,
                &mut last.laser,
                EntityUpdate::SetLaser(current.laser),
                &mut res,
                &mut had_any_delta,
            );
            diff(
                &current.limbs,
                &mut last.limbs,
                EntityUpdate::SetLimbs(current.limbs.clone()),
                &mut res,
                &mut had_any_delta,
            );
            diff(
                &current.is_enabled,
                &mut last.is_enabled,
                EntityUpdate::SetIsEnabled(current.is_enabled),
                &mut res,
                &mut had_any_delta,
            );
            diff(
                &current.counter,
                &mut last.counter,
                EntityUpdate::SetCounter(current.counter),
                &mut res,
                &mut had_any_delta,
            );

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
    pub(crate) fn wait_for_gid(&mut self, entity: EntityID, gid: Gid) {
        self.waiting_for_lid.insert(gid, entity);
    }
    pub(crate) fn apply_diff(&mut self, diff: &[EntityUpdate]) {
        let mut current_lid = Lid(0);
        let empty_data = &mut EntityInfo::default();
        let mut ent_data = &mut EntityInfo::default();
        for entry in diff.iter().cloned() {
            match entry {
                EntityUpdate::CurrentEntity(lid) => {
                    current_lid = lid;
                    ent_data = self
                        .entity_infos
                        .get_mut(&current_lid)
                        .unwrap_or(empty_data)
                }
                EntityUpdate::Init(entity_entry, gid) => {
                    if let Some(ent) = self.waiting_for_lid.remove(&gid) {
                        self.tracked.insert(current_lid, ent);
                    }
                    self.lid_to_gid.insert(current_lid, gid);
                    self.entity_infos.insert(current_lid, *entity_entry);
                    ent_data = self
                        .entity_infos
                        .get_mut(&current_lid)
                        .unwrap_or(empty_data)
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
                    ent_data = empty_data;
                }
                entry if *ent_data != EntityInfo::default() => match entry {
                    EntityUpdate::SetPosition(x, y) => (ent_data.x, ent_data.y) = (x, y),
                    EntityUpdate::SetRotation(r) => ent_data.r = r,
                    EntityUpdate::SetVelocity(vx, vy) => (ent_data.vx, ent_data.vy) = (vx, vy),
                    EntityUpdate::SetHp(hp) => ent_data.hp = hp,
                    EntityUpdate::SetPhysInfo(vec) => ent_data.phys = vec,
                    EntityUpdate::SetCost(cost) => ent_data.cost = cost,
                    EntityUpdate::SetStains(stains) => ent_data.current_stains = stains,
                    EntityUpdate::SetGameEffects(effects) => ent_data.game_effects = effects,
                    EntityUpdate::SetAnimations(animations) => ent_data.animations = animations,
                    EntityUpdate::RemoveEntity(lid) => self.pending_remove.push(lid),
                    EntityUpdate::KillEntity {
                        lid,
                        responsible_peer, //TODO make sure entity exists
                    } => self.pending_death_notify.push((lid, responsible_peer)),
                    EntityUpdate::SetWand(gid) => ent_data.wand = gid,
                    EntityUpdate::SetLaser(peer) => ent_data.laser = peer,
                    EntityUpdate::SetLimbs(limbs) => ent_data.limbs = limbs,
                    EntityUpdate::SetIsEnabled(enabled) => ent_data.is_enabled = enabled,
                    EntityUpdate::SetCounter(orbs) => ent_data.counter = orbs,
                    EntityUpdate::CurrentEntity(_)
                    | EntityUpdate::Init(_, _)
                    | EntityUpdate::LocalizeEntity(_, _) => unreachable!(),
                },
                _ => {}
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
                    if entity.has_tag("boss_wizard") {
                        for ent in entity.children(None) {
                            if ent.has_tag("touchmagic_immunity") {
                                if let Ok(var) =
                                    ent.get_first_component_including_disabled::<VariableStorageComponent>(None)
                                {
                                    if let Ok(n) = var.value_int() {
                                        if (entity_info.counter & (1 << (n as u8))) == 0 {
                                            ent.kill()
                                        } else if let Ok(damage) =
                                            ent.get_first_component::<DamageModelComponent>(None)
                                        {
                                            damage.set_wait_for_kill_flag_on_death(true)?;
                                            damage.set_hp(damage.max_hp()?)?;
                                        }
                                    }
                                }
                            }
                        }
                    } else if entity.has_tag("boss_dragon")
                        && entity_info.counter == 1
                        && entity
                            .iter_all_components_of_type_including_disabled::<LuaComponent>(None)?
                            .all(|lua| {
                                lua.script_death().ok()
                                    != Some("data/scripts/animals/boss_dragon_death.lua".into())
                            })
                    {
                        let lua = entity.add_component::<LuaComponent>()?;
                        lua.set_script_death("data/scripts/animals/boss_dragon_death.lua".into())?;
                        lua.set_execute_every_n_frame(-1)?;
                    }

                    if let Some((gid, seri)) = &entity_info.wand {
                        let inv = if let Some(inv) = entity
                            .try_get_first_component_including_disabled::<Inventory2Component>(
                                None,
                            )? {
                            inv
                        } else {
                            entity.add_component::<Inventory2Component>()?
                        };
                        let mut stop = false;
                        if let Some(wand) = inv.m_actual_active_item()? {
                            if let Some(tgid) = wand
                                .iter_all_components_of_type_including_disabled::<VariableStorageComponent>(None)?
                                .find_map(|var| {
                                    if var.name().ok()? == "ew_gid_lid" {
                                        Some(var.value_string().ok()?.parse::<u64>().ok()?)
                                    } else {
                                        None
                                    }
                                })
                            {
                                if *gid != Some(Gid(tgid)) {
                                    wand.kill()
                                } else {
                                    stop = true
                                }
                            } else if wand
                                .iter_all_components_of_type_including_disabled::<VariableStorageComponent>(None)?
                                .any(|p| p.name().ok().unwrap_or("".into()) == "ew_spawned_wand")
                            {
                                stop = true
                            } else {
                                wand.kill()
                            }
                        }
                        if !stop {
                            let (x, y) = entity.position()?;
                            let wand = deserialize_entity(seri, x, y)?;
                            if let Some(pickup) = entity.try_get_first_component_including_disabled::<ItemPickUpperComponent>(None)? {
                                pickup.set_only_pick_this_entity(Some(wand))?;
                            }
                            if gid.is_none() {
                                let var = wand.add_component::<VariableStorageComponent>()?;
                                var.set_name("ew_spawned_wand".into())?;
                            }
                            let quick = if let Some(quick) =
                                entity.children(None).iter().find_map(|a| {
                                    if a.name().ok()? == "inventory_quick" {
                                        a.children(None).iter().for_each(|e| e.kill());
                                        Some(a)
                                    } else {
                                        None
                                    }
                                }) {
                                *quick
                            } else {
                                let quick = entity_create_new(Some("inventory_quick".into()))?
                                    .wrap_err("unreachable")?;
                                entity.add_child(quick);
                                quick
                            };
                            quick.add_child(wand);
                            if let Some(ability) = wand
                                .try_get_first_component_including_disabled::<AbilityComponent>(
                                    None,
                                )?
                            {
                                ability.set_drop_as_item_on_death(false)?;
                            }
                            if let Some(item) = wand
                                .try_get_first_component_including_disabled::<ItemComponent>(None)?
                            {
                                item.set_remove_default_child_actions_on_death(true)?;
                                item.set_remove_on_death_if_empty(true)?;
                                item.set_remove_on_death(true)?;
                            }
                            //TODO set active item?
                        }
                    } else if let Some(inv) = entity
                        .children(None)
                        .iter()
                        .find(|e| e.name().unwrap_or("".into()) == "inventory_quick")
                    {
                        inv.children(None).iter().for_each(|e| e.kill())
                    }
                    if entity_info.is_enabled {
                        if entity
                            .iter_all_components_of_type_including_disabled::<VariableStorageComponent>(None)?
                            .all(|var| var.name().unwrap_or("".into()) != "ew_has_started")
                        {
                            entity.set_components_with_tag_enabled("enabled_at_start".into(), false)?;
                            entity.set_components_with_tag_enabled("disabled_at_start".into(), true)?;
                            entity.remove_all_components_of_type::<LuaComponent>()?;
                            entity
                                .add_component::<VariableStorageComponent>()?
                                .set_name("ew_has_started".into())?;
                            entity
                                .children(Some("protection".into()))
                                .iter()
                                .for_each(|ent| ent.kill());
                        } else if let Some(var) = entity.try_get_first_component_including_disabled::<VariableStorageComponent>(None)?
            .iter().find(|var| var.name().unwrap_or("".into()) == "active") {
                            var.set_value_int(1)?;
                            entity.set_components_with_tag_enabled("activate".into(), true)?
                        }
                    } else if let Some(var) = entity
                        .try_get_first_component_including_disabled::<VariableStorageComponent>(
                            None,
                        )?
                        .iter()
                        .find(|var| var.name().unwrap_or("".into()) == "active")
                    {
                        var.set_value_int(0)?;
                        entity.set_components_with_tag_enabled("activate".into(), false)?
                    }
                    for (ent, (x, y)) in entity
                        .children(None)
                        .iter()
                        .filter(|ent| ent.get_first_component::<IKLimbComponent>(None).is_ok())
                        .zip(&entity_info.limbs)
                    {
                        if let Ok(limb) = ent.get_first_component::<IKLimbComponent>(None) {
                            limb.set_end_position((*x, *y))?;
                        }
                        if let Ok(limb) = ent.get_first_component::<IKLimbWalkerComponent>(None) {
                            entity.remove_component(*limb)?
                        };
                    }
                    let m = *ctx.fps_by_player.get(&my_peer_id()).unwrap_or(&60) as f32
                        / *ctx.fps_by_player.get(&self.peer_id).unwrap_or(&60) as f32;
                    let (vx, vy) = (entity_info.vx * m, entity_info.vy * m);
                    if entity_info.phys.is_empty()
                        || (entity_info.is_enabled && entity.has_tag("boss_centipede"))
                    {
                        entity.set_position(entity_info.x, entity_info.y, Some(entity_info.r))?;
                        if let Some(vel) =
                            entity.try_get_first_component::<VelocityComponent>(None)?
                        {
                            vel.set_m_velocity((vx, vy))?;
                        }
                        if let Some(worm) =
                            entity.try_get_first_component::<BossDragonComponent>(None)?
                        {
                            worm.set_m_target_vec((vx, vy))?;
                        } else if let Some(worm) =
                            entity.try_get_first_component::<WormComponent>(None)?
                        {
                            worm.set_m_target_vec((vx, vy))?;
                        } else if let Some(vel) =
                            entity.try_get_first_component::<CharacterDataComponent>(None)?
                        {
                            vel.set_m_velocity((vx, vy))?;
                        }
                    }
                    if let Some(damage) =
                        entity.try_get_first_component::<DamageModelComponent>(None)?
                    {
                        let current_hp = damage.hp()? as f32;
                        if current_hp > entity_info.hp {
                            noita_api::raw::entity_inflict_damage(
                                entity.raw() as i32,
                                (current_hp - entity_info.hp) as f64,
                                "DAMAGE_CURSE".into(), //TODO should be enum
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
                                (p.vx * m).into(),
                                (p.vy * m).into(),
                                (p.av * m).into(),
                            )?;
                        }
                    }

                    if let Some(cost) = entity.try_get_first_component::<ItemCostComponent>(None)? {
                        cost.set_cost(entity_info.cost)?;
                    }

                    entity.set_game_effects(&entity_info.game_effects);

                    entity.set_current_stains(entity_info.current_stains)?;

                    if let Ok(sprites) = entity.iter_all_components_of_type::<SpriteComponent>(None)
                    {
                        for (sprite, animation) in sprites
                            .filter(|sprite| {
                                sprite
                                    .image_file()
                                    .map(|c| c.ends_with(".xml"))
                                    .unwrap_or(false)
                            })
                            .zip(entity_info.animations.iter())
                        {
                            if *animation == u16::MAX {
                                continue;
                            }
                            let file = sprite.image_file()?;
                            let text = noita_api::raw::mod_text_file_get_content(file)?;
                            let mut split = text.split("name=\"");
                            split.next();
                            let data: Vec<&str> =
                                split.filter_map(|piece| piece.split("\"").next()).collect();
                            if data.len() > *animation as usize {
                                sprite.set_rect_animation(data[*animation as usize].into())?;
                                sprite.set_next_rect_animation(data[*animation as usize].into())?;
                            }
                        }
                    }
                    let laser = entity.try_get_first_component::<LaserEmitterComponent>(None)?;
                    if let Some(peer) = entity_info.laser {
                        let laser = if let Some(laser) = laser {
                            laser
                        } else {
                            let laser = entity.add_component::<LaserEmitterComponent>()?;
                            laser.object_set_value::<i32>(
                                "laser",
                                "max_cell_durability_to_destroy",
                                0,
                            )?;
                            laser.object_set_value::<i32>("laser", "damage_to_cells", 0)?;
                            laser.object_set_value::<i32>("laser", "max_length", 1024)?;
                            laser.object_set_value::<i32>("laser", "beam_radius", 0)?;
                            laser.object_set_value::<i32>("laser", "beam_particle_chance", 75)?;
                            laser.object_set_value::<i32>("laser", "beam_particle_fade", 0)?;
                            laser.object_set_value::<i32>("laser", "hit_particle_chance", 0)?;
                            laser.object_set_value::<bool>("laser", "audio_enabled", false)?;
                            laser.object_set_value::<i32>("laser", "damage_to_entities", 0)?;
                            laser.object_set_value::<i32>("laser", "beam_particle_type", 225)?;
                            laser
                        };
                        if let Some(ent) = ctx.player_map.get_by_left(&peer) {
                            let (x, y) = entity.position()?;
                            let (tx, ty) = ent.position()?;
                            if !raytrace_platforms(x as f64, y as f64, tx as f64, ty as f64)?.0 {
                                laser.set_is_emitting(true)?;
                                let (dx, dy) = (tx - x, ty - y);
                                let theta = dy.atan2(dx);
                                laser.set_laser_angle_add_rad(theta - entity.rotation()?)?;
                                laser.object_set_value::<f32>(
                                    "laser",
                                    "max_length",
                                    dx.hypot(dy),
                                )?;
                            } else {
                                laser.set_is_emitting(false)?;
                            }
                        }
                    } else if let Some(laser) = laser {
                        laser.set_is_emitting(false)?;
                    }
                }
                None => {
                    let entity = spawn_entity_by_data(
                        &entity_info.spawn_info,
                        entity_info.x,
                        entity_info.y,
                    )?;
                    self.init_remote_entity(entity, *lid, entity_info.drops_gold)?;
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
            if let Some(inv) = entity
                .children(None)
                .iter()
                .find(|e| e.name().unwrap_or("".into()) == "inventory_quick")
            {
                inv.children(None).iter().for_each(|e| e.kill())
            }
            if let Some(damage) = entity.try_get_first_component::<DamageModelComponent>(None)? {
                damage.set_wait_for_kill_flag_on_death(false)?;
                damage.set_ui_report_damage(false)?;
                damage.set_hp(f64::MIN_POSITIVE)?;
                noita_api::raw::entity_inflict_damage(
                    entity.raw() as i32,
                    damage.hp()? + 0.1,
                    "DAMAGE_CURSE".into(), //TODO should be enum
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
    fn init_remote_entity(&self, entity: EntityID, lid: Lid, drops_gold: bool) -> eyre::Result<()> {
        entity.remove_all_components_of_type::<CameraBoundComponent>()?;
        entity.remove_all_components_of_type::<AnimalAIComponent>()?;
        entity.remove_all_components_of_type::<PhysicsAIComponent>()?;
        entity.remove_all_components_of_type::<AdvancedFishAIComponent>()?;
        entity.remove_all_components_of_type::<AIAttackComponent>()?;

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
                sprite.remove_tag("character")?
            }
        }

        if !drops_gold {
            for lua in entity.iter_all_components_of_type::<LuaComponent>(None)? {
                if lua.script_death().ok() == Some("data/scripts/items/drop_money.lua".into()) {
                    entity.remove_component(*lua)?;
                    break;
                }
            }
        }
        if let Some(pickup) =
            entity.try_get_first_component_including_disabled::<ItemPickUpperComponent>(None)?
        {
            pickup.set_drop_items_on_death(false)?;
            pickup.set_only_pick_this_entity(Some(EntityID(NonZero::new(1).unwrap())))?;
        }

        entity
            .iter_all_components_of_type_including_disabled::<VariableStorageComponent>(None)?
            .for_each(|var| {
                if var.name().unwrap_or("".into()) == "ew_gid_lid" {
                    let _ = entity.remove_component(*var);
                }
            });

        let var = entity.add_component::<VariableStorageComponent>()?;
        var.set_name("ew_gid_lid".into())?;
        if let Some(gid) = self.lid_to_gid.get(&lid) {
            var.set_value_string(gid.0.to_string().into())?;
        }
        var.set_value_int(i32::from_ne_bytes(lid.0.to_ne_bytes()))?;
        var.set_value_bool(false)?;
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
            component.add_tag(DES_SCRIPTS_TAG)?;
            component.add_tag("enabled_in_inventory")?;
            component.add_tag("enabled_in_world")?;
            component.add_tag("enabled_in_hand")?;
            Ok(component)
        })?;
    f(component)
}

/// If it's a wand, it might be in a pickup screen currently, and deleting it will crash the game.
fn _safe_wandkill(entity: EntityID) -> eyre::Result<()> {
    let lc = entity.add_component::<LuaComponent>()?;
    lc.set_script_source_file(
        "mods/quant.ew/files/system/entity_sync_helper/scripts/killself.lua".into(),
    )?;
    lc.set_execute_on_added(false)?;
    lc.set_m_next_execution_time(noita_api::raw::game_get_frame_num()? + 1)?;
    Ok(())
}

fn safe_entitykill(entity: EntityID) {
    let _ = entity.remove_all_components_of_type::<AudioComponent>();
    let is_wand = entity.try_get_first_component_including_disabled::<AbilityComponent>(None);
    if is_wand
        .map(|a| {
            a.map(|b| b.use_gun_script().unwrap_or(false))
                .unwrap_or(false)
        })
        .unwrap_or(false)
    {
        let _ = _safe_wandkill(entity);
    } else {
        if let Some(inv) = entity
            .children(None)
            .iter()
            .find(|e| e.name().unwrap_or("".into()) == "inventory_quick")
        {
            inv.children(None).iter().for_each(|e| e.kill())
        }
        entity.kill();
    }
}
