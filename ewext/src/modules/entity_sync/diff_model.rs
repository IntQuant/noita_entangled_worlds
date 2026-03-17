use super::NetManager;
use crate::{ephemerial, modules::ModuleCtx, my_peer_id, print_error};
use bimap::BiHashMap;
use eyre::{Context, OptionExt, eyre};
use noita_api::raw::raytrace_platforms;
use noita_api::serialize::{deserialize_entity, serialize_entity};
use noita_api::{
    AIAttackComponent, AbilityComponent, AdvancedFishAIComponent, AnimalAIComponent,
    AudioComponent, BossDragonComponent, BossHealthBarComponent, CachedTag, CameraBoundComponent,
    CharacterDataComponent, CharacterPlatformingComponent, ComponentTag, DamageModelComponent,
    DamageType, EntityID, EntityManager, ExplodeOnDamageComponent, GhostComponent,
    IKLimbAttackerComponent, IKLimbComponent, IKLimbWalkerComponent, IKLimbsAnimatorComponent,
    Inventory2Component, ItemComponent, ItemCostComponent, ItemPickUpperComponent,
    LaserEmitterComponent, LifetimeComponent, LuaComponent, PhysData, PhysicsAIComponent,
    PhysicsBody2Component, PhysicsBodyComponent, SpriteComponent, StreamingKeepAliveComponent,
    VarName, VariableStorageComponent, VelocityComponent, WormComponent, game_print,
};
use rustc_hash::{FxHashMap, FxHashSet};
use shared::des::{
    EntityInit, GLOBAL_AUTHORITY_RADIUS, GLOBAL_TRANSFER_RADIUS, TRANSFER_RADIUS, Target,
    UpdateOrUpload,
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

#[derive(Clone)]
struct EntityEntryPair {
    last: Option<EntityInfo>,
    current: Option<EntityInfo>,
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
    pub update_buffer: Vec<EntityUpdate>,
    pub init_buffer: Vec<EntityInit>,
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
        //TODO since i do this in other places, i do more work at the start of the second then the end of the second as len is not equal to a multiple of 60 generally, so this should be spread out
        let start = (frame_num % 60) * batch_size;
        let end = (start + batch_size).min(len);
        let mut upload = std::mem::take(&mut self.upload);
        let mut res: Vec<UpdateOrUpload> = self
            .entity_entries
            .iter()
            .skip(start)
            .take(end.saturating_sub(start))
            .filter_map(|(lid, p)| {
                let EntityEntryPair {
                    current: Some(current),
                    gid,
                    last,
                } = p
                else {
                    unreachable!()
                };
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
                            max_hp: current.max_hp,
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
            if let Some(EntityEntryPair {
                current: Some(current),
                gid,
                last,
            }) = self.entity_entries.get(&lid)
                && !self.dont_upload.contains(&lid)
            {
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
                        max_hp: current.max_hp,
                        counter: current.counter,
                        phys: current.phys.clone(),
                        synced_var: current.synced_var.clone(),
                    }));
                } else {
                    self.upload.insert(lid);
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
    pub(crate) fn remove_entities(self, entity_manager: &mut EntityManager) -> eyre::Result<()> {
        for (_, ent) in self.tracked.into_iter() {
            entity_manager.set_current_entity(ent)?;
            safe_entitykill(entity_manager);
        }
        Ok(())
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
            update_buffer: Vec::with_capacity(512),
            init_buffer: Vec::with_capacity(512),
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
        entity_manager: &mut EntityManager,
    ) -> eyre::Result<bool> {
        let entity = self
            .entity_by_lid(lid)
            .wrap_err_with(|| eyre!("Failed to grab update info for {:?} {:?}", gid, lid))?;
        entity_manager.set_current_entity(entity)?;

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
            self.temporary_untrack_item(ctx, gid, lid, entity, entity_manager)?;
            return Ok(false);
        }

        let (x, y, r, sx, sy) = entity.transform()?;
        let should_send_position = if let Some(com) =
            entity_manager.try_get_first_component::<ItemComponent>(ComponentTag::None)
        {
            !com.play_hover_animation()?
        } else {
            true
        };

        if should_send_position {
            (info.x, info.y) = (x as f32, y as f32);
        }

        let should_send_rotation = if let Some(com) =
            entity_manager.try_get_first_component::<ItemComponent>(ComponentTag::None)
        {
            !com.play_spinning_animation()? || com.play_hover_animation()?
        } else {
            true
        };

        if should_send_rotation {
            info.r = r as f32
        }

        if let Some(inv) = entity_manager
            .try_get_first_component_including_disabled::<Inventory2Component>(ComponentTag::None)
        {
            if let Some(wand) = inv.m_actual_active_item()? {
                if info.wand.is_none() {
                    info.wand = if wand.is_alive() {
                        wand.remove_all_components_of_type::<LuaComponent>(Some(
                            "ew_immortal".into(),
                        ))?;
                        let r = wand.rotation()?;
                        info.wand_rotation = r as f32;
                        if let Some(Some(gid)) = wand
                            .get_var("ew_gid_lid")
                            .map(|var| var.value_string().ok()?.parse::<u64>().ok())
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
                        info.wand_rotation = r as f32;
                    }
                }
            } else {
                info.wand = None;
            };
        }
        info.is_enabled = (entity_manager.has_tag(const { CachedTag::from_tag("boss_centipede") })
            && entity_manager
                .try_get_first_component::<BossHealthBarComponent>(
                    const { ComponentTag::from_str("disabled_at_start") },
                )
                .is_some())
            || entity_manager
                .get_var(const { VarName::from_str("active") })
                .map(|var| var.value_int().unwrap_or(0) == 1)
                .unwrap_or(false)
            || (entity_manager.has_tag(const { CachedTag::from_tag("pitcheck_b") })
                && entity_manager
                    .try_get_first_component::<LuaComponent>(
                        const { ComponentTag::from_str("disabled") },
                    )
                    .is_some());

        info.limbs = entity
            .children(None)
            .filter_map(|ent| {
                if let Ok(limb) = ent.get_first_component::<IKLimbComponent>(None) {
                    limb.end_position().ok()
                } else {
                    None
                }
            })
            .collect();

        if let Some(worm) =
            entity_manager.try_get_first_component::<BossDragonComponent>(ComponentTag::None)
        {
            (info.vx, info.vy) = worm.m_target_vec()?;
        } else if let Some(worm) =
            entity_manager.try_get_first_component::<WormComponent>(ComponentTag::None)
        {
            (info.vx, info.vy) = worm.m_target_vec()?;
        } else if let Some(vel) =
            entity_manager.try_get_first_component::<CharacterDataComponent>(ComponentTag::None)
        {
            (info.vx, info.vy) = vel.m_velocity()?;
        } else if let Some(vel) =
            entity_manager.try_get_first_component::<VelocityComponent>(ComponentTag::None)
        {
            (info.vx, info.vy) = vel.m_velocity()?;
        }

        if entity_manager.has_tag(const { CachedTag::from_tag("card_action") })
            && let Some(vel) =
                entity_manager.try_get_first_component::<VelocityComponent>(ComponentTag::None)
        {
            let (cx, cy) = entity_manager.camera_pos();
            if ((cx - x) as f32).powi(2) + ((cy - y) as f32).powi(2) > 512.0 * 512.0 {
                vel.set_gravity_y(0.0)?;
                vel.set_air_friction(10.0)?;
            } else {
                vel.set_gravity_y(400.0)?;
                vel.set_air_friction(0.55)?;
            }
        }

        if let Some(damage) =
            entity_manager.try_get_first_component::<DamageModelComponent>(ComponentTag::None)
        {
            info.hp = damage.hp()? as f32;
            info.max_hp = damage.max_hp()? as f32;
        }

        if entity_manager.check_all_phys_init()? {
            info.phys = collect_phys_info(entity)?;
        }

        if let Some(item_cost) =
            entity_manager.try_get_first_component::<ItemCostComponent>(ComponentTag::None)
        {
            info.cost = item_cost.cost()?;
        } else if entity_manager.has_tag(const { CachedTag::from_tag("boss_wizard") }) {
            info.cost = entity_manager.frame_num() as i64;
            info.counter = entity
                .children(None)
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
        if entity_manager.has_tag(const { CachedTag::from_tag("seed_d") }) {
            let essences = entity_manager
                .get_var_or_default(const { VarName::from_str("sunbaby_essences_list") })?;
            let sprite = entity_manager.get_first_component::<SpriteComponent>(
                const { ComponentTag::from_str("sunbaby_sprite") },
            )?;
            let sprite = sprite.image_file()?;
            let num: u8 = match sprite.as_ref() {
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
            .into_iter()
            .map(|(e, _)| e)
            .collect::<Vec<GameEffectData>>();

        info.current_stains =
            if let Some(var) = entity_manager.get_var(const { VarName::from_str("rolling") }) {
                if var.value_int()? == 0 {
                    let rng = rand::random::<i32>();
                    let var =
                        entity_manager.get_var_or_default(const { VarName::from_str("ew_rng") })?;
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
                entity_manager.get_current_stains()?
            };

        let mut any = false;
        for ai in entity_manager
            .iter_all_components_of_type_including_disabled::<AIAttackComponent>(ComponentTag::None)
        {
            any = any || ai.attack_ranged_aim_rotation_enabled()?;
        }
        for ai in entity_manager
            .iter_all_components_of_type_including_disabled::<AnimalAIComponent>(ComponentTag::None)
        {
            any = any || ai.attack_ranged_aim_rotation_enabled()?;
        }
        if any {
            if let Some(ai) = entity_manager
                .try_get_first_component_including_disabled::<AnimalAIComponent>(ComponentTag::None)
            {
                info.ai_state = ai.ai_state()?;
                info.ai_rotation = ai.m_ranged_attack_current_aim_angle()?;
            }
        } else {
            let mut files = std::mem::take(&mut entity_manager.files);
            let sprites =
                entity_manager.iter_all_components_of_type::<SpriteComponent>(ComponentTag::None);
            info.facing_direction = (sx.is_sign_positive(), sy.is_sign_positive());
            info.animations = sprites
                .filter_map(|sprite| {
                    let file = sprite.image_file().ok()?;
                    if file.ends_with(".xml") {
                        let text = noita_api::get_file(&mut files, file).ok()?;
                        let animation = sprite.rect_animation().unwrap_or("".into());
                        Some(
                            text.iter()
                                .position(|name| name == &animation)
                                .unwrap_or(usize::MAX) as u16,
                        )
                    } else {
                        None
                    }
                })
                .collect();
            if let Some(ai) =
                entity_manager.try_get_first_component::<AnimalAIComponent>(ComponentTag::None)
                && ai.attack_ranged_use_laser_sight()?
                && !ai.is_static_turret()?
            {
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
            entity_manager.files = files;
        }

        info.synced_var = entity_manager
            .iter_all_components_of_type_including_disabled::<VariableStorageComponent>(
                const { ComponentTag::from_str("ew_synced_var") },
            )
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
            let is_beyond_authority = (x as f32 - cam_pos.0).powi(2)
                + (y as f32 - cam_pos.1).powi(2)
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
                    self.transfer_authority_to(
                        ctx,
                        gid,
                        lid,
                        peer,
                        info,
                        do_upload,
                        entity_manager,
                    )
                    .wrap_err("Failed to transfer authority")?;
                    return Ok(do_upload);
                } else if !info.is_global && is_beyond_authority {
                    self.release_authority(ctx, gid, lid, info, do_upload, entity_manager)
                        .wrap_err("Failed to release authority")?;
                    return Ok(do_upload);
                }
            }
        }
        if let Some(var) = entity_manager.get_var(const { VarName::from_str("ew_was_stealable") }) {
            let n = var.value_int()?;
            if n == 1 {
                if let Some(cost) =
                    entity_manager.try_get_first_component::<ItemCostComponent>(ComponentTag::None)
                {
                    let (cx, cy) = entity_manager.camera_pos();
                    if ((cx - x) as f32).powi(2) + ((cy - y) as f32).powi(2) < 256.0 * 256.0 {
                        cost.set_stealable(true)?;
                        entity_manager.remove_component(var)?;
                    }
                }
                if let Some(vel) =
                    entity_manager.try_get_first_component::<VelocityComponent>(ComponentTag::None)
                {
                    vel.set_gravity_y(400.0)?;
                    vel.set_air_friction(0.55)?;
                }
            } else if n == 0 {
                var.set_value_int(48)?;
                if let Some(vel) =
                    entity_manager.try_get_first_component::<VelocityComponent>(ComponentTag::None)
                {
                    vel.set_gravity_y(0.0)?;
                    vel.set_air_friction(10.0)?;
                }
            } else {
                var.set_value_int(n - 1)?;
                if let Some(vel) =
                    entity_manager.try_get_first_component::<VelocityComponent>(ComponentTag::None)
                {
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
        entity_manager: &mut EntityManager,
    ) -> Result<(), eyre::Error> {
        self.untrack_entity(ctx, gid, lid, Some(entity.0))?;
        entity_manager.remove_tag(const { CachedTag::from_tag(DES_TAG) })?;
        with_entity_scripts(entity_manager, |luac| {
            luac.set_script_throw_item(
                "mods/quant.ew/files/system/entity_sync_helper/item_notify.lua".into(),
            )
        })?;
        for entity in entity.children(None) {
            with_entity_scripts_no_mgr(entity, |luac| {
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
                    max_hp: info.max_hp,
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
        entity_manager: &mut EntityManager,
    ) -> eyre::Result<()> {
        let entity = self._release_authority_update_data(ctx, gid, lid, info, do_upload)?;
        ctx.net.send(&NoitaOutbound::DesToProxy(
            shared::des::DesToProxy::ReleaseAuthority(gid),
        ))?;
        self.pending_removal.push(lid);
        entity_manager.set_current_entity(entity)?;
        safe_entitykill(entity_manager);
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
        entity_manager: &mut EntityManager,
    ) -> eyre::Result<()> {
        let entity = self._release_authority_update_data(ctx, gid, lid, info, do_upload)?;
        ctx.net.send(&NoitaOutbound::DesToProxy(
            shared::des::DesToProxy::TransferAuthorityTo(gid, peer),
        ))?;
        self.pending_removal.push(lid);
        entity_manager.set_current_entity(entity)?;
        safe_entitykill(entity_manager);
        Ok(())
    }
}

impl LocalDiffModel {
    fn alloc_lid(&mut self) -> Lid {
        let ret = self.next_lid;
        self.next_lid.0 += 1;
        ret
    }

    pub(crate) fn track_entity(
        &mut self,
        entity: EntityID,
        gid: Gid,
        entity_manager: &mut EntityManager,
    ) -> eyre::Result<Lid> {
        entity_manager.set_current_entity(entity)?;
        self.wait_to_transfer = 16;
        let lid = self.alloc_lid();
        let should_not_serialize = entity_manager
            .remove_all_components_of_type::<CameraBoundComponent>(ComponentTag::None)?
            || (entity.is_alive() && entity_manager.check_all_phys_init()? && entity.get_physics_body_ids().unwrap_or_default()
                .len()
                == entity_manager
                    .iter_all_components_of_type_including_disabled::<PhysicsBodyComponent>(ComponentTag::None)
                    .count()
                    + entity_manager
                        .iter_all_components_of_type_including_disabled::<PhysicsBody2Component>(
                            ComponentTag::None,
                        )
                        .count());
        entity_manager.add_tag(const { CachedTag::from_tag(DES_TAG) })?;
        if let Some(ghost) = entity_manager
            .try_get_first_component_including_disabled::<GhostComponent>(ComponentTag::None)
        {
            ghost.set_target_tag("".into())?;
        }

        self.tracker.tracked.insert(lid, entity);

        let (x, y) = entity.position()?;

        if entity_manager.has_tag(const { CachedTag::from_tag("card_action") })
            && let Some(cost) =
                entity_manager.try_get_first_component::<ItemCostComponent>(ComponentTag::None)
            && cost.stealable()?
        {
            cost.set_stealable(false)?;
            entity_manager.get_var_or_default(const { VarName::from_str("ew_was_stealable") })?;
        }

        let entity_kind = classify_entity(entity)?;
        let spawn_info = match entity_kind {
            EntityKind::Normal if should_not_serialize => {
                EntitySpawnInfo::Filename(entity.filename()?.to_string())
            }
            _ => EntitySpawnInfo::Serialized {
                //serialized_at: game_get_frame_num()?,
                data: serialize_entity(entity)?, //TODO we never update this?
            },
        };
        with_entity_scripts(entity_manager, |scripts| {
            scripts.set_script_death(
                "mods/quant.ew/files/system/entity_sync_helper/death_notify.lua".into(),
            )
        })?;
        let n = entity_manager.get_var(const { VarName::from_str("ew_gid_lid") });
        if let Some(lua) = n {
            entity_manager.remove_component(lua)?;
        }
        let var = entity_manager.add_component::<VariableStorageComponent>()?;
        var.set_name("ew_gid_lid".into())?;
        var.set_value_string(gid.0.to_string().into())?;
        var.set_value_int(i32::from_le_bytes(lid.0.to_le_bytes()))?;
        var.set_value_bool(true)?;

        if entity_manager.has_tag(const { CachedTag::from_tag("card_action") })
            && let Some(vel) =
                entity_manager.try_get_first_component::<VelocityComponent>(ComponentTag::None)
        {
            vel.set_gravity_y(0.0)?;
            vel.set_air_friction(10.0)?;
        }

        if entity_manager
            .try_get_first_component::<BossDragonComponent>(ComponentTag::None)
            .is_some()
            && entity_manager
                .try_get_first_component::<StreamingKeepAliveComponent>(ComponentTag::None)
                .is_none()
        {
            entity_manager.add_component::<StreamingKeepAliveComponent>()?;
        }

        let is_global = entity_manager
            .try_get_first_component_including_disabled::<BossHealthBarComponent>(
                ComponentTag::None,
            )
            .is_some()
            || entity_manager
                .try_get_first_component::<StreamingKeepAliveComponent>(ComponentTag::None)
                .is_some();

        if is_global {
            self.tracker.global_entities.insert(entity);
        }

        let drops_gold = (entity_manager
            .iter_all_components_of_type::<LuaComponent>(ComponentTag::None)
            .any(|lua| {
                lua.script_death().ok() == Some("data/scripts/items/drop_money.lua".into())
            })
            && entity_manager
                .iter_all_components_of_type::<VariableStorageComponent>(ComponentTag::None)
                .all(|var| !var.has_tag("no_gold_drop")))
            || (entity_manager.has_tag(const { CachedTag::from_tag("boss_dragon") })
                && entity_manager
                    .iter_all_components_of_type::<LuaComponent>(ComponentTag::None)
                    .any(|lua| {
                        lua.script_death().ok()
                            == Some("data/scripts/animals/boss_dragon_death.lua".into())
                    }))
            || entity_manager
                .get_var(const { VarName::from_str("throw_time") })
                .map(|v| v.value_int().ok() != Some(-1))
                .unwrap_or(false);

        self.entity_entries.insert(
            lid,
            EntityEntryPair {
                last: None,
                current: Some(EntityInfo {
                    spawn_info,
                    kind: entity_kind,
                    x: x as f32,
                    y: y as f32,
                    r: 0.0,
                    vx: 0.0,
                    vy: 0.0,
                    hp: 1.0,
                    max_hp: 1.0,
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
                }),
                gid,
            },
        );

        Ok(lid)
    }

    pub(crate) fn track_and_upload_entity(
        &mut self,
        entity: EntityID,
        entity_manager: &mut EntityManager,
    ) -> eyre::Result<()> {
        let gid = Gid(rand::random());
        let lid = self.track_entity(entity, gid, entity_manager)?;
        self.upload.insert(lid);
        Ok(())
    }

    pub(crate) fn phys_later(&mut self, entity_manager: &mut EntityManager) -> eyre::Result<()> {
        for (entity, phys) in self.phys_later.drain(..) {
            entity_manager.set_current_entity(entity)?;
            if entity.is_alive() && entity_manager.check_all_phys_init()? {
                let phys_bodies = entity.get_physics_body_ids().unwrap_or_default();
                for (p, physics_body_id) in phys.iter().zip(phys_bodies.iter()) {
                    let Some(p) = p else {
                        continue;
                    };
                    let (x, y) =
                        noita_api::raw::game_pos_to_physics_pos(p.x.into(), Some(p.y.into()))?;
                    physics_body_id.set_transform(
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

    pub(crate) fn enable_later(&mut self, entity_manager: &mut EntityManager) -> eyre::Result<()> {
        for entity in self.enable_later.drain(..) {
            if entity.is_alive() {
                entity_manager.set_current_entity(entity)?;
                entity_manager.set_components_with_tag_enabled(
                    const { ComponentTag::from_str("disabled_at_start") },
                    true,
                )?;
                entity_manager.set_components_with_tag_enabled(
                    const { ComponentTag::from_str("enabled_at_start") },
                    false,
                )?;
                entity
                    .children(Some("protection".into()))
                    .for_each(|ent| ent.kill());
                entity_manager.add_tag(const { CachedTag::from_tag("boss_centipede_active") })?;
                entity.set_static(false)?
            }
        }
        Ok(())
    }

    pub(crate) fn update_pending_authority(
        &mut self,
        start: Instant,
        entity_manager: &mut EntityManager,
    ) -> eyre::Result<Instant> {
        while let Some(entity_data) = self.tracker.pending_authority.pop() {
            let entity = spawn_entity_by_data(
                &entity_data.data,
                entity_data.pos.x as f32,
                entity_data.pos.y as f32,
                entity_manager,
            )?;
            entity.set_position(entity_data.pos.x as f64, entity_data.pos.y as f64, None)?;
            if entity_data.is_charmed {
                if entity_manager.has_tag(const { CachedTag::from_tag("boss_centipede") }) {
                    self.enable_later.push(entity);
                } else if entity_manager.has_tag(const { CachedTag::from_tag("pitcheck_b") }) {
                    entity_manager
                        .entity()
                        .set_components_with_tag_enabled("disabled".into(), true)?;
                } else if let Some(var) =
                    entity_manager.get_var(const { VarName::from_str("active") })
                {
                    var.set_value_int(1)?;
                    entity_manager.set_components_with_tag_enabled(
                        const { ComponentTag::from_str("activate") },
                        true,
                    )?
                } else {
                    entity.set_game_effects(&[GameEffectData::Normal(GameEffectEnum::Charm)])?
                }
            }
            for (name, s, i, f, b) in &entity_data.synced_var {
                let v = entity_manager.get_var_or_default_unknown(name)?;
                v.set_value_string(s.into())?;
                v.set_value_int(*i)?;
                v.set_value_float(*f)?;
                v.set_value_bool(*b)?;
            }
            if !entity_data.phys.is_empty() {
                self.phys_later.push((entity, entity_data.phys));
            }

            mom(entity_manager, entity_data.counter, None)?;
            sun(entity_manager, entity_data.counter)?;
            if entity_data.hp != -1.0
                && let Some(damage) = entity_manager
                    .try_get_first_component::<DamageModelComponent>(ComponentTag::None)
            {
                damage.set_max_hp(entity_data.max_hp as f64)?;
                damage.set_hp(entity_data.hp as f64)?;
            }
            if !entity_data.drops_gold {
                let n = entity_manager
                    .iter_all_components_of_type::<LuaComponent>(ComponentTag::None)
                    .find(|lua| {
                        lua.script_death().ok() == Some("data/scripts/items/drop_money.lua".into())
                    });
                if let Some(lua) = n {
                    entity_manager.remove_component(lua)?
                }
            } else if entity_manager.has_tag(const { CachedTag::from_tag("boss_dragon") }) {
                let lua = entity_manager.add_component::<LuaComponent>()?;
                lua.set_script_death("data/scripts/animals/boss_dragon_death.lua".into())?;
                lua.set_execute_every_n_frame(-1)?;
            }
            if let Some(wand) = entity_data.wand {
                give_wand(entity, &wand, None, false, None, entity_manager)?;
            }
            let lid = self.track_entity(entity, entity_data.gid, entity_manager)?;
            self.dont_upload.insert(lid);

            // Don't handle too much in one frame to avoid stutters.
            if start.elapsed().as_micros() > 1000 {
                break;
            }
        }
        Ok(start)
    }

    #[allow(clippy::type_complexity)]
    pub(crate) fn update_tracked_entities(
        &mut self,
        ctx: &mut ModuleCtx,
        start: usize,
        tmr: Instant,
        entity_manager: &mut EntityManager,
    ) -> eyre::Result<(Vec<(WorldPos, SpawnOnce)>, usize)> {
        self.update_buffer.clear();
        let (cam_x, cam_y) = entity_manager.camera_pos();
        let cam_x = cam_x as f32;
        let cam_y = cam_y as f32;
        let mut dead = Vec::with_capacity(self.tracker.pending_death_notify.len());
        let mut to_untrack = Vec::with_capacity(self.tracker.pending_death_notify.len());
        for (killed, wait_on_kill, pos, file, responsible) in
            self.tracker.pending_death_notify.drain(..)
        {
            let responsible_peer = responsible
                .and_then(|ent| ctx.player_map.get_by_right(&ent))
                .copied();
            let Some((lid, _)) = self.tracker.tracked.remove_by_right(&killed) else {
                continue;
            };
            let drops_gold = if let Some(info) = self.entity_entries.remove(&lid) {
                to_untrack.push((info.gid, lid));
                info.current.unwrap().drops_gold
            } else {
                false
            };
            self.update_buffer.push(EntityUpdate::KillEntity {
                lid,
                wait_on_kill,
                responsible_peer,
            });
            dead.push((pos, SpawnOnce::Enemy(file, drops_gold, responsible_peer)));
            self.tracker.global_entities.remove(&killed);
            self.upload.remove(&lid);
            self.dont_save.remove(&lid);
            entity_manager.remove_ent(&killed);
        }
        for (gid, lid) in to_untrack {
            self.tracker.untrack_entity(ctx, gid, lid, None)?
        }
        let mut should_transfer = false;
        if let Some(pe) = ctx.player_map.get_by_left(&my_peer_id()) {
            let (px, py) = pe.position()?;
            should_transfer = (px as f32 - cam_x).powi(2) + (py as f32 - cam_y).powi(2)
                > AUTHORITY_RADIUS.powi(2);
        }
        if !should_transfer {
            self.wait_to_transfer = 16
        } else {
            self.wait_to_transfer = self.wait_to_transfer.saturating_sub(1)
        }
        let l = self.entity_entries.len();
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
                    current.as_mut().unwrap(),
                    lid,
                    (cam_x, cam_y),
                    self.upload.contains(&lid) && !self.dont_upload.contains(&lid),
                    should_transfer && self.wait_to_transfer == 0,
                    !self.dont_save.contains(&lid),
                    entity_manager,
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
                    if self.tracker.pending_removal.contains(&lid) {
                        continue;
                    }
                    let Some(last) = last.as_mut() else {
                        *last = current.clone();
                        self.init_buffer.push(EntityInit {
                            info: std::mem::take(current).unwrap(),
                            lid,
                            gid: *gid,
                        });
                        continue;
                    };
                    let Some(current) = current else {
                        unreachable!()
                    };
                    let mut had_any_delta = false;
                    fn diff<T: PartialEq + Clone, K: Fn() -> EntityUpdate>(
                        current: &T,
                        last: &mut T,
                        update: K,
                        res: &mut Vec<EntityUpdate>,
                        had_any_delta: &mut bool,
                        lid: Lid,
                    ) {
                        if current != last {
                            if !*had_any_delta {
                                *had_any_delta = true;
                                res.push(EntityUpdate::CurrentEntity(lid));
                            }
                            res.push(update());
                            *last = current.clone();
                        }
                    }
                    if match (&current.wand, &last.wand) {
                        (Some((_, _, a)), Some((_, _, b))) => a != b,
                        (Some(_), None) | (None, Some(_)) => true,
                        (None, None) => false,
                    } {
                        had_any_delta = true;
                        self.update_buffer.push(EntityUpdate::CurrentEntity(lid));
                        self.update_buffer
                            .push(EntityUpdate::SetWand(current.wand.clone()));
                        last.wand = current.wand.clone();
                    }
                    diff(
                        &current.wand_rotation,
                        &mut last.wand_rotation,
                        || EntityUpdate::SetWandRotation(current.wand_rotation),
                        &mut self.update_buffer,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.laser,
                        &mut last.laser,
                        || EntityUpdate::SetLaser(current.laser),
                        &mut self.update_buffer,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &(current.x, current.y),
                        &mut (last.x, last.y),
                        || EntityUpdate::SetPosition(current.x, current.y),
                        &mut self.update_buffer,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &(current.vx, current.vy),
                        &mut (last.vx, last.vy),
                        || EntityUpdate::SetVelocity(current.vx, current.vy),
                        &mut self.update_buffer,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.hp,
                        &mut last.hp,
                        || EntityUpdate::SetHp(current.hp),
                        &mut self.update_buffer,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.max_hp,
                        &mut last.max_hp,
                        || EntityUpdate::SetMaxHp(current.max_hp),
                        &mut self.update_buffer,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.animations,
                        &mut last.animations,
                        || EntityUpdate::SetAnimations(current.animations.clone()),
                        &mut self.update_buffer,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.synced_var,
                        &mut last.synced_var,
                        || EntityUpdate::SetSyncedVar(current.synced_var.clone()),
                        &mut self.update_buffer,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.facing_direction,
                        &mut last.facing_direction,
                        || EntityUpdate::SetFacingDirection(current.facing_direction),
                        &mut self.update_buffer,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.r,
                        &mut last.r,
                        || EntityUpdate::SetRotation(current.r),
                        &mut self.update_buffer,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.phys,
                        &mut last.phys,
                        || EntityUpdate::SetPhysInfo(current.phys.clone()),
                        &mut self.update_buffer,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.cost,
                        &mut last.cost,
                        || EntityUpdate::SetCost(current.cost),
                        &mut self.update_buffer,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.current_stains,
                        &mut last.current_stains,
                        || EntityUpdate::SetStains(current.current_stains),
                        &mut self.update_buffer,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.game_effects,
                        &mut last.game_effects,
                        || EntityUpdate::SetGameEffects(current.game_effects.clone()),
                        &mut self.update_buffer,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.ai_rotation,
                        &mut last.ai_rotation,
                        || EntityUpdate::SetAiRotation(current.ai_rotation),
                        &mut self.update_buffer,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.ai_state,
                        &mut last.ai_state,
                        || EntityUpdate::SetAiState(current.ai_state),
                        &mut self.update_buffer,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.limbs,
                        &mut last.limbs,
                        || EntityUpdate::SetLimbs(current.limbs.clone()),
                        &mut self.update_buffer,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.is_enabled,
                        &mut last.is_enabled,
                        || EntityUpdate::SetIsEnabled(current.is_enabled),
                        &mut self.update_buffer,
                        &mut had_any_delta,
                        lid,
                    );
                    diff(
                        &current.counter,
                        &mut last.counter,
                        || EntityUpdate::SetCounter(current.counter),
                        &mut self.update_buffer,
                        &mut had_any_delta,
                        lid,
                    );
                }
            }
            if tmr.elapsed().as_micros() > 3000 {
                end = (start + i + 1) % l;
                break;
            }
        }
        for (lid, peer) in self.tracker.pending_localize.drain(..) {
            self.update_buffer
                .push(EntityUpdate::LocalizeEntity(lid, peer));
        }

        for lid in self.tracker.pending_removal.drain(..) {
            self.update_buffer.push(EntityUpdate::RemoveEntity(lid));
            // "Untrack" entity
            let ent = self.tracker.tracked.remove_by_left(&lid);
            self.entity_entries.remove(&lid);
            self.upload.remove(&lid);
            self.dont_save.remove(&lid);
            if let Some((_, ent)) = ent {
                entity_manager.remove_ent(&ent);
            }
        }
        Ok((dead, end))
    }
    pub(crate) fn make_init(&mut self) {
        for (lid, EntityEntryPair { current, gid, .. }) in self.entity_entries.iter_mut() {
            //res.push(EntityUpdate::CurrentEntity(*lid));
            //*last = Some(current.clone());
            self.init_buffer.push(EntityInit {
                info: std::mem::take(current).unwrap(),
                lid: *lid,
                gid: *gid,
            });
        }
    }
    pub(crate) fn uninit(&mut self) {
        for EntityInit { info, lid, .. } in self.init_buffer.drain(..) {
            self.entity_entries.get_mut(&lid).unwrap().current = Some(info);
        }
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

    pub(crate) fn entity_grabbed(
        &mut self,
        source: PeerId,
        lid: Lid,
        net: &mut NetManager,
        entity_manager: &mut EntityManager,
    ) -> eyre::Result<()> {
        let Some(info) = self.entity_entries.get(&lid) else {
            return Ok(());
        };
        if let Ok(entity) = self.tracker.entity_by_lid(lid) {
            if info.current.as_ref().unwrap().kind == EntityKind::Item {
                self.tracker.pending_localize.push((lid, source));
                entity_manager.set_current_entity(entity)?;
                safe_entitykill(entity_manager);
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
        Ok(())
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
        let phys_bodies = entity.get_physics_body_ids().unwrap_or_default();
        phys_bodies
            .into_iter()
            .map(|body| -> eyre::Result<Option<PhysBodyInfo>> {
                Ok(body.get_transform()?.and_then(|data| {
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
                }))
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
    pub(crate) fn apply_init(
        &mut self,
        diff: Vec<EntityInit>,
        entity_manager: &mut EntityManager,
        em: &mut noita_api::noita::types::EntityManager,
    ) -> eyre::Result<Vec<EntityID>> {
        let mut dont_kill = Vec::with_capacity(self.waiting_for_lid.len());
        for info in diff {
            if let Some(ent) = self.waiting_for_lid.remove(&info.gid) {
                self.tracked.insert(info.lid, ent);
                let _ = init_remote_entity(
                    ent,
                    Some(info.lid),
                    Some(info.gid),
                    false,
                    entity_manager,
                    em,
                );
                dont_kill.push(ent);
            }
            self.lid_to_gid.insert(info.lid, info.gid);
            self.entity_infos.insert(info.lid, info.info);
        }
        Ok(dont_kill)
    }
    pub(crate) fn apply_diff(
        &mut self,
        diff: Vec<EntityUpdate>,
        entity_manager: &mut EntityManager,
    ) -> eyre::Result<()> {
        let empty_data = &mut EntityInfo::default();
        let mut ent_data = &mut EntityInfo::default();
        for entry in diff {
            match entry {
                EntityUpdate::CurrentEntity(lid) => {
                    ent_data = self.entity_infos.get_mut(&lid).unwrap_or(empty_data);
                }
                EntityUpdate::LocalizeEntity(lid, peer_id) => {
                    if let Some((_, entity)) = self.tracked.remove_by_left(&lid)
                        && peer_id != my_peer_id()
                    {
                        entity_manager.set_current_entity(entity)?;
                        safe_entitykill(entity_manager);
                    }
                    self.entity_infos.remove(&lid);
                    ent_data = empty_data;
                }
                EntityUpdate::RemoveEntity(lid) => self.pending_remove.push(lid),
                EntityUpdate::KillEntity {
                    lid,
                    wait_on_kill,
                    responsible_peer,
                } => self
                    .pending_death_notify
                    .push((lid, wait_on_kill, responsible_peer)),
                EntityUpdate::SetPosition(x, y) => (ent_data.x, ent_data.y) = (x, y),
                EntityUpdate::SetRotation(r) => ent_data.r = r,
                EntityUpdate::SetVelocity(vx, vy) => (ent_data.vx, ent_data.vy) = (vx, vy),
                EntityUpdate::SetHp(hp) => ent_data.hp = hp,
                EntityUpdate::SetMaxHp(max_hp) => ent_data.max_hp = max_hp,
                EntityUpdate::SetFacingDirection(direction) => {
                    ent_data.facing_direction = direction
                }
                EntityUpdate::SetPhysInfo(vec) => ent_data.phys = vec,
                EntityUpdate::SetCost(cost) => ent_data.cost = cost,
                EntityUpdate::SetAnimations(ani) => ent_data.animations = ani,
                EntityUpdate::SetStains(stains) => ent_data.current_stains = stains,
                EntityUpdate::SetGameEffects(effects) => ent_data.game_effects = effects,
                EntityUpdate::SetAiState(state) => ent_data.ai_state = state,
                EntityUpdate::SetWand(wand) => ent_data.wand = wand,
                EntityUpdate::SetWandRotation(rot) => ent_data.wand_rotation = rot,
                EntityUpdate::SetLaser(peer) => ent_data.laser = peer,
                EntityUpdate::SetAiRotation(rot) => ent_data.ai_rotation = rot,
                EntityUpdate::SetLimbs(limbs) => ent_data.limbs = limbs,
                EntityUpdate::SetSyncedVar(vars) => ent_data.synced_var = vars,
                EntityUpdate::SetIsEnabled(enabled) => ent_data.is_enabled = enabled,
                EntityUpdate::SetCounter(orbs) => ent_data.counter = orbs,
            }
        }
        Ok(())
    }

    fn inner(
        &self,
        ctx: &mut ModuleCtx,
        entity_info: &EntityInfo,
        entity: EntityID,
        lid: &Lid,
        entity_manager: &mut EntityManager,
    ) -> eyre::Result<Option<Lid>> {
        if entity_info.kind == EntityKind::Item && item_in_my_inventory(entity)?
            || item_in_entity_inventory(entity)?
        {
            entity_manager.remove_tag(const { CachedTag::from_tag(DES_TAG) })?;
            with_entity_scripts(entity_manager, |luac| {
                luac.set_script_throw_item(
                    "mods/quant.ew/files/system/entity_sync_helper/item_notify.lua".into(),
                )
            })?;
            for entity in entity.children(None) {
                with_entity_scripts_no_mgr(entity, |luac| {
                    luac.set_script_throw_item(
                        "mods/quant.ew/files/system/entity_sync_helper/item_notify.lua".into(),
                    )
                })?;
            }
            return Ok(Some(*lid));
        }
        for (name, s, i, f, b) in &entity_info.synced_var {
            let v = entity_manager.get_var_or_default_unknown(name)?;
            v.set_value_string(s.into())?;
            v.set_value_int(*i)?;
            v.set_value_float(*f)?;
            v.set_value_bool(*b)?;
        }
        mom(
            entity_manager,
            entity_info.counter,
            Some(entity_info.cost as i32),
        )?;
        sun(entity_manager, entity_info.counter)?;

        if let Some((gid, seri, _)) = &entity_info.wand {
            give_wand(
                entity,
                seri,
                *gid,
                true,
                Some(entity_info.wand_rotation),
                entity_manager,
            )?;
        } else if let Some(inv) = entity
            .children(None)
            .find(|e| e.name().unwrap_or("".into()) == "inventory_quick")
        {
            inv.children(None).for_each(|e| e.kill())
        }
        if entity_info.is_enabled {
            if entity_manager
                .get_var(const { VarName::from_str("ew_has_started") })
                .is_none()
            {
                entity_manager.set_components_with_tag_enabled(
                    const { ComponentTag::from_str("disabled_at_start") },
                    true,
                )?;
                entity_manager.set_components_with_tag_enabled(
                    const { ComponentTag::from_str("enabled_at_start") },
                    false,
                )?;
                entity_manager.add_tag(const { CachedTag::from_tag("boss_centipede_active") })?;
                let mut to_remove = Vec::new();
                for lua in entity_manager
                    .iter_all_components_of_type_including_disabled::<LuaComponent>(
                        ComponentTag::None,
                    )
                {
                    if [
                        "data/entities/animals/boss_centipede/boss_centipede_before_fight.lua",
                        "data/entities/animals/boss_centipede/boss_centipede_update.lua",
                    ]
                    .contains(&&*lua.script_source_file()?)
                    {
                        to_remove.push(lua);
                    }
                }
                for lua in to_remove {
                    entity_manager.remove_component(lua)?;
                }
                let immortal = entity_manager.add_component::<LuaComponent>()?;
                immortal.add_tag("ew_immortal")?;
                immortal.set_script_damage_about_to_be_received(
                    "mods/quant.ew/files/system/entity_sync_helper/immortal.lua".into(),
                )?;
                entity_manager
                    .add_component::<VariableStorageComponent>()?
                    .set_name("ew_has_started".into())?;
                entity
                    .children(Some("protection".into()))
                    .for_each(|ent| ent.kill());
            } else if let Some(var) = entity_manager.get_var(const { VarName::from_str("active") })
            {
                var.set_value_int(1)?;
                entity_manager.set_components_with_tag_enabled(
                    const { ComponentTag::from_str("activate") },
                    true,
                )?
            }
        } else if let Some(var) = entity_manager.get_var(const { VarName::from_str("active") }) {
            var.set_value_int(0)?;
            entity_manager.set_components_with_tag_enabled(
                const { ComponentTag::from_str("activate") },
                false,
            )?
        }
        for (ent, (x, y)) in entity
            .children(None)
            .filter(|ent| ent.get_first_component::<IKLimbComponent>(None).is_ok())
            .zip(&entity_info.limbs)
        {
            if let Ok(limb) = ent.get_first_component::<IKLimbComponent>(None) {
                limb.set_end_position((*x, *y))?;
            }
            if let Ok(limb) = ent.get_first_component::<IKLimbWalkerComponent>(None) {
                entity_manager.remove_component(limb)?
            };
            if let Ok(limb) = ent.get_first_component::<IKLimbAttackerComponent>(None) {
                entity_manager.remove_component(limb)?
            };
            if let Ok(limb) = ent.get_first_component::<IKLimbsAnimatorComponent>(None) {
                entity_manager.remove_component(limb)?
            };
        }
        let m = *ctx.fps_by_player.get(&self.peer_id).unwrap_or(&60) as f32
            / *ctx.fps_by_player.get(&my_peer_id()).unwrap_or(&60) as f32;
        let (vx, vy) = (entity_info.vx * m, entity_info.vy * m);
        if entity_info.phys.is_empty()
            || (entity_info.is_enabled
                && entity_manager.has_tag(const { CachedTag::from_tag("boss_centipede") }))
        {
            let should_send_position = if let Some(com) =
                entity_manager.try_get_first_component::<ItemComponent>(ComponentTag::None)
            {
                !com.play_hover_animation()?
            } else {
                true
            };

            let should_send_rotation = if let Some(com) =
                entity_manager.try_get_first_component::<ItemComponent>(ComponentTag::None)
            {
                !com.play_spinning_animation()? || com.play_hover_animation()?
            } else {
                true
            };
            if should_send_rotation && should_send_position {
                entity.set_position(
                    entity_info.x as f64,
                    entity_info.y as f64,
                    Some(entity_info.r as f64),
                )?;
            } else if should_send_position {
                entity.set_position(entity_info.x as f64, entity_info.y as f64, None)?;
            } else if should_send_rotation {
                let (x, y) = entity.position()?;
                entity.set_position(x, y, Some(entity_info.r as f64))?;
            }
            if let Some(worm) =
                entity_manager.try_get_first_component::<BossDragonComponent>(ComponentTag::None)
            {
                worm.set_m_target_vec((vx, vy))?;
            } else if let Some(worm) =
                entity_manager.try_get_first_component::<WormComponent>(ComponentTag::None)
            {
                worm.set_m_target_vec((vx, vy))?;
            } else if let Some(vel) =
                entity_manager.try_get_first_component::<CharacterDataComponent>(ComponentTag::None)
            {
                vel.set_m_velocity((vx, vy))?;
            } else if let Some(vel) =
                entity_manager.try_get_first_component::<VelocityComponent>(ComponentTag::None)
            {
                vel.set_m_velocity((vx, vy))?;
            }
        }
        if let Some(damage) =
            entity_manager.try_get_first_component::<DamageModelComponent>(ComponentTag::None)
        {
            damage.set_max_hp(entity_info.max_hp as f64)?;

            let current_hp = damage.hp()? as f32;
            if current_hp > entity_info.hp {
                let old = damage.object_get_value::<f64>("damage_multipliers", "curse")?;
                if old != 1.0 {
                    damage.object_set_value("damage_multipliers", "curse", 1.0)?
                }
                entity.inflict_damage(
                    (current_hp - entity_info.hp) as f64,
                    DamageType::DamageCurse,
                    "hp sync",
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
                entity.inflict_damage(
                    (current_hp - entity_info.hp) as f64,
                    DamageType::DamageHealing,
                    "hp sync",
                    None,
                )?;
                if old != 0.0 {
                    damage.object_set_value("damage_multipliers", "healing", old)?
                }
                damage.set_hp(entity_info.hp as f64)?;
            }
        }

        if !entity_info.phys.is_empty() && entity_manager.check_all_phys_init()? {
            let phys_bodies = entity.get_physics_body_ids().unwrap_or_default();
            for (p, physics_body_id) in entity_info.phys.iter().zip(phys_bodies.iter()) {
                let Some(p) = p else {
                    continue;
                };
                let (x, y) = noita_api::raw::game_pos_to_physics_pos(p.x.into(), Some(p.y.into()))?;
                physics_body_id.set_transform(
                    x,
                    y,
                    p.angle.into(),
                    (p.vx * m).into(),
                    (p.vy * m).into(),
                    (p.av * m).into(),
                )?;
            }
        }

        if let Some(cost) =
            entity_manager.try_get_first_component::<ItemCostComponent>(ComponentTag::None)
        {
            cost.set_cost(entity_info.cost)?;
            if entity_info.cost == 0 {
                entity_manager.set_components_with_tag_enabled(
                    const { ComponentTag::from_str("shop_cost") },
                    false,
                )?;
            }
        }

        entity.set_game_effects(&entity_info.game_effects)?;

        if entity_manager
            .get_var(const { VarName::from_str("rolling") })
            .is_some()
        {
            let var = entity_manager.get_var_or_default(const { VarName::from_str("ew_rng") })?;
            let bytes = entity_info.current_stains.to_le_bytes();
            let is_rolling = bytes[0];
            let bytes: [u8; 4] = [bytes[4], bytes[5], bytes[6], bytes[7]];
            let rng = i32::from_le_bytes(bytes);
            var.set_value_int(rng)?;
            let var = entity_manager.get_var_or_default(const { VarName::from_str("rolling") })?;
            if is_rolling == 1 {
                if var.value_int()? == 0 {
                    var.set_value_int(4)?;
                    entity_manager
                        .iter_all_components_of_type::<SpriteComponent>(ComponentTag::None)
                        .for_each(|s| {
                            let _ = s.set_rect_animation("roll".into());
                        })
                } else if var.value_int()? == 8 {
                    let (x, y) = entity.position()?;
                    if !EntityID::get_in_radius_with_tag(x, y, 480.0, "player_unit")?.is_empty() {
                        game_print("$item_die_roll");
                    }
                }
            } else {
                var.set_value_int(0)?;
            }
        } else {
            entity_manager.set_current_stains(entity_info.current_stains)?;
        }
        if let Some(ai) = entity_manager
            .try_get_first_component_including_disabled::<AnimalAIComponent>(ComponentTag::None)
        {
            ai.set_ai_state(entity_info.ai_state)?;
            ai.set_m_ranged_attack_current_aim_angle(entity_info.ai_rotation)?;
        } else {
            let mut files = std::mem::take(&mut entity_manager.files);
            let sprites =
                entity_manager.iter_all_components_of_type::<SpriteComponent>(ComponentTag::None);
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
                let text = noita_api::get_file(&mut files, file)?;
                if let Some(ani) = text.get(*animation as usize) {
                    sprite.set_rect_animation(ani.into())?;
                    sprite.set_next_rect_animation(ani.into())?;
                }
            }
            entity_manager.files = files;
        }
        let laser =
            entity_manager.try_get_first_component::<LaserEmitterComponent>(ComponentTag::None);
        if entity_info.laser != Target::None {
            let laser = if let Some(laser) = laser {
                laser
            } else {
                let laser = entity_manager.add_component::<LaserEmitterComponent>()?;
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
            let ent = match entity_info.laser {
                Target::Peer(peer) => ctx.player_map.get_by_left(&peer).cloned(),
                Target::Gid(gid) => self.find_by_gid(gid),
                Target::None => None,
            };
            if let Some(ent) = ent {
                let (x, y) = entity.position()?;
                let (tx, ty) = ent.position()?;
                if !raytrace_platforms(x, y, tx, ty)?.0 {
                    laser.set_is_emitting(true)?;
                    let (dx, dy) = (tx - x, ty - y);
                    let theta = dy.atan2(dx);
                    laser.set_laser_angle_add_rad((theta - entity.rotation()?) as f32)?;
                    laser.object_set_value("laser", "max_length", dx.hypot(dy))?;
                } else {
                    laser.set_is_emitting(false)?;
                }
            }
        } else if let Some(laser) = laser {
            laser.set_is_emitting(false)?;
        }
        Ok(None)
    }

    pub(crate) fn apply_entities(
        &mut self,
        ctx: &mut ModuleCtx,
        start: usize,
        tmr: Instant,
        entity_manager: &mut EntityManager,
    ) -> eyre::Result<usize> {
        let mut to_remove = Vec::new();
        let l = self.entity_infos.len();
        let mut end = None;
        let start = if start >= l { 0 } else { start };
        for (i, (lid, entity_info)) in self.entity_infos.iter().enumerate() {
            match self.tracked.get_by_left(lid) {
                Some(entity) if entity.is_alive() => {
                    entity_manager.set_current_entity(*entity)?;
                    if tmr.elapsed().as_micros() > 5000 || start > i {
                        if end.is_none() && start <= i {
                            end = Some(i);
                        }
                        if entity_info.phys.is_empty()
                            || (entity_info.is_enabled
                                && entity_manager
                                    .has_tag(const { CachedTag::from_tag("boss_centipede") }))
                        {
                            let should_send_position = if let Some(com) =
                                entity_manager
                                    .try_get_first_component::<ItemComponent>(ComponentTag::None)
                            {
                                !com.play_hover_animation()?
                            } else {
                                true
                            };
                            let should_send_rotation = if let Some(com) =
                                entity_manager
                                    .try_get_first_component::<ItemComponent>(ComponentTag::None)
                            {
                                !com.play_spinning_animation()? || com.play_hover_animation()?
                            } else {
                                true
                            };
                            if should_send_rotation && should_send_position {
                                entity.set_position(
                                    entity_info.x as f64,
                                    entity_info.y as f64,
                                    Some(entity_info.r as f64),
                                )?;
                            } else if should_send_position {
                                entity.set_position(
                                    entity_info.x as f64,
                                    entity_info.y as f64,
                                    None,
                                )?;
                            } else if should_send_rotation {
                                let (x, y) = entity.position()?;
                                entity.set_position(x, y, Some(entity_info.r as f64))?;
                            }
                        }
                    } else {
                        match self.inner(ctx, entity_info, *entity, lid, entity_manager) {
                            Ok(Some(lid)) => to_remove.push(lid),
                            Err(s) => print_error(s)?,
                            _ => {}
                        }
                    }
                }
                _ => {
                    if start <= i {
                        if tmr.elapsed().as_micros() > 5000 {
                            if end.is_none() {
                                end = Some(i);
                            }
                        } else {
                            if let Some(gid) = self.lid_to_gid.get(lid)
                                && ctx.dont_spawn.contains(gid)
                            {
                                continue;
                            }
                            let entity = spawn_entity_by_data(
                                &entity_info.spawn_info,
                                entity_info.x,
                                entity_info.y,
                                entity_manager,
                            )?;
                            init_remote_entity(
                                entity,
                                Some(*lid),
                                self.lid_to_gid.get(lid).copied(),
                                entity_info.drops_gold,
                                entity_manager,
                                ctx.globals.entity_manager,
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
        Ok(end.unwrap_or(0))
    }

    pub(crate) fn kill_entities(
        &mut self,
        ctx: &mut ModuleCtx,
        entity_manager: &mut EntityManager,
    ) -> eyre::Result<()> {
        for (lid, wait_on_kill, responsible) in self.pending_death_notify.drain(..) {
            let responsible_entity = responsible
                .and_then(|peer| ctx.player_map.get_by_left(&peer))
                .copied();
            self.entity_infos.remove(&lid);
            let Some(entity) = self.tracked.get_by_left(&lid).copied() else {
                continue;
            };
            entity_manager.set_current_entity(entity)?;
            if let Some(explosion) = entity_manager
                .try_get_first_component::<ExplodeOnDamageComponent>(ComponentTag::None)
            {
                explosion.set_explode_on_death_percent(1.0)?;
            }
            if let Some(inv) = entity
                .children(None)
                .find(|e| e.name().unwrap_or("".into()) == "inventory_quick")
            {
                inv.children(None).for_each(|e| e.kill())
            }
            if let Some(damage) =
                entity_manager.try_get_first_component::<DamageModelComponent>(ComponentTag::None)
            {
                entity_manager.remove_ent(&entity);
                entity
                    .children(Some("protection".into()))
                    .for_each(|ent| ent.kill());
                self.pending_remove.retain(|l| l != &lid);
                if !wait_on_kill {
                    damage.set_wait_for_kill_flag_on_death(false)?;
                }
                damage.object_set_value("damage_multipliers", "curse", 1.0)?;
                entity.inflict_damage(
                    damage.hp()? + f32::MIN_POSITIVE as f64,
                    DamageType::DamageCurse,
                    "kill sync",
                    responsible_entity,
                )?;
                damage.set_ui_report_damage(false)?;
                entity.inflict_damage(
                    damage.max_hp()? * 100.0,
                    DamageType::DamageCurse,
                    "kill sync",
                    responsible_entity,
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
                entity_manager.set_current_entity(entity)?;
                safe_entitykill(entity_manager);
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

            let _ = shooter_entity.shoot_projectile(
                projectile.position.0 as f64,
                projectile.position.1 as f64,
                projectile.target.0 as f64,
                projectile.target.1 as f64,
                deserialized,
            );
            if let Ok(Some(vel)) = deserialized.try_get_first_component::<VelocityComponent>(None)
                && let Some((vx, vy)) = projectile.vel
            {
                let _ = vel.set_m_velocity((vx, vy));
            }
        }
    }

    /*pub(crate) fn drain_backtrack(&mut self) -> impl DoubleEndedIterator<Item = EntityID> + '_ {
        self.backtrack.drain(..)
    }*/

    pub(crate) fn drain_grab_request(&mut self) -> impl DoubleEndedIterator<Item = Lid> + '_ {
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
    entity_manager: &mut EntityManager,
    em: &mut noita_api::noita::types::EntityManager,
) -> eyre::Result<()> {
    if entity.has_tag("player_unit") {
        entity.kill();
        entity_manager.remove_ent(&entity);
        return Ok(());
    }
    entity_manager.set_current_entity(entity)?;
    entity_manager.remove_all_components_of_type::<CameraBoundComponent>(ComponentTag::None)?;
    entity_manager
        .remove_all_components_of_type::<StreamingKeepAliveComponent>(ComponentTag::None)?;
    entity_manager
        .remove_all_components_of_type::<CharacterPlatformingComponent>(ComponentTag::None)?;
    entity_manager.remove_all_components_of_type::<PhysicsAIComponent>(ComponentTag::None)?;
    entity_manager.remove_all_components_of_type::<AdvancedFishAIComponent>(ComponentTag::None)?;
    entity_manager.remove_all_components_of_type::<IKLimbsAnimatorComponent>(ComponentTag::None)?;
    entity_manager.remove_all_components_of_type::<LifetimeComponent>(ComponentTag::None)?;
    let mut any = false;
    for ai in entity_manager
        .iter_all_components_of_type_including_disabled::<AIAttackComponent>(ComponentTag::None)
    {
        any = any || ai.attack_ranged_aim_rotation_enabled()?;
        ai.set_attack_ranged_entity_count_max(0)?;
        ai.set_attack_ranged_entity_count_min(0)?;
    }
    for ai in entity_manager
        .iter_all_components_of_type_including_disabled::<AnimalAIComponent>(ComponentTag::None)
    {
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
        entity_manager.remove_all_components_of_type::<AnimalAIComponent>(ComponentTag::None)?;
        entity_manager.remove_all_components_of_type::<AIAttackComponent>(ComponentTag::None)?;
        for sprite in entity_manager.iter_all_components_of_type::<SpriteComponent>(
            const { ComponentTag::from_str("character") },
        ) {
            sprite.remove_tag("character")?;
            sprite.set_has_special_scale(true)?;
        }
    }
    if let Some(w) = entity_manager
        .try_get_first_component_including_disabled::<WormComponent>(ComponentTag::None)
    {
        w.set_bite_damage(0.0)?;
    }
    if let Some(w) = entity_manager
        .try_get_first_component_including_disabled::<BossDragonComponent>(ComponentTag::None)
    {
        w.set_bite_damage(0.0)?;
    }
    entity_manager.add_tag(const { CachedTag::from_tag(DES_TAG) })?;
    entity_manager.add_tag(const { CachedTag::from_tag("polymorphable_NOT") })?;
    if lid.is_some()
        && let Some(damage) =
            entity_manager.try_get_first_component::<DamageModelComponent>(ComponentTag::None)
    {
        damage.set_wait_for_kill_flag_on_death(true)?;
        damage.set_physics_objects_damage(false)?;
    }

    for pb2 in
        entity_manager.iter_all_components_of_type::<PhysicsBody2Component>(ComponentTag::None)
    {
        pb2.set_destroy_body_if_entity_destroyed(true)?;
    }

    for expl in
        entity_manager.iter_all_components_of_type::<ExplodeOnDamageComponent>(ComponentTag::None)
    {
        expl.set_explode_on_damage_percent(0.0)?;
        expl.set_explode_on_death_percent(0.0)?;
        expl.set_physics_body_modified_death_probability(0.0)?;
    }

    if let Some(itemc) =
        entity_manager.try_get_first_component::<ItemCostComponent>(ComponentTag::None)
    {
        itemc.set_stealable(false)?;
    }

    let mut to_remove = Vec::new();
    for lua in entity_manager
        .iter_all_components_of_type_including_disabled::<LuaComponent>(ComponentTag::None)
    {
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
                "data/scripts/animals/mimic_damage.lua",
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
            to_remove.push(lua);
        }
    }
    for lua in to_remove {
        entity_manager.remove_component(lua)?;
    }
    let immortal = entity_manager.add_component::<LuaComponent>()?;
    immortal.add_tag("ew_immortal")?;
    immortal.set_script_damage_about_to_be_received(
        "mods/quant.ew/files/system/entity_sync_helper/immortal.lua".into(),
    )?;
    if let Some(var) = entity_manager.get_var(const { VarName::from_str("ghost_id") })
        && let Ok(ent) = EntityID::try_from(var.value_int()? as isize)
    {
        ent.kill()
    }
    if entity_manager.has_tag(const { CachedTag::from_tag("boss_dragon") }) && drops_gold {
        let lua = entity_manager.add_component::<LuaComponent>()?;
        lua.set_script_death("data/scripts/animals/boss_dragon_death.lua".into())?;
        lua.set_execute_every_n_frame(-1)?;
    }
    if let Some(life) = entity_manager
        .try_get_first_component_including_disabled::<LifetimeComponent>(ComponentTag::None)
    {
        life.set_lifetime(i32::MAX)?;
    }
    if let Some(pickup) = entity_manager
        .try_get_first_component_including_disabled::<ItemPickUpperComponent>(ComponentTag::None)
    {
        pickup.set_drop_items_on_death(false)?;
        pickup.set_only_pick_this_entity(Some(EntityID(NonZero::new(1).unwrap())))?;
    }

    if let Some(ghost) = entity_manager
        .try_get_first_component_including_disabled::<GhostComponent>(ComponentTag::None)
    {
        ghost.set_die_if_no_home(false)?;
    }

    if entity_manager.has_tag(const { CachedTag::from_tag("egg_item") })
        && let Some(explosion) = entity_manager
            .try_get_first_component_including_disabled::<ExplodeOnDamageComponent>(
                ComponentTag::None,
            )
    {
        explosion.object_set_value::<Cow<'_, str>>(
            "config_explosion",
            "load_this_entity",
            "".into(),
        )?
    }

    if let Some(var) = entity_manager.get_var(const { VarName::from_str("ew_gid_lid") }) {
        entity_manager.remove_component(var)?;
    }
    if let Some(var) = entity_manager.get_var(const { VarName::from_str("throw_time") }) {
        var.set_value_int(entity_manager.frame_num() - 4)?;
    }

    if let Some(lid) = lid {
        let var = entity_manager.add_component::<VariableStorageComponent>()?;
        var.set_name("ew_gid_lid".into())?;
        if let Some(gid) = gid {
            var.set_value_string(gid.0.to_string().into())?;
        }
        var.set_value_int(i32::from_le_bytes(lid.0.to_le_bytes()))?;
        var.set_value_bool(false)?;
    }

    if entity_manager
        .try_get_first_component_including_disabled::<PhysicsBodyComponent>(ComponentTag::None)
        .is_none()
        && entity_manager
            .try_get_first_component_including_disabled::<PhysicsBody2Component>(ComponentTag::None)
            .is_none()
    {
        ephemerial(entity.0.get().cast_unsigned(), em)
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

fn spawn_entity_by_data(
    entity_data: &EntitySpawnInfo,
    x: f32,
    y: f32,
    entity_manager: &mut EntityManager,
) -> eyre::Result<EntityID> {
    match entity_data {
        EntitySpawnInfo::Filename(filename) => {
            let ent = EntityID::load(filename, Some(x as f64), Some(y as f64))?;
            entity_manager.set_current_entity(ent)?;
            let mut to_remove = Vec::new();
            for lua in
                entity_manager.iter_all_components_of_type::<LuaComponent>(ComponentTag::None)
            {
                if ["data/scripts/props/suspended_container_physics_objects.lua"]
                    .contains(&&*lua.script_source_file()?)
                {
                    to_remove.push(lua);
                }
            }
            for lua in to_remove {
                entity_manager.remove_component(lua)?;
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
    entity: &mut EntityManager,
    f: impl FnOnce(LuaComponent) -> eyre::Result<T>,
) -> eyre::Result<T> {
    let component = if let Some(c) =
        entity.try_get_first_component(const { ComponentTag::from_str(DES_SCRIPTS_TAG) })
    {
        c
    } else {
        let component = entity.add_component::<LuaComponent>()?;
        component.add_tag(DES_SCRIPTS_TAG)?;
        component.add_tag("enabled_in_inventory")?;
        component.add_tag("enabled_in_world")?;
        component.add_tag("enabled_in_hand")?;
        component.add_tag("ew_remove_on_send")?;
        component
    };
    f(component)
}

fn with_entity_scripts_no_mgr<T>(
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
fn _safe_wandkill(entity: &mut EntityManager) -> eyre::Result<()> {
    //TODO ent mgr
    let lc = entity.add_component::<LuaComponent>()?;
    lc.set_script_source_file(
        "mods/quant.ew/files/system/entity_sync_helper/scripts/killself.lua".into(),
    )?;
    entity.set_component_enabled(lc, true)?;
    lc.add_tag("enabled_in_inventory")?;
    lc.add_tag("enabled_in_world")?;
    lc.add_tag("enabled_in_hand")?;
    lc.set_execute_on_added(false)?;
    lc.set_m_next_execution_time(entity.frame_num() + 1)?;
    Ok(())
}

fn safe_entitykill(entity: &mut EntityManager) {
    let _ = entity.remove_all_components_of_type::<AudioComponent>(ComponentTag::None);
    let is_wand =
        entity.try_get_first_component_including_disabled::<AbilityComponent>(ComponentTag::None);
    if is_wand
        .map(|b| b.use_gun_script().unwrap_or(false))
        .unwrap_or(false)
    {
        let _ = _safe_wandkill(entity);
    } else {
        if let Some(inv) = entity
            .entity()
            .children(None)
            .find(|e| e.name().unwrap_or("".into()) == "inventory_quick")
        {
            inv.children(None).for_each(|e| e.kill())
        }
        entity.entity().kill();
    }
    entity.remove_current();
}

fn give_wand(
    entity: EntityID,
    seri: &[u8],
    gid: Option<Gid>,
    delete: bool,
    r: Option<f32>,
    entity_manager: &mut EntityManager,
) -> eyre::Result<()> {
    let inv = if let Some(inv) = entity_manager
        .try_get_first_component_including_disabled::<Inventory2Component>(ComponentTag::None)
    {
        inv
    } else {
        entity_manager.add_component::<Inventory2Component>()?
    };
    let mut stop = false;
    if let Some(wand) = inv.m_actual_active_item()? {
        if let Some(Some(tgid)) = wand
            .get_var("ew_gid_lid")
            .map(|var| var.value_string().unwrap_or_default().parse::<u64>().ok())
        {
            if gid != Some(Gid(tgid)) {
                if r.is_some() {
                    entity_manager.set_component_enabled(inv, true)?;
                }
                wand.kill()
            } else {
                if r.is_some() {
                    entity_manager.set_component_enabled(inv, false)?;
                }
                stop = true
            }
        } else if wand.get_var("ew_spawned_wand").is_some() {
            if r.is_some() {
                entity_manager.set_component_enabled(inv, false)?;
            }
            stop = true
        } else {
            if r.is_some() {
                entity_manager.set_component_enabled(inv, true)?;
            }
            wand.kill()
        }
        if let Some(r) = r {
            let (x, y) = entity.get_hotspot("hand")?;
            wand.set_position(x, y, Some(r as f64))?;
        }
    }
    if !stop {
        if r.is_some() {
            entity_manager.set_component_enabled(inv, true)?;
        }
        let (x, y) = entity.position()?;
        let wand = deserialize_entity(seri, x as f32, y as f32)?;
        if delete {
            if let Some(pickup) = entity_manager
                .try_get_first_component_including_disabled::<ItemPickUpperComponent>(
                    ComponentTag::None,
                )
            {
                pickup.set_only_pick_this_entity(Some(wand))?;
            }
            let quick = if let Some(quick) = entity.children(None).find_map(|a| {
                if a.name().ok()? == "inventory_quick" {
                    a.children(None).for_each(|e| e.kill());
                    Some(a)
                } else {
                    None
                }
            }) {
                quick
            } else {
                let quick = EntityID::create(Some("inventory_quick".into()))?;
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

fn mom(entity: &mut EntityManager, counter: u8, cost: Option<i32>) -> eyre::Result<()> {
    if entity.has_tag(const { CachedTag::from_tag("boss_wizard") }) {
        for ent in entity.entity().children(None) {
            if ent.has_tag("touchmagic_immunity")
                && let Ok(var) = ent
                    .get_first_component_including_disabled::<VariableStorageComponent>(Some(
                        "wizard_orb_id".into(),
                    ))
            {
                if let Ok(n) = var.value_int() {
                    if (counter & (1 << (n as u8))) == 0 {
                        ent.kill()
                    } else if let Ok(damage) = ent.get_first_component::<DamageModelComponent>(None)
                    {
                        damage.set_wait_for_kill_flag_on_death(true)?;
                        damage.set_hp(damage.max_hp()?)?;
                    }
                }
                if let Some(cost) = cost
                    && let Ok(v) = ent.get_var_or_default("ew_frame_num")
                {
                    let _ = v.add_tag("ew_frame_num");
                    let _ = v.set_value_int(cost);
                }
            }
        }
    }
    Ok(())
}
fn sun(entity: &mut EntityManager, counter: u8) -> eyre::Result<()> {
    if entity.has_tag(const { CachedTag::from_tag("seed_d") }) {
        let essences =
            entity.get_var_or_default(const { VarName::from_str("sunbaby_essences_list") })?;
        let mut s = String::new();
        if counter & 1 == 1 {
            s += "water,";
            entity
                .entity()
                .set_components_with_tag_enabled("water".into(), true)?;
        }
        if counter & 2 == 2 {
            s += "fire,";
            entity
                .entity()
                .set_components_with_tag_enabled("fire".into(), true)?;
            entity
                .entity()
                .set_components_with_tag_enabled("fire_disable".into(), false)?;
        }
        if counter & 4 == 4 {
            s += "air,";
            entity
                .entity()
                .set_components_with_tag_enabled("air".into(), true)?;
        }
        if counter & 8 == 8 {
            s += "earth,";
            entity
                .entity()
                .set_components_with_tag_enabled("earth".into(), true)?;
            entity
                .entity()
                .set_components_with_tag_enabled("earth_disable".into(), false)?;
        }
        if counter & 16 == 16 {
            s += "poop,";
            entity
                .entity()
                .set_components_with_tag_enabled("poop".into(), true)?;
        }
        essences.set_value_string(s.into())?;
        let n = (counter & (32 + 64 + 128)) / 32;
        if counter != 0 {
            let sprite = entity.get_first_component::<SpriteComponent>(
                const { ComponentTag::from_str("sunbaby_sprite") },
            )?;
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
