use super::NetManager;
use crate::{ephemerial, modules::ModuleCtx, my_peer_id, print_error};
use bimap::BiHashMap;
use eyre::{Context, ContextCompat, OptionExt, eyre};
use noita_api::raw::{entity_create_new, game_get_frame_num, raytrace_platforms};
use noita_api::serialize::{deserialize_entity, serialize_entity};
use noita_api::{
    AIAttackComponent, AbilityComponent, AdvancedFishAIComponent, AnimalAIComponent,
    AudioComponent, BossDragonComponent, BossHealthBarComponent, CameraBoundComponent,
    CharacterDataComponent, CharacterPlatformingComponent, DamageModelComponent, EntityID,
    ExplodeOnDamageComponent, GhostComponent, IKLimbAttackerComponent, IKLimbComponent,
    IKLimbWalkerComponent, IKLimbsAnimatorComponent, Inventory2Component, ItemComponent,
    ItemCostComponent, ItemPickUpperComponent, LaserEmitterComponent, LifetimeComponent,
    LuaComponent, PhysData, PhysicsAIComponent, PhysicsBody2Component, PhysicsBodyComponent,
    SpriteComponent, StreamingKeepAliveComponent, VariableStorageComponent, VelocityComponent,
    WormComponent, game_print,
};
use rustc_hash::{FxHashMap, FxHashSet};
use shared::des::{
    GLOBAL_AUTHORITY_RADIUS, GLOBAL_TRANSFER_RADIUS, TRANSFER_RADIUS, Target, UpdateOrUpload,
};
use shared::{
    GameEffectData, GameEffectEnum, NoitaOutbound, PeerId, SpawnOnce, WorldPos,
    des::{
        AUTHORITY_RADIUS, EntityInfo, EntityKind, EntitySpawnInfo, EntityUpdate, FullEntityData,
        Gid, Lid, PhysBodyInfo, ProjectileFired, UpdatePosition,
    },
};
use std::borrow::Cow;
use std::num::NonZero;
use std::time::Instant;
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
    pending_death_notify: Vec<(EntityID, bool, WorldPos, String, Option<EntityID>)>,
    global_entities: FxHashSet<EntityID>,
    got_polied: FxHashSet<Gid>,
}

pub(crate) struct LocalDiffModel {
    next_lid: Lid,
    entity_entries: FxHashMap<Lid, EntityEntryPair>,
    tracker: LocalDiffModelTracker,
    upload: FxHashSet<Lid>,
    dont_upload: FxHashSet<Lid>,
    enable_later: Vec<EntityID>,
    dont_save: FxHashSet<Lid>,
    phys_later: Vec<(EntityID, Vec<Option<PhysBodyInfo>>)>,
    wait_to_transfer: u8,
}
impl LocalDiffModel {
    /*pub(crate) fn get_lids(&self) -> Vec<Lid> {
        self.tracker.tracked.left_values().cloned().collect()
    }*/
    pub(crate) fn dont_save(&mut self, lid: Lid) {
        self.dont_save.insert(lid);
    }
    pub(crate) fn got_polied(&mut self, gid: Gid) {
        self.tracker.got_polied(gid);
    }
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
    pub(crate) fn get_pos_data(&mut self, frame_num: usize) -> Vec<UpdateOrUpload> {
        let len = self.entity_entries.len();
        let batch_size = (len / 60).max(1);
        let start = (frame_num % 60) * batch_size;
        let end = (start + batch_size).min(len);
        let mut upload = std::mem::take(&mut self.upload);
        let mut res: Vec<UpdateOrUpload> = self
            .entity_entries
            .iter()
            .skip(start)
            .take(end - start)
            .filter_map(|(lid, p)| {
                let EntityEntryPair { current, gid, last } = p;
                if last.is_some() && !self.dont_save.contains(lid) {
                    Some(if upload.remove(lid) && !self.dont_upload.contains(lid) {
                        UpdateOrUpload::Upload(FullEntityData {
                            gid: *gid,
                            pos: WorldPos::from_f32(current.x, current.y),
                            data: current.spawn_info.clone(),
                            wand: current.wand.clone().map(|(_, w, _)| w),
                            //rotation: entry_pair.current.r,
                            drops_gold: current.drops_gold,
                            is_charmed: current.is_charmed(),
                            hp: current.hp,
                            counter: current.counter,
                            phys: current.phys.clone(),
                            synced_var: current.synced_var.clone(),
                        })
                    } else {
                        UpdateOrUpload::Update(UpdatePosition {
                            gid: *gid,
                            pos: WorldPos::from_f32(current.x, current.y),
                            counter: current.counter,
                            is_charmed: current.is_charmed(),
                            hp: current.hp,
                            phys: current.phys.clone(),
                            synced_var: current.synced_var.clone(),
                        })
                    })
                } else {
                    None
                }
            })
            .collect();
        for lid in upload {
            if let Some(EntityEntryPair { current, gid, last }) = self.entity_entries.get(&lid) {
                if !self.dont_upload.contains(&lid) {
                    if last.is_some() {
                        res.push(UpdateOrUpload::Upload(FullEntityData {
                            gid: *gid,
                            pos: WorldPos::from_f32(current.x, current.y),
                            data: current.spawn_info.clone(),
                            wand: current.wand.clone().map(|(_, w, _)| w),
                            //rotation: entry_pair.current.r,
                            drops_gold: current.drops_gold,
                            is_charmed: current.is_charmed(),
                            hp: current.hp,
                            counter: current.counter,
                            phys: current.phys.clone(),
                            synced_var: current.synced_var.clone(),
                        }));
                    } else {
                        self.upload.insert(lid);
                    }
                }
            }
        }
        res
    }

    pub(crate) fn is_entity_tracked(&self, entity: EntityID) -> bool {
        self.tracker.tracked.contains_right(&entity)
    }
}
pub(crate) struct RemoteDiffModel {
    tracked: BiHashMap<Lid, EntityID>,
    entity_infos: FxHashMap<Lid, EntityInfo>,
    lid_to_gid: FxHashMap<Lid, Gid>,
    waiting_for_lid: FxHashMap<Gid, EntityID>,
    /// Entities that we want to track again. Typically when we move authority locally from a different peer.
    //backtrack: Vec<EntityID>,
    grab_request: Vec<Lid>,
    pending_remove: Vec<Lid>,
    pending_death_notify: Vec<(Lid, bool, Option<PeerId>)>,
    peer_id: PeerId,
}

impl RemoteDiffModel {
    /*pub fn check_entities(&mut self, lids: Vec<Lid>) {
        let to_remove: Vec<Lid> = self
            .tracked
            .left_values()
            .filter(|l| !lids.contains(l) && !self.pending_remove.contains(l))
            .cloned()
            .collect();
        self.pending_remove.extend(to_remove)
    }*/
    pub fn new(peer_id: PeerId) -> Self {
        Self {
            tracked: Default::default(),
            entity_infos: Default::default(),
            lid_to_gid: Default::default(),
            waiting_for_lid: Default::default(),
            //backtrack: Default::default(),
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
    pub(crate) fn remove_entities(&self) {
        for (_, ent) in self.tracked.iter() {
            safe_entitykill(*ent)
        }
    }
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
                global_entities: Default::default(),
                got_polied: Default::default(),
            },
            upload: Default::default(),
            dont_upload: Default::default(),
            enable_later: Default::default(),
            phys_later: Default::default(),
            dont_save: Default::default(),
            wait_to_transfer: 0,
        }
    }
}

impl LocalDiffModelTracker {
    pub(crate) fn got_polied(&mut self, gid: Gid) {
        self.got_polied.insert(gid);
    }

    #[allow(clippy::too_many_arguments)]
    fn update_entity(
        &mut self,
        ctx: &mut ModuleCtx,
        gid: Gid,
        info: &mut EntityInfo,
        lid: Lid,
        cam_pos: (f32, f32),
        do_upload: bool,
        should_transfer: bool,
        ignore_transfer: bool,
    ) -> eyre::Result<bool> {
        let entity = self
            .entity_by_lid(lid)
            .wrap_err_with(|| eyre!("Failed to grab update info for {:?} {:?}", gid, lid))?;

        if !entity.is_alive() {
            if self.got_polied.remove(&gid) {
                self.pending_removal.push(lid);
            } else if self.global_entities.remove(&entity)
                && self
                    .pending_death_notify
                    .iter()
                    .all(|(e, _, _, _, _)| *e != entity)
            {
                self._release_authority_update_data(ctx, gid, lid, info, do_upload)?;
                self.pending_removal.push(lid);
                ctx.net.send(&NoitaOutbound::DesToProxy(
                    shared::des::DesToProxy::ReleaseAuthority(gid),
                ))?;
                return Ok(do_upload);
            } else {
                self.untrack_entity(ctx, gid, lid, None)?;
            }
            return Ok(false);
        }
        let item_and_was_picked = info.kind == EntityKind::Item && item_in_inventory(entity)?;
        if item_and_was_picked && not_in_player_inventory(entity)? {
            self.temporary_untrack_item(ctx, gid, lid, entity)?;
            return Ok(false);
        }

        let (x, y, r, sx, sy) = entity.transform()?;
        let should_send_position =
            if let Some(com) = entity.try_get_first_component::<ItemComponent>(None)? {
                !com.play_hover_animation()?
            } else {
                true
            };

        if should_send_position {
            (info.x, info.y) = (x, y);
        }

        let should_send_rotation =
            if let Some(com) = entity.try_get_first_component::<ItemComponent>(None)? {
                !com.play_spinning_animation()? || com.play_hover_animation()?
            } else {
                true
            };

        if should_send_rotation {
            info.r = r
        }

        if let Some(inv) =
            entity.try_get_first_component_including_disabled::<Inventory2Component>(None)?
        {
            if let Some(wand) = inv.m_actual_active_item()? {
                if info.wand.is_none() {
                    info.wand = if wand.is_alive() {
                        wand.remove_all_components_of_type::<LuaComponent>(Some(
                            "ew_immortal".into(),
                        ))?;
                        let r = wand.rotation()?;
                        info.wand_rotation = r;
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
                            Some((Some(Gid(gid)), serialize_entity(wand)?, wand.raw()))
                        } else {
                            Some((None, serialize_entity(wand)?, wand.raw()))
                        }
                    } else {
                        None
                    }
                } else if let Some((_, _, ent)) = info.wand {
                    let ent = EntityID::try_from(ent)?;
                    if ent != wand {
                        info.wand = None
                    } else {
                        let r = wand.rotation()?;
                        info.wand_rotation = r;
                    }
                }
            } else {
                info.wand = None;
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
                })
            || (entity.has_tag("pitcheck_b")
                && entity
                    .try_get_first_component::<LuaComponent>(Some("disabled".into()))?
                    .is_some());

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

        if let Some(worm) = entity.try_get_first_component::<BossDragonComponent>(None)? {
            (info.vx, info.vy) = worm.m_target_vec()?;
        } else if let Some(worm) = entity.try_get_first_component::<WormComponent>(None)? {
            (info.vx, info.vy) = worm.m_target_vec()?;
        } else if let Some(vel) = entity.try_get_first_component::<CharacterDataComponent>(None)? {
            (info.vx, info.vy) = vel.m_velocity()?;
        } else if let Some(vel) = entity.try_get_first_component::<VelocityComponent>(None)? {
            (info.vx, info.vy) = vel.m_velocity()?;
        }

        if entity.has_tag("card_action") {
            if let Some(vel) = entity.try_get_first_component::<VelocityComponent>(None)? {
                let (cx, cy) = noita_api::raw::game_get_camera_pos()?;
                if (cx as f32 - x).powi(2) + (cy as f32 - y).powi(2) > 512.0 * 512.0 {
                    vel.set_gravity_y(0.0)?;
                    vel.set_air_friction(10.0)?;
                } else {
                    vel.set_gravity_y(400.0)?;
                    vel.set_air_friction(0.55)?;
                }
            }
        }

        if let Some(damage) = entity.try_get_first_component::<DamageModelComponent>(None)? {
            let hp = damage.hp()?;
            info.hp = hp as f32;
        }

        if entity.check_all_phys_init()? {
            info.phys = collect_phys_info(entity)?;
        }

        if let Some(item_cost) = entity.try_get_first_component::<ItemCostComponent>(None)? {
            info.cost = item_cost.cost()?;
        } else if entity.has_tag("boss_wizard") {
            info.cost = game_get_frame_num()? as i64;
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
                        if let Ok(v) = ent.get_var_or_default("ew_frame_num") {
                            let _ = v.add_tag("ew_frame_num");
                            let _ = v.set_value_int(info.cost as i32);
                        }
                        Some(1 << var.value_int().ok()?)
                    } else {
                        None
                    }
                })
                .sum();
        } else {
            info.cost = 0;
        }
        if entity.has_tag("seed_d") {
            let essences = entity.get_var_or_default("sunbaby_essences_list")?;
            let sprite =
                entity.get_first_component::<SpriteComponent>(Some("sunbaby_sprite".into()))?;
            let sprite = sprite.image_file()?;
            let num: u8 = match sprite.to_string().as_str() {
                "data/props_gfx/sun_small_purple.png" => 0,
                "data/props_gfx/sun_small_red.png" => 1,
                "data/props_gfx/sun_small_blue.png" => 2,
                "data/props_gfx/sun_small_green.png" => 3,
                "data/props_gfx/sun_small_orange.png" => 4,
                _ => 5,
            };
            info.counter = essences
                .value_string()?
                .split(',')
                .filter_map(|s| match s {
                    "water" => Some(1),
                    "fire" => Some(2),
                    "air" => Some(4),
                    "earth" => Some(8),
                    "poop" => Some(16),
                    _ => None,
                })
                .sum::<u8>()
                + (num * 32);
        }

        info.game_effects = entity
            .get_game_effects()?
            .iter()
            .map(|(e, _)| e.clone())
            .collect::<Vec<GameEffectData>>();

        info.current_stains = if let Some(var) = entity.get_var("rolling") {
            if var.value_int()? == 0 {
                let rng = rand::random::<i32>();
                let var = entity.get_var_or_default("ew_rng")?;
                var.set_value_int(rng)?;
                let bytes = rng.to_le_bytes();
                u64::from_le_bytes([0, 0, 0, 0, bytes[0], bytes[1], bytes[2], bytes[3]])
            } else {
                let bytes = info.current_stains.to_le_bytes();
                if bytes[0] == 0 {
                    u64::from_le_bytes([1, 0, 0, 0, bytes[4], bytes[5], bytes[6], bytes[7]])
                } else {
                    info.current_stains
                }
            }
        } else {
            entity.get_current_stains()?
        };

        let mut any = false;
        for ai in
            entity.iter_all_components_of_type_including_disabled::<AIAttackComponent>(None)?
        {
            any = any || ai.attack_ranged_aim_rotation_enabled()?;
        }
        for ai in
            entity.iter_all_components_of_type_including_disabled::<AnimalAIComponent>(None)?
        {
            any = any || ai.attack_ranged_aim_rotation_enabled()?;
        }
        if any {
            if let Some(ai) =
                entity.try_get_first_component_including_disabled::<AnimalAIComponent>(None)?
            {
                info.ai_state = ai.ai_state()?;
                info.ai_rotation = ai.m_ranged_attack_current_aim_angle()?;
            }
        } else if let Ok(sprites) = entity.iter_all_components_of_type::<SpriteComponent>(None) {
            info.facing_direction = (sx.is_sign_positive(), sy.is_sign_positive());
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
            if let Some(ai) = entity.try_get_first_component::<AnimalAIComponent>(None)? {
                if ai.attack_ranged_use_laser_sight()? && !ai.is_static_turret()? {
                    info.laser = if let Some(target) = ai.m_greatest_prey()? {
                        if ![15, 16].contains(&ai.ai_state()?) {
                            Target::None
                        } else if let Some(peer) = ctx.player_map.get_by_right(&target) {
                            Target::Peer(*peer)
                        } else if let Some(var) = target.get_var("ew_gid_lid") {
                            if var.value_bool()? {
                                Target::Gid(Gid(var.value_string()?.parse::<u64>()?))
                            } else {
                                Target::None
                            }
                        } else {
                            Target::None
                        }
                    } else {
                        Target::None
                    }
                }
            }
        } else {
            info.animations.clear()
        }

        info.synced_var = entity
            .iter_all_components_of_type_including_disabled::<VariableStorageComponent>(Some(
                "ew_synced_var".into(),
            ))?
            .filter_map(|a| {
                Some((
                    a.name().ok()?.to_string(),
                    a.value_string().ok()?.to_string(),
                    a.value_int().ok()?,
                    a.value_float().ok()?,
                    a.value_bool().ok()?,
                ))
            })
            .collect::<Vec<(String, String, i32, f32, bool)>>();
        if ignore_transfer {
            // Check if entity went out of range, remove and release authority if it did.
            let is_beyond_authority = (x - cam_pos.0).powi(2) + (y - cam_pos.1).powi(2)
                > if info.is_global {
                    GLOBAL_AUTHORITY_RADIUS
                } else {
                    AUTHORITY_RADIUS
                }
                .powi(2);
            if is_beyond_authority || should_transfer {
                if let Some(peer) = ctx.locate_player_within_except_me(
                    x as i32,
                    y as i32,
                    if info.is_global {
                        GLOBAL_TRANSFER_RADIUS
                    } else {
                        TRANSFER_RADIUS
                    },
                )? {
                    self.transfer_authority_to(ctx, gid, lid, peer, info, do_upload)
                        .wrap_err("Failed to transfer authority")?;
                    return Ok(do_upload);
                } else if !info.is_global && is_beyond_authority {
                    self.release_authority(ctx, gid, lid, info, do_upload)
                        .wrap_err("Failed to release authority")?;
                    return Ok(do_upload);
                }
            }
        }
        if let Some(var) = entity.get_var("ew_was_stealable") {
            let n = var.value_int()?;
            if n == 1 {
                if let Some(cost) = entity.try_get_first_component::<ItemCostComponent>(None)? {
                    let (cx, cy) = noita_api::raw::game_get_camera_pos()?;
                    if (cx as f32 - x).powi(2) + (cy as f32 - y).powi(2) < 256.0 * 256.0 {
                        cost.set_stealable(true)?;
                        entity.remove_component(*var)?;
                    }
                }
                if let Some(vel) = entity.try_get_first_component::<VelocityComponent>(None)? {
                    vel.set_gravity_y(400.0)?;
                    vel.set_air_friction(0.55)?;
                }
            } else if n == 0 {
                var.set_value_int(32)?;
                if let Some(vel) = entity.try_get_first_component::<VelocityComponent>(None)? {
                    vel.set_gravity_y(0.0)?;
                    vel.set_air_friction(10.0)?;
                }
            } else {
                var.set_value_int(n - 1)?;
                if let Some(vel) = entity.try_get_first_component::<VelocityComponent>(None)? {
                    vel.set_gravity_y(0.0)?;
                    vel.set_air_friction(10.0)?;
                }
            }
        }
        Ok(false)
    }
    fn untrack_entity(
        &mut self,
        ctx: &mut ModuleCtx<'_>,
        gid: Gid,
        lid: Lid,
        ent: Option<NonZero<isize>>,
    ) -> Result<(), eyre::Error> {
        self.pending_removal.push(lid);
        ctx.net.send(&NoitaOutbound::DesToProxy(
            shared::des::DesToProxy::DeleteEntity(gid, ent),
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
        self.untrack_entity(ctx, gid, lid, Some(entity.0))?;
        entity.remove_tag(DES_TAG)?;
        with_entity_scripts(entity, |luac| {
            luac.set_script_throw_item(
                "mods/quant.ew/files/system/entity_sync_helper/item_notify.lua".into(),
            )
        })?;
        for entity in entity.children(None) {
            with_entity_scripts(entity, |luac| {
                luac.set_script_throw_item(
                    "mods/quant.ew/files/system/entity_sync_helper/item_notify.lua".into(),
                )
            })?;
        }
        Ok(())
    }

    fn entity_by_lid(&self, lid: Lid) -> eyre::Result<EntityID> {
        Ok(*self
            .tracked
            .get_by_left(&lid)
            .ok_or_eyre("Expected to find a corresponding entity")?)
    }

    #[allow(clippy::too_many_arguments)]
    fn _release_authority_update_data(
        &mut self,
        ctx: &mut ModuleCtx<'_>,
        gid: Gid,
        lid: Lid,
        info: &EntityInfo,
        do_upload: bool,
    ) -> Result<EntityID, eyre::Error> {
        let entity = self
            .entity_by_lid(lid)
            .wrap_err("Failed to release authority and upload update data")?;
        if !entity
            .filename()?
            .starts_with("data/entities/animals/wand_ghost")
        {
            ctx.net.send(&NoitaOutbound::DesToProxy(
                shared::des::DesToProxy::UpdateWand(gid, info.wand.clone().map(|(_, w, _)| w)),
            ))?;
        }
        ctx.net.send(&NoitaOutbound::DesToProxy(
            shared::des::DesToProxy::UpdatePosition(if do_upload {
                UpdateOrUpload::Upload(FullEntityData {
                    gid,
                    pos: WorldPos::from_f32(info.x, info.y),
                    data: info.spawn_info.clone(),
                    wand: info.wand.clone().map(|(_, w, _)| w),
                    //rotation: entry_pair.info.r,
                    drops_gold: info.drops_gold,
                    is_charmed: info.is_charmed(),
                    hp: info.hp,
                    counter: info.counter,
                    phys: info.phys.clone(),
                    synced_var: info.synced_var.clone(),
                })
            } else {
                UpdateOrUpload::Update(UpdatePosition {
                    gid,
                    pos: WorldPos::from_f32(info.x, info.y),
                    counter: info.counter,
                    is_charmed: info.is_charmed(),
                    hp: info.hp,
                    phys: info.phys.clone(),
                    synced_var: info.synced_var.clone(),
                })
            }),
        ))?;
        Ok(entity)
    }

    #[allow(clippy::too_many_arguments)]
    fn release_authority(
        &mut self,
        ctx: &mut ModuleCtx<'_>,
        gid: Gid,
        lid: Lid,
        info: &EntityInfo,
        do_upload: bool,
    ) -> eyre::Result<()> {
        let entity = self._release_authority_update_data(ctx, gid, lid, info, do_upload)?;
        ctx.net.send(&NoitaOutbound::DesToProxy(
            shared::des::DesToProxy::ReleaseAuthority(gid),
        ))?;
        self.pending_removal.push(lid);
        safe_entitykill(entity);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn transfer_authority_to(
        &mut self,
        ctx: &mut ModuleCtx<'_>,
        gid: Gid,
        lid: Lid,
        peer: PeerId,
        info: &EntityInfo,
        do_upload: bool,
    ) -> eyre::Result<()> {
        let entity = self._release_authority_update_data(ctx, gid, lid, info, do_upload)?;
        ctx.net.send(&NoitaOutbound::DesToProxy(
            shared::des::DesToProxy::TransferAuthorityTo(gid, peer),
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
        self.wait_to_transfer = 16;
        let lid = self.alloc_lid();
        let should_not_serialize = entity
            .remove_all_components_of_type::<CameraBoundComponent>(None)?
            || (entity.is_alive() && entity.check_all_phys_init()? && noita_api::raw::physics_body_id_get_from_entity(entity, None)
                .unwrap_or_default()
                .len()
                == entity
                    .iter_all_components_of_type_including_disabled::<PhysicsBodyComponent>(None)
                    .iter()
                    .len()
                    + entity
                        .iter_all_components_of_type_including_disabled::<PhysicsBody2Component>(
                            None,
                        )
                        .iter()
                        .len());
        entity.add_tag(DES_TAG)?;
        if let Some(ghost) =
            entity.try_get_first_component_including_disabled::<GhostComponent>(None)?
        {
            ghost.set_target_tag("".into())?;
        }

        self.tracker.tracked.insert(lid, entity);

        let (x, y) = entity.position()?;

        if entity.has_tag("card_action") {
            if let Some(cost) = entity.try_get_first_component::<ItemCostComponent>(None)? {
                if cost.stealable()? {
                    cost.set_stealable(false)?;
                    entity.get_var_or_default("ew_was_stealable")?;
                }
            }
        }

        let entity_kind = classify_entity(entity)?;
        let spawn_info = match entity_kind {
            EntityKind::Normal if should_not_serialize => {
                EntitySpawnInfo::Filename(entity.filename()?)
            }
            _ => EntitySpawnInfo::Serialized {
                //serialized_at: game_get_frame_num()?,
                data: serialize_entity(entity)?, //TODO we never update this?
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
        var.set_value_int(i32::from_le_bytes(lid.0.to_le_bytes()))?;
        var.set_value_bool(true)?;

        if entity.has_tag("card_action") {
            if let Some(vel) = entity.try_get_first_component::<VelocityComponent>(None)? {
                vel.set_gravity_y(0.0)?;
                vel.set_air_friction(10.0)?;
            }
        }

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

        if is_global {
            self.tracker.global_entities.insert(entity);
        }

        let drops_gold = (entity
            .iter_all_components_of_type::<LuaComponent>(None)?
            .any(|lua| {
                lua.script_death().ok() == Some("data/scripts/items/drop_money.lua".into())
            })
            && entity
                .iter_all_components_of_type::<VariableStorageComponent>(None)?
                .all(|var| !var.has_tag("no_gold_drop")))
            || (entity.has_tag("boss_dragon")
                && entity
                    .iter_all_components_of_type::<LuaComponent>(None)?
                    .any(|lua| {
                        lua.script_death().ok()
                            == Some("data/scripts/animals/boss_dragon_death.lua".into())
                    }))
            || entity
                .get_var("throw_time")
                .map(|v| v.value_int().ok() != Some(-1))
                .unwrap_or(false);

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
                    game_effects: Vec::new(),
                    current_stains: 0,
                    facing_direction: (false, false),
                    animations: Vec::new(),
                    wand: None,
                    wand_rotation: 0.0,
                    is_global,
                    drops_gold,
                    laser: Default::default(),
                    ai_rotation: 0.0,
                    ai_state: 0,
                    limbs: Vec::new(),
                    is_enabled: false,
                    counter: 0,
                    synced_var: Vec::new(),
                },
                gid,
            },
        );

        Ok(lid)
    }

    pub(crate) fn track_and_upload_entity(&mut self, entity: EntityID) -> eyre::Result<()> {
        let gid = Gid(rand::random());
        let lid = self.track_entity(entity, gid)?;
        self.upload.insert(lid);
        Ok(())
    }

    pub(crate) fn phys_later(&mut self) -> eyre::Result<()> {
        for (entity, phys) in self.phys_later.drain(..) {
            if entity.is_alive() && entity.check_all_phys_init()? {
                let phys_bodies = noita_api::raw::physics_body_id_get_from_entity(entity, None)
                    .unwrap_or_default();
                for (p, physics_body_id) in phys.iter().zip(phys_bodies.iter()) {
                    let Some(p) = p else {
                        continue;
                    };
                    let (x, y) =
                        noita_api::raw::game_pos_to_physics_pos(p.x.into(), Some(p.y.into()))?;
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
        }
        Ok(())
    }

    pub(crate) fn enable_later(&mut self) -> eyre::Result<()> {
        for entity in self.enable_later.drain(..) {
            if entity.is_alive() {
                entity.set_components_with_tag_enabled("disabled_at_start".into(), true)?;
                entity.set_components_with_tag_enabled("enabled_at_start".into(), false)?;
                entity
                    .children(Some("protection".into()))
                    .iter()
                    .for_each(|ent| ent.kill());
                entity.add_tag("boss_centipede_active")?;
                noita_api::raw::physics_set_static(entity, false)?;
            }
        }
        Ok(())
    }

    pub(crate) fn update_pending_authority(&mut self) -> eyre::Result<u128> {
        let start = Instant::now();
        while let Some(entity_data) = self.tracker.pending_authority.pop() {
            let entity = spawn_entity_by_data(
                &entity_data.data,
                entity_data.pos.x as f32,
                entity_data.pos.y as f32,
            )?;
            entity.set_position(entity_data.pos.x as f32, entity_data.pos.y as f32, None)?;
            if entity_data.is_charmed {
                if entity.has_tag("boss_centipede") {
                    self.enable_later.push(entity);
                } else if entity.has_tag("pitcheck_b") {
                    entity.set_components_with_tag_enabled("disabled".into(), true)?;
                } else if let Some(var) = entity
                    .try_get_first_component_including_disabled::<VariableStorageComponent>(None)?
                    .iter()
                    .find(|var| var.name().unwrap_or("".into()) == "active")
                {
                    var.set_value_int(1)?;
                    entity.set_components_with_tag_enabled("activate".into(), true)?
                } else {
                    entity.set_game_effects(&[GameEffectData::Normal(GameEffectEnum::Charm)])?
                }
            }
            for (name, s, i, f, b) in &entity_data.synced_var {
                let v = entity.get_var_or_default(name)?;
                v.set_value_string(s.into())?;
                v.set_value_int(*i)?;
                v.set_value_float(*f)?;
                v.set_value_bool(*b)?;
            }
            if !entity_data.phys.is_empty() {
                self.phys_later.push((entity, entity_data.phys));
            }

            mom(entity, entity_data.counter, None)?;
            sun(entity, entity_data.counter)?;
            if entity_data.hp != -1.0 {
                if let Some(damage) =
                    entity.try_get_first_component::<DamageModelComponent>(None)?
                {
                    if entity_data.hp > damage.max_hp_cap()? as f32 {
                        damage.set_max_hp_cap(entity_data.hp as f64)?;
                    }
                    if entity_data.hp > damage.max_hp()? as f32 {
                        damage.set_max_hp(entity_data.hp as f64)?;
                    }
                    damage.set_hp(entity_data.hp as f64)?;
                }
            }
            if !entity_data.drops_gold {
                for lua in entity.iter_all_components_of_type::<LuaComponent>(None)? {
                    if lua.script_death().ok() == Some("data/scripts/items/drop_money.lua".into()) {
                        entity.remove_component(*lua)?
                    }
                }
            } else if entity.has_tag("boss_dragon") {
                let lua = entity.add_component::<LuaComponent>()?;
                lua.set_script_death("data/scripts/animals/boss_dragon_death.lua".into())?;
                lua.set_execute_every_n_frame(-1)?;
            }
            if let Some(wand) = entity_data.wand {
                give_wand(entity, &wand, None, false, None)?;
            }
            let lid = self.track_entity(entity, entity_data.gid)?;
            self.dont_upload.insert(lid);

            // Don't handle too much in one frame to avoid stutters.
            if start.elapsed().as_micros() > 1000 {
                break;
            }
        }
        Ok(start.elapsed().as_micros())
    }

    #[allow(clippy::type_complexity)]
    pub(crate) fn update_tracked_entities(
        &mut self,
        ctx: &mut ModuleCtx,
        start: usize,
        time: u128,
    ) -> eyre::Result<(Vec<EntityUpdate>, Vec<(WorldPos, SpawnOnce)>, u128, usize)> {
        let (cam_x, cam_y) = noita_api::raw::game_get_camera_pos()?;
        let cam_x = cam_x as f32;
        let cam_y = cam_y as f32;
        let mut res = Vec::new();
        let mut should_transfer = false;
        if let Some(pe) = ctx.player_map.get_by_left(&my_peer_id()) {
            let (px, py) = pe.position()?;
            should_transfer =
                (px - cam_x).powi(2) + (py - cam_y).powi(2) > AUTHORITY_RADIUS.powi(2);
        }
        if !should_transfer {
            self.wait_to_transfer = 16
        } else {
            self.wait_to_transfer = self.wait_to_transfer.saturating_sub(1)
        }
        let l = self.entity_entries.len();
        let tmr = Instant::now();
        let mut end = 0;
        let start = if start >= l { 0 } else { start };
        for (i, (&lid, EntityEntryPair { last, current, gid })) in
            self.entity_entries.iter_mut().skip(start).enumerate()
        {
            match self
                .tracker
                .update_entity(
                    ctx,
                    *gid,
                    current,
                    lid,
                    (cam_x, cam_y),
                    self.upload.contains(&lid) && !self.dont_upload.contains(&lid),
                    should_transfer && self.wait_to_transfer == 0,
                    !self.dont_save.contains(&lid),
                )
                .wrap_err("Failed to update local entity")
            {
                Err(error) => {
                    print_error(error)?;
                    self.tracker.untrack_entity(ctx, *gid, lid, None)?;
                }
                Ok(do_remove) => {
                    if do_remove {
                        self.upload.remove(&lid);
                    }
                    let Some(last) = last.as_mut() else {
                        *last = Some(current.clone());
                        res.push(EntityUpdate::Init(Box::from(current.clone()), lid, *gid));
                        continue;
                    };
                    let mut had_any_delta = false;

                    fn diff<T: PartialEq + Clone>(
                        current: &T,
                        last: &mut T,
                        update: EntityUpdate,
                        res: &mut Vec<EntityUpdate>,
                        had_any_delta: &mut bool,
                        lid: Lid,
                    ) {
                        if current != last {
                            if !*had_any_delta {
                                *had_any_delta = true;
                                res.push(EntityUpdate::CurrentEntity(lid));
                            }
                            res.push(update);
                            *last = current.clone();
                        }
                    }
                    if current.wand.clone().map(|(_, _, g)| g)
                        != last.wand.clone().map(|(_, _, g)| g)
                    {
                        had_any_delta = true;
                        res.push(EntityUpdate::CurrentEntity(lid));
                        res.push(EntityUpdate::SetWand(current.wand.clone()));
                        last.wand = current.wand.clone();
                    }
                    diff(
                        &current.wand_rotation,
                        &mut last.wand_rotation,
                        EntityUpdate::SetWandRotation(current.wand_rotation),
                        &mut res,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.laser,
                        &mut last.laser,
                        EntityUpdate::SetLaser(current.laser),
                        &mut res,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &(current.x, current.y),
                        &mut (last.x, last.y),
                        EntityUpdate::SetPosition(current.x, current.y),
                        &mut res,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &(current.vx, current.vy),
                        &mut (last.vx, last.vy),
                        EntityUpdate::SetVelocity(current.vx, current.vy),
                        &mut res,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.hp,
                        &mut last.hp,
                        EntityUpdate::SetHp(current.hp),
                        &mut res,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.animations,
                        &mut last.animations,
                        EntityUpdate::SetAnimations(current.animations.clone()),
                        &mut res,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.synced_var,
                        &mut last.synced_var,
                        EntityUpdate::SetSyncedVar(current.synced_var.clone()),
                        &mut res,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.facing_direction,
                        &mut last.facing_direction,
                        EntityUpdate::SetFacingDirection(current.facing_direction),
                        &mut res,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.r,
                        &mut last.r,
                        EntityUpdate::SetRotation(current.r),
                        &mut res,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.phys,
                        &mut last.phys,
                        EntityUpdate::SetPhysInfo(current.phys.clone()),
                        &mut res,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.cost,
                        &mut last.cost,
                        EntityUpdate::SetCost(current.cost),
                        &mut res,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.current_stains,
                        &mut last.current_stains,
                        EntityUpdate::SetStains(current.current_stains),
                        &mut res,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.game_effects,
                        &mut last.game_effects,
                        EntityUpdate::SetGameEffects(current.game_effects.clone()),
                        &mut res,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.ai_rotation,
                        &mut last.ai_rotation,
                        EntityUpdate::SetAiRotation(current.ai_rotation),
                        &mut res,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.ai_state,
                        &mut last.ai_state,
                        EntityUpdate::SetAiState(current.ai_state),
                        &mut res,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.limbs,
                        &mut last.limbs,
                        EntityUpdate::SetLimbs(current.limbs.clone()),
                        &mut res,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.is_enabled,
                        &mut last.is_enabled,
                        EntityUpdate::SetIsEnabled(current.is_enabled),
                        &mut res,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.counter,
                        &mut last.counter,
                        EntityUpdate::SetCounter(current.counter),
                        &mut res,
                        &mut had_any_delta,
                        lid,
                    );
                }
            }
            if time + tmr.elapsed().as_micros() > 3000 {
                end = (start + i + 1) % l;
                break;
            }
        }
        for (lid, peer) in self.tracker.pending_localize.drain(..) {
            res.push(EntityUpdate::LocalizeEntity(lid, peer));
        }

        let mut dead = Vec::new();
        for (killed, wait_on_kill, pos, file, responsible) in
            self.tracker.pending_death_notify.drain(..)
        {
            let responsible_peer = responsible
                .and_then(|ent| ctx.player_map.get_by_right(&ent))
                .copied();
            let Some(lid) = self.tracker.tracked.get_by_right(&killed).copied() else {
                continue;
            };
            let drops_gold = if let Some(info) = self.entity_entries.get(&lid) {
                info.current.drops_gold
            } else {
                false
            };
            res.push(EntityUpdate::KillEntity {
                lid,
                wait_on_kill,
                responsible_peer,
            });
            dead.push((pos, SpawnOnce::Enemy(file, drops_gold, responsible_peer)));
            self.tracker.global_entities.remove(&killed);
        }

        for lid in self.tracker.pending_removal.drain(..) {
            res.push(EntityUpdate::RemoveEntity(lid));
            // "Untrack" entity
            self.tracker.tracked.remove_by_left(&lid);
            self.entity_entries.remove(&lid);
            self.upload.remove(&lid);
            self.dont_save.remove(&lid);
        }
        Ok((res, dead, time + tmr.elapsed().as_micros(), end))
    }
    pub(crate) fn make_init(&mut self) -> Vec<EntityUpdate> {
        let mut res = Vec::new();
        for (lid, EntityEntryPair { current, gid, .. }) in self.entity_entries.iter() {
            //res.push(EntityUpdate::CurrentEntity(*lid));
            //*last = Some(current.clone());
            res.push(EntityUpdate::Init(Box::from(current.clone()), *lid, *gid));
        }
        res
    }

    pub(crate) fn lid_by_entity(&self, entity: EntityID) -> Option<Lid> {
        self.tracker.tracked.get_by_right(&entity).copied()
    }

    pub(crate) fn got_authority(&mut self, full_entity_data: FullEntityData) {
        self.tracker.pending_authority.push(full_entity_data);
    }

    pub(crate) fn got_authoritys(&mut self, full_entity_data: Vec<FullEntityData>) {
        self.tracker.pending_authority.extend(full_entity_data);
    }

    pub(crate) fn entity_grabbed(&mut self, source: PeerId, lid: Lid, net: &mut NetManager) {
        let Some(info) = self.entity_entries.get(&lid) else {
            return;
        };
        if let Ok(entity) = self.tracker.entity_by_lid(lid) {
            if info.current.kind == EntityKind::Item {
                self.tracker.pending_localize.push((lid, source));
                safe_entitykill(entity);
                // "Untrack" entity
                self.tracker.tracked.remove_by_left(&lid);
                if let Some(gid) = self.entity_entries.remove(&lid).map(|e| e.gid) {
                    let _ = net.send(&NoitaOutbound::DesToProxy(
                        shared::des::DesToProxy::DeleteEntity(gid, None),
                    ));
                }
            } else {
                game_print("Tried to localize entity that's not an item");
            }
        }
    }

    pub(crate) fn death_notify(
        &mut self,
        entity_killed: EntityID,
        wait_on_kill: bool,
        pos: WorldPos,
        file: String,
        entity_responsible: Option<EntityID>,
    ) {
        self.tracker.pending_death_notify.push((
            entity_killed,
            wait_on_kill,
            pos,
            file,
            entity_responsible,
        ))
    }
}

fn collect_phys_info(entity: EntityID) -> eyre::Result<Vec<Option<PhysBodyInfo>>> {
    if entity.is_alive() {
        let phys_bodies =
            noita_api::raw::physics_body_id_get_from_entity(entity, None).unwrap_or_default();
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
                            noita_api::raw::physics_pos_to_game_pos(x.into(), Some(y.into()))
                                .ok()?;
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
    } else {
        Ok(Vec::new())
    }
}

impl RemoteDiffModel {
    pub(crate) fn wait_for_gid(&mut self, entity: EntityID, gid: Gid) {
        self.waiting_for_lid.insert(gid, entity);
    }
    pub(crate) fn apply_diff(&mut self, diff: &[EntityUpdate]) -> Vec<EntityID> {
        let empty_data = &mut EntityInfo::default();
        let mut ent_data = &mut EntityInfo::default();
        let mut dont_kill = Vec::new();
        for entry in diff.iter().cloned() {
            match entry {
                EntityUpdate::CurrentEntity(lid) => {
                    ent_data = self.entity_infos.get_mut(&lid).unwrap_or(empty_data)
                }
                EntityUpdate::Init(entity_entry, lid, gid) => {
                    if let Some(ent) = self.waiting_for_lid.remove(&gid) {
                        self.tracked.insert(lid, ent);
                        let _ = init_remote_entity(ent, Some(lid), Some(gid), false);
                        dont_kill.push(ent);
                    }
                    self.lid_to_gid.insert(lid, gid);
                    self.entity_infos.insert(lid, *entity_entry);
                    ent_data = self.entity_infos.get_mut(&lid).unwrap_or(empty_data)
                }
                EntityUpdate::LocalizeEntity(lid, peer_id) => {
                    if let Some((_, entity)) = self.tracked.remove_by_left(&lid) {
                        if peer_id != my_peer_id() {
                            safe_entitykill(entity);
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
                    EntityUpdate::SetFacingDirection(direction) => {
                        ent_data.facing_direction = direction
                    }
                    EntityUpdate::SetPhysInfo(vec) => ent_data.phys = vec,
                    EntityUpdate::SetCost(cost) => ent_data.cost = cost,
                    EntityUpdate::SetAnimations(ani) => ent_data.animations = ani,
                    EntityUpdate::SetStains(stains) => ent_data.current_stains = stains,
                    EntityUpdate::SetGameEffects(effects) => ent_data.game_effects = effects,
                    EntityUpdate::SetAiState(state) => ent_data.ai_state = state,
                    EntityUpdate::RemoveEntity(lid) => self.pending_remove.push(lid),
                    EntityUpdate::KillEntity {
                        lid,
                        wait_on_kill,
                        responsible_peer,
                    } => self
                        .pending_death_notify
                        .push((lid, wait_on_kill, responsible_peer)),
                    EntityUpdate::SetWand(wand) => ent_data.wand = wand,
                    EntityUpdate::SetWandRotation(rot) => ent_data.wand_rotation = rot,
                    EntityUpdate::SetLaser(peer) => ent_data.laser = peer,
                    EntityUpdate::SetAiRotation(rot) => ent_data.ai_rotation = rot,
                    EntityUpdate::SetLimbs(limbs) => ent_data.limbs = limbs,
                    EntityUpdate::SetSyncedVar(vars) => ent_data.synced_var = vars,
                    EntityUpdate::SetIsEnabled(enabled) => ent_data.is_enabled = enabled,
                    EntityUpdate::SetCounter(orbs) => ent_data.counter = orbs,
                    EntityUpdate::CurrentEntity(_)
                    | EntityUpdate::Init(_, _, _)
                    | EntityUpdate::LocalizeEntity(_, _) => unreachable!(),
                },
                _ => {}
            }
        }
        dont_kill
    }

    fn inner(
        &self,
        ctx: &mut ModuleCtx,
        entity_info: &EntityInfo,
        entity: EntityID,
        lid: &Lid,
    ) -> eyre::Result<Option<Lid>> {
        if entity_info.kind == EntityKind::Item && item_in_my_inventory(entity)?
            || item_in_entity_inventory(entity)?
        {
            entity.remove_tag(DES_TAG)?;
            with_entity_scripts(entity, |luac| {
                luac.set_script_throw_item(
                    "mods/quant.ew/files/system/entity_sync_helper/item_notify.lua".into(),
                )
            })?;
            for entity in entity.children(None) {
                with_entity_scripts(entity, |luac| {
                    luac.set_script_throw_item(
                        "mods/quant.ew/files/system/entity_sync_helper/item_notify.lua".into(),
                    )
                })?;
            }
            return Ok(Some(*lid));
        }
        for (name, s, i, f, b) in &entity_info.synced_var {
            let v = entity.get_var_or_default(name)?;
            v.set_value_string(s.into())?;
            v.set_value_int(*i)?;
            v.set_value_float(*f)?;
            v.set_value_bool(*b)?;
        }
        mom(entity, entity_info.counter, Some(entity_info.cost as i32))?;
        sun(entity, entity_info.counter)?;

        if let Some((gid, seri, _)) = &entity_info.wand {
            give_wand(entity, seri, *gid, true, Some(entity_info.wand_rotation))?;
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
                entity.set_components_with_tag_enabled("disabled_at_start".into(), true)?;
                entity.set_components_with_tag_enabled("enabled_at_start".into(), false)?;
                entity.add_tag("boss_centipede_active")?;
                for lua in
                    entity.iter_all_components_of_type_including_disabled::<LuaComponent>(None)?
                {
                    if [
                        "data/entities/animals/boss_centipede/boss_centipede_before_fight.lua",
                        "data/entities/animals/boss_centipede/boss_centipede_update.lua",
                    ]
                    .contains(&&*lua.script_source_file()?)
                    {
                        entity.remove_component(*lua)?;
                    }
                }
                let immortal = entity.add_component::<LuaComponent>()?;
                immortal.add_tag("ew_immortal")?;
                immortal.set_script_damage_about_to_be_received(
                    "mods/quant.ew/files/system/entity_sync_helper/immortal.lua".into(),
                )?;
                entity
                    .add_component::<VariableStorageComponent>()?
                    .set_name("ew_has_started".into())?;
                entity
                    .children(Some("protection".into()))
                    .iter()
                    .for_each(|ent| ent.kill());
            } else if let Some(var) = entity
                .try_get_first_component_including_disabled::<VariableStorageComponent>(None)?
                .iter()
                .find(|var| var.name().unwrap_or("".into()) == "active")
            {
                var.set_value_int(1)?;
                entity.set_components_with_tag_enabled("activate".into(), true)?
            }
        } else if let Some(var) = entity
            .try_get_first_component_including_disabled::<VariableStorageComponent>(None)?
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
            if let Ok(limb) = ent.get_first_component::<IKLimbAttackerComponent>(None) {
                entity.remove_component(*limb)?
            };
            if let Ok(limb) = ent.get_first_component::<IKLimbsAnimatorComponent>(None) {
                entity.remove_component(*limb)?
            };
        }
        let m = *ctx.fps_by_player.get(&self.peer_id).unwrap_or(&60) as f32
            / *ctx.fps_by_player.get(&my_peer_id()).unwrap_or(&60) as f32;
        let (vx, vy) = (entity_info.vx * m, entity_info.vy * m);
        if entity_info.phys.is_empty()
            || (entity_info.is_enabled && entity.has_tag("boss_centipede"))
        {
            let should_send_position =
                if let Some(com) = entity.try_get_first_component::<ItemComponent>(None)? {
                    !com.play_hover_animation()?
                } else {
                    true
                };

            let should_send_rotation =
                if let Some(com) = entity.try_get_first_component::<ItemComponent>(None)? {
                    !com.play_spinning_animation()? || com.play_hover_animation()?
                } else {
                    true
                };
            if should_send_rotation && should_send_position {
                entity.set_position(entity_info.x, entity_info.y, Some(entity_info.r))?;
            } else if should_send_position {
                entity.set_position(entity_info.x, entity_info.y, None)?;
            } else if should_send_rotation {
                let (x, y) = entity.position()?;
                entity.set_position(x, y, Some(entity_info.r))?;
            }
            if let Some(worm) = entity.try_get_first_component::<BossDragonComponent>(None)? {
                worm.set_m_target_vec((vx, vy))?;
            } else if let Some(worm) = entity.try_get_first_component::<WormComponent>(None)? {
                worm.set_m_target_vec((vx, vy))?;
            } else if let Some(vel) =
                entity.try_get_first_component::<CharacterDataComponent>(None)?
            {
                vel.set_m_velocity((vx, vy))?;
            } else if let Some(vel) = entity.try_get_first_component::<VelocityComponent>(None)? {
                vel.set_m_velocity((vx, vy))?;
            }
        }
        if let Some(damage) = entity.try_get_first_component::<DamageModelComponent>(None)? {
            if entity_info.hp > damage.max_hp()? as f32 {
                damage.set_max_hp(entity_info.hp as f64)?
            }
            let current_hp = damage.hp()? as f32;
            if current_hp > entity_info.hp {
                let old = damage.object_get_value::<f64>("damage_multipliers", "curse")?;
                if old != 1.0 {
                    damage.object_set_value("damage_multipliers", "curse", 1.0)?
                }
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
                if old != 0.0 {
                    damage.object_set_value("damage_multipliers", "curse", old)?
                }
                damage.set_hp(entity_info.hp as f64)?;
            } else if current_hp < entity_info.hp {
                if current_hp < 0.0 && entity_info.hp >= 0.0 {
                    damage.set_hp(f32::MIN_POSITIVE as f64)?;
                }
                let old = damage.object_get_value::<f64>("damage_multipliers", "healing")?;
                if old != 0.0 {
                    damage.object_set_value("damage_multipliers", "healing", 1.0)?
                }
                noita_api::raw::entity_inflict_damage(
                    entity.raw() as i32,
                    (current_hp - entity_info.hp) as f64,
                    "DAMAGE_HEALING".into(), //TODO should be enum
                    "hp sync".into(),
                    "NONE".into(),
                    0.0,
                    0.0,
                    None,
                    None,
                    None,
                    None,
                )?;
                if old != 0.0 {
                    damage.object_set_value("damage_multipliers", "healing", old)?
                }
                damage.set_hp(entity_info.hp as f64)?;
            }
        }

        if !entity_info.phys.is_empty() && entity.check_all_phys_init()? && entity.is_alive() {
            let phys_bodies =
                noita_api::raw::physics_body_id_get_from_entity(entity, None).unwrap_or_default();
            for (p, physics_body_id) in entity_info.phys.iter().zip(phys_bodies.iter()) {
                let Some(p) = p else {
                    continue;
                };
                let (x, y) = noita_api::raw::game_pos_to_physics_pos(p.x.into(), Some(p.y.into()))?;
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
            if entity_info.cost == 0 {
                entity.set_components_with_tag_enabled("shop_cost".into(), false)?;
            }
        }

        entity.set_game_effects(&entity_info.game_effects)?;

        if entity.get_var("rolling").is_some() {
            let var = entity.get_var_or_default("ew_rng")?;
            let bytes = entity_info.current_stains.to_le_bytes();
            let is_rolling = bytes[0];
            let bytes: [u8; 4] = [bytes[4], bytes[5], bytes[6], bytes[7]];
            let rng = i32::from_le_bytes(bytes);
            var.set_value_int(rng)?;
            let var = entity.get_var_or_default("rolling")?;
            if is_rolling == 1 {
                if var.value_int()? == 0 {
                    var.set_value_int(4)?;
                    entity
                        .iter_all_components_of_type::<SpriteComponent>(None)?
                        .for_each(|s| {
                            let _ = s.set_rect_animation("roll".into());
                        })
                } else if var.value_int()? == 8 {
                    let (x, y) = entity.position()?;
                    if !noita_api::raw::entity_get_in_radius_with_tag(
                        x as f64,
                        y as f64,
                        480.0,
                        "player_unit".into(),
                    )?
                    .is_empty()
                    {
                        game_print("$item_die_roll");
                    }
                }
            } else {
                var.set_value_int(0)?;
            }
        } else {
            entity.set_current_stains(entity_info.current_stains)?;
        }
        if let Some(ai) =
            entity.try_get_first_component_including_disabled::<AnimalAIComponent>(None)?
        {
            ai.set_ai_state(entity_info.ai_state)?;
            ai.set_m_ranged_attack_current_aim_angle(entity_info.ai_rotation)?;
        } else if let Ok(sprites) = entity.iter_all_components_of_type::<SpriteComponent>(None) {
            for (sprite, animation) in sprites
                .filter(|sprite| {
                    sprite
                        .image_file()
                        .map(|c| c.ends_with(".xml"))
                        .unwrap_or(false)
                })
                .zip(entity_info.animations.iter())
            {
                sprite.set_special_scale_x(if entity_info.facing_direction.0 {
                    1.0
                } else {
                    -1.0
                })?;
                sprite.set_special_scale_y(if entity_info.facing_direction.1 {
                    1.0
                } else {
                    -1.0
                })?;
                if *animation == u16::MAX {
                    continue;
                }
                let file = sprite.image_file()?;
                let text = noita_api::raw::mod_text_file_get_content(file)?;
                let mut split = text.split("name=\"");
                split.next();
                let data: Vec<&str> = split.filter_map(|piece| piece.split("\"").next()).collect();
                if data.len() > *animation as usize {
                    sprite.set_rect_animation(data[*animation as usize].into())?;
                    sprite.set_next_rect_animation(data[*animation as usize].into())?;
                }
            }
            let laser = entity.try_get_first_component::<LaserEmitterComponent>(None)?;
            if entity_info.laser != Target::None {
                let laser = if let Some(laser) = laser {
                    laser
                } else {
                    let laser = entity.add_component::<LaserEmitterComponent>()?;
                    laser.object_set_value::<i32>("laser", "max_cell_durability_to_destroy", 0)?;
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
                let ent = if let Target::Peer(peer) = entity_info.laser {
                    ctx.player_map.get_by_left(&peer).cloned()
                } else if let Target::Gid(gid) = entity_info.laser {
                    self.find_by_gid(gid)
                } else {
                    None
                };
                if let Some(ent) = ent {
                    let (x, y) = entity.position()?;
                    let (tx, ty) = ent.position()?;
                    if !raytrace_platforms(x as f64, y as f64, tx as f64, ty as f64)?.0 {
                        laser.set_is_emitting(true)?;
                        let (dx, dy) = (tx - x, ty - y);
                        let theta = dy.atan2(dx);
                        laser.set_laser_angle_add_rad(theta - entity.rotation()?)?;
                        laser.object_set_value::<f32>("laser", "max_length", dx.hypot(dy))?;
                    } else {
                        laser.set_is_emitting(false)?;
                    }
                }
            } else if let Some(laser) = laser {
                laser.set_is_emitting(false)?;
            }
        }
        Ok(None)
    }

    pub(crate) fn apply_entities(
        &mut self,
        ctx: &mut ModuleCtx,
        start: usize,
        time: u128,
    ) -> eyre::Result<(usize, u128)> {
        let mut to_remove = Vec::new();
        let tmr = Instant::now();
        let l = self.entity_infos.len();
        let mut end = None;
        let start = if start >= l { 0 } else { start };
        for (i, (lid, entity_info)) in self.entity_infos.iter().enumerate() {
            match self
                .tracked
                .get_by_left(lid)
                .and_then(|entity_id| entity_id.is_alive().then_some(*entity_id))
            {
                Some(entity) => {
                    if time + tmr.elapsed().as_micros() > 5000 || start > i {
                        if end.is_none() && start <= i {
                            end = Some(i);
                        }
                        if entity_info.phys.is_empty()
                            || (entity_info.is_enabled && entity.has_tag("boss_centipede"))
                        {
                            let should_send_position = if let Some(com) =
                                entity.try_get_first_component::<ItemComponent>(None)?
                            {
                                !com.play_hover_animation()?
                            } else {
                                true
                            };
                            let should_send_rotation = if let Some(com) =
                                entity.try_get_first_component::<ItemComponent>(None)?
                            {
                                !com.play_spinning_animation()? || com.play_hover_animation()?
                            } else {
                                true
                            };
                            if should_send_rotation && should_send_position {
                                entity.set_position(
                                    entity_info.x,
                                    entity_info.y,
                                    Some(entity_info.r),
                                )?;
                            } else if should_send_position {
                                entity.set_position(entity_info.x, entity_info.y, None)?;
                            } else if should_send_rotation {
                                let (x, y) = entity.position()?;
                                entity.set_position(x, y, Some(entity_info.r))?;
                            }
                        }
                    } else {
                        match self.inner(ctx, entity_info, entity, lid) {
                            Ok(Some(lid)) => to_remove.push(lid),
                            Err(s) => print_error(s)?,
                            _ => {}
                        }
                    }
                }
                _ => {
                    if start <= i {
                        if time + tmr.elapsed().as_micros() > 5000 {
                            if end.is_none() {
                                end = Some(i);
                            }
                        } else {
                            if let Some(gid) = self.lid_to_gid.get(lid) {
                                if ctx.dont_spawn.contains(gid) {
                                    continue;
                                }
                            }
                            let entity = spawn_entity_by_data(
                                &entity_info.spawn_info,
                                entity_info.x,
                                entity_info.y,
                            )?;
                            init_remote_entity(
                                entity,
                                Some(*lid),
                                self.lid_to_gid.get(lid).copied(),
                                entity_info.drops_gold,
                            )?;
                            self.tracked.insert(*lid, entity);
                        }
                    }
                }
            }
        }
        for lid in to_remove {
            self.grab_request.push(lid);
            self.entity_infos.remove(&lid);
        }
        Ok((end.unwrap_or(0), time + tmr.elapsed().as_micros()))
    }

    pub(crate) fn kill_entities(&mut self, ctx: &mut ModuleCtx) -> eyre::Result<()> {
        for (lid, wait_on_kill, responsible) in self.pending_death_notify.drain(..) {
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
                entity
                    .children(Some("protection".into()))
                    .iter()
                    .for_each(|ent| ent.kill());
                self.pending_remove.retain(|l| l != &lid);
                self.entity_infos.remove(&lid);
                if !wait_on_kill {
                    damage.set_wait_for_kill_flag_on_death(false)?;
                }
                damage.object_set_value("damage_multipliers", "curse", 1.0)?;
                noita_api::raw::entity_inflict_damage(
                    entity.raw() as i32,
                    damage.hp()? + f32::MIN_POSITIVE as f64,
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
                damage.set_ui_report_damage(false)?;
                noita_api::raw::entity_inflict_damage(
                    entity.raw() as i32,
                    damage.max_hp()? * 100.0,
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
                if wait_on_kill {
                    damage.set_kill_now(true)?;
                } else {
                    entity.kill()
                }
            }
        }
        for lid in self.pending_remove.drain(..) {
            self.entity_infos.remove(&lid);
            if let Some((_, entity)) = self.tracked.remove_by_left(&lid) {
                safe_entitykill(entity);
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
            if let Ok(Some(vel)) = deserialized.try_get_first_component::<VelocityComponent>(None) {
                if let Some((vx, vy)) = projectile.vel {
                    let _ = vel.set_m_velocity((vx, vy));
                }
            }
        }
    }

    /*pub(crate) fn drain_backtrack(&mut self) -> impl Iterator<Item = EntityID> + '_ {
        self.backtrack.drain(..)
    }*/

    pub(crate) fn drain_grab_request(&mut self) -> impl Iterator<Item = Lid> + '_ {
        self.grab_request.drain(..)
    }
}

/// Modifies a newly spawned entity so it can be synced properly.
/// Removes components that shouldn't be on entities that were replicated from a remote,
/// generally because they interfere with things we're supposed to sync.
pub fn init_remote_entity(
    entity: EntityID,
    lid: Option<Lid>,
    gid: Option<Gid>,
    drops_gold: bool,
) -> eyre::Result<()> {
    if entity.has_tag("player_unit") {
        entity.kill()
    }
    entity.remove_all_components_of_type::<CameraBoundComponent>(None)?;
    entity.remove_all_components_of_type::<StreamingKeepAliveComponent>(None)?;
    entity.remove_all_components_of_type::<CharacterPlatformingComponent>(None)?;
    entity.remove_all_components_of_type::<PhysicsAIComponent>(None)?;
    entity.remove_all_components_of_type::<AdvancedFishAIComponent>(None)?;
    entity.remove_all_components_of_type::<IKLimbsAnimatorComponent>(None)?;
    entity.remove_all_components_of_type::<LifetimeComponent>(None)?;
    let mut any = false;
    for ai in entity.iter_all_components_of_type_including_disabled::<AIAttackComponent>(None)? {
        any = any || ai.attack_ranged_aim_rotation_enabled()?;
        ai.set_attack_ranged_entity_count_max(0)?;
        ai.set_attack_ranged_entity_count_min(0)?;
    }
    for ai in entity.iter_all_components_of_type_including_disabled::<AnimalAIComponent>(None)? {
        any = any || ai.attack_ranged_aim_rotation_enabled()?;
        ai.set_attack_ranged_entity_count_max(0)?;
        ai.set_attack_ranged_entity_count_min(0)?;
        ai.set_attack_melee_damage_min(0.0)?;
        ai.set_attack_melee_damage_max(0.0)?;
        ai.set_attack_dash_damage(0.0)?;
        ai.set_ai_state_timer(i32::MAX)?;
        ai.set_attack_ranged_state_duration_frames(i32::MAX)?;
        ai.set_keep_state_alive_when_enabled(true)?;
    }
    if !any {
        entity.remove_all_components_of_type::<AnimalAIComponent>(None)?;
        entity.remove_all_components_of_type::<AIAttackComponent>(None)?;
        for sprite in
            entity.iter_all_components_of_type::<SpriteComponent>(Some("character".into()))?
        {
            sprite.remove_tag("character")?;
            sprite.set_has_special_scale(true)?;
        }
    }
    entity
        .try_get_first_component_including_disabled::<WormComponent>(None)?
        .iter()
        .for_each(|w| w.set_bite_damage(0.0).unwrap_or(()));
    entity
        .try_get_first_component_including_disabled::<BossDragonComponent>(None)?
        .iter()
        .for_each(|w| w.set_bite_damage(0.0).unwrap_or(()));

    entity.add_tag(DES_TAG)?;
    entity.add_tag("polymorphable_NOT")?;
    if lid.is_some() {
        if let Some(damage) = entity.try_get_first_component::<DamageModelComponent>(None)? {
            damage.set_wait_for_kill_flag_on_death(true)?;
            damage.set_physics_objects_damage(false)?;
        }
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

    for lua in entity.iter_all_components_of_type_including_disabled::<LuaComponent>(None)? {
        if (!drops_gold
            && lua.script_death().ok() == Some("data/scripts/items/drop_money.lua".into()))
            || [
                "data/scripts/animals/leader_damage.lua",
                "data/scripts/animals/giantshooter_death.lua",
                "data/scripts/animals/blob_damage.lua",
                "data/scripts/items/die_roll.lua",
                "data/scripts/animals/iceskull_damage.lua",
                "data/scripts/buildings/lukki_eggs.lua",
                "data/scripts/props/physics_vase_damage.lua",
            ]
            .contains(&&*lua.script_damage_received()?)
            || [
                "data/scripts/buildings/firebugnest.lua",
                "data/scripts/buildings/flynest.lua",
                "data/scripts/buildings/spidernest.lua",
                "data/scripts/buildings/bunker2_check.lua",
                "data/scripts/buildings/bunker_check.lua",
                "data/scripts/buildings/statue_hand_state.lua",
                "data/scripts/buildings/failed_alchemist_orb.lua",
                "data/scripts/buildings/ghost_crystal.lua",
                "data/scripts/buildings/snowcrystal.lua",
                "data/scripts/buildings/sun/spot_2.lua",
                "data/scripts/buildings/sun/spot_3.lua",
                "data/scripts/buildings/sun/spot_4.lua",
                "data/scripts/props/suspended_container_physics_objects.lua",
            ]
            .contains(&&*lua.script_source_file()?)
            || ["data/scripts/buildings/statue_hand_modified.lua"].contains(&&*lua.script_kick()?)
            || [
                "data/scripts/items/utility_box.lua",
                "data/scripts/items/chest_random.lua",
                "data/scripts/buildings/chest_steel.lua",
                "data/scripts/items/chest_random_super.lua",
                "data/scripts/buildings/chest_light.lua",
                "data/scripts/buildings/chest_dark.lua",
                "data/biome_impl/static_tile/chest_darkness.lua",
            ]
            .contains(&&*lua.script_physics_body_modified()?)
            || ["data/scripts/animals/failed_alchemist_b_death.lua"]
                .contains(&&*lua.script_death()?)
        {
            entity.remove_component(*lua)?;
        }
    }
    let immortal = entity.add_component::<LuaComponent>()?;
    immortal.add_tag("ew_immortal")?;
    immortal.set_script_damage_about_to_be_received(
        "mods/quant.ew/files/system/entity_sync_helper/immortal.lua".into(),
    )?;
    if let Some(var) = entity.get_var("ghost_id") {
        if let Ok(ent) = EntityID::try_from(var.value_int()? as isize) {
            ent.kill()
        }
    }
    if entity.has_tag("boss_dragon") && drops_gold {
        let lua = entity.add_component::<LuaComponent>()?;
        lua.set_script_death("data/scripts/animals/boss_dragon_death.lua".into())?;
        lua.set_execute_every_n_frame(-1)?;
    }
    if let Some(life) =
        entity.try_get_first_component_including_disabled::<LifetimeComponent>(None)?
    {
        life.set_lifetime(i32::MAX)?;
    }
    if let Some(pickup) =
        entity.try_get_first_component_including_disabled::<ItemPickUpperComponent>(None)?
    {
        pickup.set_drop_items_on_death(false)?;
        pickup.set_only_pick_this_entity(Some(EntityID(NonZero::new(1).unwrap())))?;
    }

    if let Some(ghost) =
        entity.try_get_first_component_including_disabled::<GhostComponent>(None)?
    {
        ghost.set_die_if_no_home(false)?;
    }

    if entity.has_tag("egg_item") {
        if let Some(explosion) =
            entity.try_get_first_component_including_disabled::<ExplodeOnDamageComponent>(None)?
        {
            explosion.object_set_value::<Cow<'_, str>>(
                "config_explosion",
                "load_this_entity",
                "".into(),
            )?
        }
    }

    entity
        .iter_all_components_of_type_including_disabled::<VariableStorageComponent>(None)?
        .for_each(|var| {
            let name = var.name().unwrap_or("".into());
            if name == "ew_gid_lid" {
                let _ = entity.remove_component(*var);
            } else if name == "throw_time" && drops_gold {
                let _ = var.set_value_int(game_get_frame_num().unwrap_or(0) - 4);
            }
        });

    if let Some(lid) = lid {
        let var = entity.add_component::<VariableStorageComponent>()?;
        var.set_name("ew_gid_lid".into())?;
        if let Some(gid) = gid {
            var.set_value_string(gid.0.to_string().into())?;
        }
        var.set_value_int(i32::from_le_bytes(lid.0.to_le_bytes()))?;
        var.set_value_bool(false)?;
    }

    if entity
        .try_get_first_component_including_disabled::<PhysicsBodyComponent>(None)?
        .is_none()
        && entity
            .try_get_first_component_including_disabled::<PhysicsBody2Component>(None)?
            .is_none()
    {
        ephemerial(entity.0.get() as u32)?
    }

    Ok(())
}

fn item_in_inventory(entity: EntityID) -> Result<bool, eyre::Error> {
    Ok(entity.root()? != Some(entity))
}

fn item_in_my_inventory(entity: EntityID) -> Result<bool, eyre::Error> {
    Ok(entity
        .root()?
        .map(|e| {
            !e.has_tag("ew_client") && (e.has_tag("player_unit") || e.has_tag("polymorphed_player"))
        })
        .unwrap_or(false))
}

fn item_in_entity_inventory(entity: EntityID) -> Result<bool, eyre::Error> {
    Ok(entity
        .root()?
        .and_then(|e| e.get_var("ew_gid_lid").unwrap().value_bool().ok())
        .unwrap_or(false))
}

fn not_in_player_inventory(entity: EntityID) -> Result<bool, eyre::Error> {
    Ok(entity
        .root()?
        .map(|e| !e.has_tag("ew_client"))
        .unwrap_or(true))
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
            let ent = EntityID::load(filename, Some(x as f64), Some(y as f64))?;
            for lua in ent.iter_all_components_of_type::<LuaComponent>(None)? {
                if ["data/scripts/props/suspended_container_physics_objects.lua"]
                    .contains(&&*lua.script_source_file()?)
                {
                    ent.remove_component(*lua)?;
                }
            }
            Ok(ent)
        }
        EntitySpawnInfo::Serialized {
            //serialized_at: _,
            data,
        } => deserialize_entity(data, x, y),
    }
}

pub(crate) fn entity_is_item(entity: EntityID) -> eyre::Result<bool> {
    Ok(entity
        .try_get_first_component_including_disabled::<ItemComponent>(None)?
        .is_some()
        && entity.root()? == Some(entity))
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
            component.add_tag("ew_remove_on_send")?;
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
    entity.set_component_enabled(*lc, true)?;
    lc.add_tag("enabled_in_inventory")?;
    lc.add_tag("enabled_in_world")?;
    lc.add_tag("enabled_in_hand")?;
    lc.set_execute_on_added(false)?;
    lc.set_m_next_execution_time(game_get_frame_num()? + 1)?;
    Ok(())
}

fn safe_entitykill(entity: EntityID) {
    let _ = entity.remove_all_components_of_type::<AudioComponent>(None);
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

fn give_wand(
    entity: EntityID,
    seri: &[u8],
    gid: Option<Gid>,
    delete: bool,
    r: Option<f32>,
) -> eyre::Result<()> {
    let inv = if let Some(inv) =
        entity.try_get_first_component_including_disabled::<Inventory2Component>(None)?
    {
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
            if gid != Some(Gid(tgid)) {
                if r.is_some() {
                    entity.set_component_enabled(*inv, true)?;
                }
                wand.kill()
            } else {
                if r.is_some() {
                    entity.set_component_enabled(*inv, false)?;
                }
                stop = true
            }
        } else if wand
            .iter_all_components_of_type_including_disabled::<VariableStorageComponent>(None)?
            .any(|p| p.name().ok().unwrap_or("".into()) == "ew_spawned_wand")
        {
            if r.is_some() {
                entity.set_component_enabled(*inv, false)?;
            }
            stop = true
        } else {
            if r.is_some() {
                entity.set_component_enabled(*inv, true)?;
            }
            wand.kill()
        }
        if let Some(r) = r {
            let (x, y) =
                noita_api::raw::entity_get_hotspot(entity.raw() as i32, "hand".into(), true, None)?;
            wand.set_position(x as f32, y as f32, Some(r))?;
        }
    }
    if !stop {
        if r.is_some() {
            entity.set_component_enabled(*inv, true)?;
        }
        let (x, y) = entity.position()?;
        let wand = deserialize_entity(seri, x, y)?;
        if delete {
            if let Some(pickup) =
                entity.try_get_first_component_including_disabled::<ItemPickUpperComponent>(None)?
            {
                pickup.set_only_pick_this_entity(Some(wand))?;
            }
            let quick = if let Some(quick) = entity.children(None).iter().find_map(|a| {
                if a.name().ok()? == "inventory_quick" {
                    a.children(None).iter().for_each(|e| e.kill());
                    Some(a)
                } else {
                    None
                }
            }) {
                *quick
            } else {
                let quick =
                    entity_create_new(Some("inventory_quick".into()))?.wrap_err("unreachable")?;
                entity.add_child(quick);
                quick
            };
            quick.add_child(wand);
            if let Some(ability) =
                wand.try_get_first_component_including_disabled::<AbilityComponent>(None)?
            {
                ability.set_drop_as_item_on_death(false)?;
            }
            if let Some(item) =
                wand.try_get_first_component_including_disabled::<ItemComponent>(None)?
            {
                item.set_remove_default_child_actions_on_death(true)?;
                item.set_remove_on_death_if_empty(true)?;
                item.set_remove_on_death(true)?;
            }
            let lua = wand.add_component::<LuaComponent>()?;
            lua.set_script_source_file(
                "mods/quant.ew/files/system/entity_sync_helper/scripts/kill_on_drop.lua".into(),
            )?;
            lua.set_execute_every_n_frame(1)?;
            lua.set_execute_times(-1)?;
            wand.set_component_enabled(*lua, true)?;
            lua.add_tag("enabled_in_world")?;
            if gid.is_none() {
                let var = wand.add_component::<VariableStorageComponent>()?;
                var.set_name("ew_spawned_wand".into())?;
            }
        } else {
            wand.set_components_with_tag_enabled("enabled_in_hand".into(), false)?;
            wand.set_components_with_tag_enabled("enabled_in_inventory".into(), false)?;
            wand.set_components_with_tag_enabled("enabled_in_world".into(), true)?;
        }
        //TODO set active item?
    }
    Ok(())
}

fn mom(entity: EntityID, counter: u8, cost: Option<i32>) -> eyre::Result<()> {
    if entity.has_tag("boss_wizard") {
        for ent in entity.children(None) {
            if ent.has_tag("touchmagic_immunity") {
                if let Ok(var) = ent
                    .get_first_component_including_disabled::<VariableStorageComponent>(Some(
                        "wizard_orb_id".into(),
                    ))
                {
                    if let Ok(n) = var.value_int() {
                        if (counter & (1 << (n as u8))) == 0 {
                            ent.kill()
                        } else if let Ok(damage) =
                            ent.get_first_component::<DamageModelComponent>(None)
                        {
                            damage.set_wait_for_kill_flag_on_death(true)?;
                            damage.set_hp(damage.max_hp()?)?;
                        }
                    }
                    if let Some(cost) = cost {
                        if let Ok(v) = ent.get_var_or_default("ew_frame_num") {
                            let _ = v.add_tag("ew_frame_num");
                            let _ = v.set_value_int(cost);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
fn sun(entity: EntityID, counter: u8) -> eyre::Result<()> {
    if entity.has_tag("seed_d") {
        let essences = entity.get_var_or_default("sunbaby_essences_list")?;
        let mut s = String::new();
        if counter & 1 == 1 {
            s += "water,";
            entity.set_components_with_tag_enabled("water".into(), true)?;
        }
        if counter & 2 == 2 {
            s += "fire,";
            entity.set_components_with_tag_enabled("fire".into(), true)?;
            entity.set_components_with_tag_enabled("fire_disable".into(), false)?;
        }
        if counter & 4 == 4 {
            s += "air,";
            entity.set_components_with_tag_enabled("air".into(), true)?;
        }
        if counter & 8 == 8 {
            s += "earth,";
            entity.set_components_with_tag_enabled("earth".into(), true)?;
            entity.set_components_with_tag_enabled("earth_disable".into(), false)?;
        }
        if counter & 16 == 16 {
            s += "poop,";
            entity.set_components_with_tag_enabled("poop".into(), true)?;
        }
        essences.set_value_string(s.into())?;
        let n = (counter & (32 + 64 + 128)) / 32;
        if counter != 0 {
            let sprite =
                entity.get_first_component::<SpriteComponent>(Some("sunbaby_sprite".into()))?;
            match n {
                0 => sprite.set_image_file("data/props_gfx/sun_small_purple.png".into())?,
                1 => sprite.set_image_file("data/props_gfx/sun_small_red.png".into())?,
                2 => sprite.set_image_file("data/props_gfx/sun_small_blue.png".into())?,
                3 => sprite.set_image_file("data/props_gfx/sun_small_green.png".into())?,
                4 => sprite.set_image_file("data/props_gfx/sun_small_orange.png".into())?,
                _ => {}
            }
        }
    }
    Ok(())
}
