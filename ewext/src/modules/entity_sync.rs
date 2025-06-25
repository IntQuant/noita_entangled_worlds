//! Distibuted Entity Sync, a.k.a. DES.
//! The idea is that we completely disregard the normal saving system for entities we sync.
//! Also, each entity gets an owner.
//! Each peer broadcasts an "Interest" zone. If it intersects any peer they receive all information about entities this peer owns.

use super::{Module, ModuleCtx, NetManager};
use crate::{ToState, my_peer_id};
use bimap::BiHashMap;
use diff_model::{DES_TAG, LocalDiffModel, RemoteDiffModel, entity_is_item};
use eyre::{Context, OptionExt};
use interest::InterestTracker;
use noita_api::raw::game_get_frame_num;
use noita_api::serialize::serialize_entity;
use noita_api::{
    CachedTag, ComponentTag, DamageModelComponent, DamageType, EntityID, EntityManager,
    ItemCostComponent, LuaComponent, PositionSeedComponent, ProjectileComponent, VarName,
    VelocityComponent,
};
use rustc_hash::{FxHashMap, FxHashSet};
use shared::des::DesToProxy::UpdatePositions;
use shared::{
    Destination, NoitaOutbound, PeerId, RemoteMessage, WorldPos,
    des::{Gid, InterestRequest, ProjectileFired, RemoteDes},
};
use std::sync::{LazyLock, Mutex};
mod diff_model;
mod interest;

static ENTITY_EXCLUDES: LazyLock<FxHashSet<String>> = LazyLock::new(|| {
    let mut hs = FxHashSet::default();
    hs.insert("data/entities/items/pickup/perk.xml".to_string());
    hs.insert("data/entities/items/pickup/spell_refresh.xml".to_string());
    hs.insert("data/entities/items/pickup/heart.xml".to_string());
    hs.insert("data/entities/items/pickup/heart_better.xml".to_string());
    hs.insert("data/entities/items/pickup/heart_evil.xml".to_string());
    hs.insert("data/entities/items/pickup/heart_fullhp.xml".to_string());
    hs.insert("data/entities/items/pickup/heart_fullhp_temple.xml".to_string());
    hs.insert("data/entities/items/pickup/perk_reroll.xml".to_string());
    hs
});

pub(crate) struct EntitySync {
    /// Last entity we stopped looking for new tracked entities at.
    look_current_entity: EntityID,

    interest_tracker: InterestTracker,
    local_diff_model: LocalDiffModel,
    remote_models: FxHashMap<PeerId, RemoteDiffModel>,

    pending_fired_projectiles: Mutex<Vec<(EntityID, ProjectileFired)>>,
    dont_kill: FxHashSet<EntityID>,
    dont_kill_by_gid: FxHashSet<Gid>,
    dont_track: FxHashSet<EntityID>,
    spawn_once: Vec<(WorldPos, shared::SpawnOnce)>,
    kill_later: Vec<(EntityID, Option<PeerId>)>,
    to_track: Vec<EntityID>,
    local_index: usize,
    remote_index: FxHashMap<PeerId, usize>,
    peer_order: Vec<PeerId>,
    log_performance: bool,
    entity_manager: EntityManager,
}
impl EntitySync {
    pub(crate) fn set_perf(&mut self, perf: bool) {
        self.log_performance = perf;
    }
    /*pub(crate) fn has_gid(&self, gid: Gid) -> bool {
        self.local_diff_model.has_gid(gid) || self.remote_models.values().any(|r| r.has_gid(gid))
    }*/
    pub(crate) fn track_entity(&mut self, ent: EntityID) {
        let _ = self
            .local_diff_model
            .track_and_upload_entity(ent, &mut self.entity_manager);
    }
    pub(crate) fn notrack_entity(&mut self, ent: EntityID) {
        self.dont_track.insert(ent);
    }
    pub(crate) fn find_by_gid(&self, gid: Gid) -> Option<EntityID> {
        self.local_diff_model
            .find_by_gid(gid)
            .or(self.remote_models.values().find_map(|r| r.find_by_gid(gid)))
    }
    pub(crate) fn find_peer_by_gid(&self, gid: Gid) -> Option<&PeerId> {
        self.remote_models.iter().find_map(|(p, g)| {
            if g.find_by_gid(gid).is_some() {
                Some(p)
            } else {
                None
            }
        })
    }
}
impl Default for EntitySync {
    fn default() -> Self {
        Self {
            look_current_entity: EntityID::try_from(1).unwrap(),

            interest_tracker: InterestTracker::new(512.0),
            local_diff_model: LocalDiffModel::default(),
            remote_models: Default::default(),

            pending_fired_projectiles: Vec::new().into(),
            dont_kill: Default::default(),
            dont_kill_by_gid: Default::default(),
            dont_track: Default::default(),
            spawn_once: Default::default(),
            kill_later: Vec::new(),
            to_track: Vec::new(),
            local_index: 0,
            remote_index: Default::default(),
            peer_order: Vec::new(),
            log_performance: false,
            entity_manager: EntityManager::default(),
        }
    }
}

fn entity_is_excluded(entity: EntityID) -> eyre::Result<bool> {
    let good = "data/entities/items/wands/wand_good/wand_good_";
    let filename = entity.filename()?;
    let tags = format!(",{},", entity.tags()?);
    Ok(tags.contains(",ew_no_enemy_sync,")
        || tags.contains(",polymorphed_player,")
        || tags.contains(",gold_nugget,")
        || tags.contains(",nightmare_starting_wand,")
        || ENTITY_EXCLUDES.contains(&filename)
        || filename.starts_with(good)
        || tags.contains(",player_unit,")
        || filename == "data/entities/items/pickup/greed_curse.xml"
        || (!tags.contains(",ew_sync_child,") && entity.root()? != Some(entity)))
}

impl EntitySync {
    fn clear_buffer(&mut self, ctx: &mut ModuleCtx, new_intersects: &[PeerId]) -> eyre::Result<()> {
        if !self.local_diff_model.init_buffer.is_empty() {
            let res = std::mem::take(&mut self.local_diff_model.init_buffer);
            let (RemoteDes::EntityInit(diff), err) = send_remotedes_ret(
                ctx,
                true,
                Destination::Peers(
                    self.interest_tracker
                        .iter_interested()
                        .filter(|p| !new_intersects.contains(p))
                        .collect(),
                ),
                RemoteDes::EntityInit(res),
            ) else {
                unreachable!()
            };
            self.local_diff_model.init_buffer = diff;
            self.local_diff_model.uninit();
            err?;
        }
        if !self.local_diff_model.update_buffer.is_empty() {
            let res = std::mem::take(&mut self.local_diff_model.update_buffer);
            let (RemoteDes::EntityUpdate(diff), err) = send_remotedes_ret(
                ctx,
                true,
                Destination::Peers(
                    self.interest_tracker
                        .iter_interested()
                        .filter(|p| !new_intersects.contains(p))
                        .collect(),
                ),
                RemoteDes::EntityUpdate(res),
            ) else {
                unreachable!()
            };
            self.local_diff_model.update_buffer = diff;
            err?;
        }
        Ok(())
    }
    pub(crate) fn spawn_once(
        &mut self,
        ctx: &mut super::ModuleCtx,
        frame_num: usize,
    ) -> eyre::Result<()> {
        let (x, y) = noita_api::raw::game_get_camera_pos()?;
        let len = self.spawn_once.len();
        if len > 0 {
            let batch_size = (len / 20).max(1);
            let start_index = (frame_num % 20) * batch_size;
            let end_index = (start_index + batch_size).min(len);
            let mut i = end_index;
            while i > start_index {
                i -= 1;
                if i < self.spawn_once.len() {
                    let (pos, data) = &self.spawn_once[i];
                    if pos.contains(x, y, 512 + 256) {
                        let (x, y) = (pos.x as f64, pos.y as f64);
                        match data {
                            shared::SpawnOnce::Enemy(file, drops_gold, offending_peer) => {
                                if let Ok(entity) = EntityID::load(file, Some(x), Some(y)) {
                                    entity.add_tag("ew_no_enemy_sync")?;
                                    diff_model::init_remote_entity(
                                        entity,
                                        None,
                                        None,
                                        *drops_gold,
                                        &mut self.entity_manager,
                                    )?;
                                    if let Some(damage) = entity
                                        .try_get_first_component::<DamageModelComponent>(None)?
                                    {
                                        for lua in entity
                                            .iter_all_components_of_type::<LuaComponent>(None)?
                                        {
                                            if ["data/entities/animals/boss_spirit/islandspirit.lua","data/entities/animals/boss_sky/boss_sky.lua"].contains(&&*lua.script_damage_received()?) {
                                                lua.set_script_damage_received("".into())?
                                            }
                                        }
                                        entity
                                            .children(Some("protection".into()))
                                            .for_each(|ent| ent.kill());
                                        damage.set_ui_report_damage(false)?;
                                        if entity.has_tag("boss_centipede") {
                                            entity.set_components_with_tag_enabled(
                                                "enabled_at_start".into(),
                                                false,
                                            )?;
                                            entity.set_components_with_tag_enabled(
                                                "disabled_at_start".into(),
                                                true,
                                            )?;
                                            self.kill_later.push((entity, *offending_peer))
                                        } else {
                                            let responsible_entity = offending_peer
                                                .and_then(|peer| ctx.player_map.get_by_left(&peer))
                                                .copied();
                                            damage.object_set_value(
                                                "damage_multipliers",
                                                "curse",
                                                1.0,
                                            )?;
                                            entity.inflict_damage(
                                                damage.max_hp()? * 100.0,
                                                DamageType::DamageCurse,
                                                "kill sync",
                                                responsible_entity,
                                            )?;
                                        }
                                    }
                                }
                            }
                            shared::SpawnOnce::Chest(file, rx, ry) => {
                                if let Ok(ent) = EntityID::load(file, Some(x), Some(y)) {
                                    ent.add_tag("ew_no_enemy_sync")?;
                                    if let Some(file) = ent
                                        .iter_all_components_of_type_including_disabled::<LuaComponent>(
                                            None,
                                        )?
                                        .find(|l| {
                                            !l.script_physics_body_modified()
                                              .unwrap_or("".into())
                                              .is_empty()
                                        })
                                        .map(|l| l.script_physics_body_modified().unwrap_or("".into()))
                                    {
                                        if let Some(seed) = ent.try_get_first_component_including_disabled::<PositionSeedComponent>(None)? {
                                            seed.set_pos_x(*rx)?;
                                            seed.set_pos_y(*ry)?;
                                        }
                                        ent.add_lua_init_component::<LuaComponent>(&file)?;
                                    }
                                }
                            }
                            shared::SpawnOnce::BrokenWand => {
                                let ent = EntityID::create(None)?;
                                ent.set_position(x as f32, y as f32, None)?;
                                ent.add_tag("broken_wand")?;
                                ent.add_lua_init_component::<LuaComponent>(
                                    "data/scripts/buildings/forge_item_convert.lua",
                                )?;
                            }
                        }
                        self.spawn_once.remove(i);
                        i += 1;
                    }
                }
            }
        }
        Ok(())
    }
    pub fn iter_peers<'a>(
        &'a self,
        player_map: &'a BiHashMap<PeerId, EntityID>,
    ) -> impl Iterator<Item = (bool, PeerId)> + 'a {
        player_map.left_values().filter_map(move |p| {
            if *p != my_peer_id() {
                Some((self.interest_tracker.contains(*p), *p))
            } else {
                None
            }
        })
    }
    fn should_be_tracked(&mut self, entity: EntityID) -> eyre::Result<bool> {
        let tags = format!(",{},", entity.tags()?);
        let should_be_tracked = [
            ",enemy,",
            ",ew_synced,",
            ",plague_rat,",
            ",seed_f,",
            ",seed_e,",
            ",seed_d,",
            ",seed_c,",
            ",perk_fungus_tiny,",
            ",helpless_animal,",
            ",nest,",
        ]
        .iter()
        .any(|tag| tags.contains(tag))
            || entity_is_item(entity)?;

        Ok(should_be_tracked && !entity_is_excluded(entity)?)
    }

    pub(crate) fn handle_proxytodes(
        &mut self,
        proxy_to_des: shared::des::ProxyToDes,
    ) -> eyre::Result<()> {
        match proxy_to_des {
            shared::des::ProxyToDes::GotAuthority(full_entity_data) => {
                self.local_diff_model.got_authority(full_entity_data);
            }
            shared::des::ProxyToDes::GotAuthoritys(full_entity_data) => {
                self.local_diff_model.got_authoritys(full_entity_data);
            }
            shared::des::ProxyToDes::RemoveEntities(peer) => {
                if let Some(remote) = self.remote_models.remove(&peer) {
                    remote.remove_entities(&mut self.entity_manager)?
                }
                self.interest_tracker.remove_peer(peer);
                let _ = crate::ExtState::with_global(|state| {
                    state.fps_by_player.remove(&peer);
                    state.player_entity_map.remove_by_left(&peer);
                });
            }
            shared::des::ProxyToDes::DeleteEntity(entity) => {
                EntityID(entity).kill();
            }
        }
        Ok(())
    }

    pub(crate) fn handle_remotedes(
        &mut self,
        source: PeerId,
        remote_des: RemoteDes,
        net: &mut NetManager,
        player_entity_map: &BiHashMap<PeerId, EntityID>,
        dont_spawn: &FxHashSet<Gid>,
    ) -> eyre::Result<ToState> {
        match remote_des {
            RemoteDes::ChestOpen(gid, x, y, file, rx, ry) => {
                if !dont_spawn.contains(&gid) {
                    if let Some(ent) = self.find_by_gid(gid) {
                        ent.kill()
                    }
                    if let Ok(ent) = EntityID::load(file, Some(x as f64), Some(y as f64)) {
                        ent.add_tag("ew_no_enemy_sync")?;
                        if let Some(file) = ent
                            .iter_all_components_of_type_including_disabled::<LuaComponent>(None)?
                            .find(|l| {
                                !l.script_physics_body_modified()
                                    .unwrap_or("".into())
                                    .is_empty()
                            })
                            .map(|l| l.script_physics_body_modified().unwrap_or("".into()))
                        {
                            if let Some(seed) = ent.try_get_first_component_including_disabled::<PositionSeedComponent>(None)? {
                                            seed.set_pos_x(rx)?;
                                            seed.set_pos_y(ry)?;
                                        }
                            ent.add_lua_init_component::<LuaComponent>(&file)?;
                        }
                    }
                }
                return Ok(ToState::Gid(gid));
            }
            RemoteDes::ChestOpenRequest(gid, x, y, file, rx, ry) => {
                net.send(&NoitaOutbound::RemoteMessage {
                    reliable: true,
                    destination: Destination::Peer(my_peer_id()),
                    message: RemoteMessage::RemoteDes(RemoteDes::ChestOpen(
                        gid,
                        x,
                        y,
                        file.clone(),
                        rx,
                        ry,
                    )),
                })?;
                for (has_interest, peer) in self.iter_peers(player_entity_map) {
                    if has_interest {
                        net.send(&NoitaOutbound::RemoteMessage {
                            reliable: true,
                            destination: Destination::Peer(peer),
                            message: RemoteMessage::RemoteDes(RemoteDes::ChestOpen(
                                gid,
                                x,
                                y,
                                file.clone(),
                                rx,
                                ry,
                            )),
                        })?;
                    } else {
                        net.send(&NoitaOutbound::RemoteMessage {
                            reliable: true,
                            destination: Destination::Peer(peer),
                            message: RemoteMessage::RemoteDes(RemoteDes::SpawnOnce(
                                WorldPos::from((x, y)),
                                shared::SpawnOnce::Chest(file.clone(), rx, ry),
                            )),
                        })?;
                    }
                }
            }
            RemoteDes::SpawnOnce(pos, data) => self.spawn_once.push((pos, data)),
            /*RemoteDes::AllEntities(lids) => self
            .remote_models
            .entry(source)
            .or_insert(RemoteDiffModel::new(source))
            .check_entities(lids),*/
            RemoteDes::CameraPos(pos) => {
                return Ok(ToState::Pos(pos));
            }
            RemoteDes::DeadEntities(vec) => self.spawn_once.extend(vec),
            RemoteDes::InterestRequest(interest_request) => self
                .interest_tracker
                .handle_interest_request(source, interest_request),
            RemoteDes::EntityUpdate(vec) => {
                self.remote_models
                    .entry(source)
                    .or_insert(RemoteDiffModel::new(source))
                    .apply_diff(vec, &mut self.entity_manager)?;
            }
            RemoteDes::EntityInit(vec) => {
                self.dont_kill.extend(
                    self.remote_models
                        .entry(source)
                        .or_insert(RemoteDiffModel::new(source))
                        .apply_init(vec, &mut self.entity_manager),
                );
            }
            RemoteDes::ExitedInterest => {
                if let Some(remote) = self.remote_models.remove(&source) {
                    remote.remove_entities(&mut self.entity_manager)?
                }
            }
            RemoteDes::Reset => self.interest_tracker.reset_interest_for(source),
            RemoteDes::Projectiles(vec) => {
                self.remote_models
                    .entry(source)
                    .or_insert(RemoteDiffModel::new(source))
                    .spawn_projectiles(&vec);
            }
            RemoteDes::RequestGrab(lid) => {
                self.local_diff_model
                    .entity_grabbed(source, lid, net, &mut self.entity_manager)?;
            }
        }
        Ok(ToState::None)
    }

    pub(crate) fn cross_item_thrown(&mut self, entity: Option<EntityID>) -> eyre::Result<()> {
        let entity = entity.ok_or_eyre("Passed entity 0 into cross call")?;
        // It might be already tracked in case of tablet telekinesis, no need to track it again.
        if !self.local_diff_model.is_entity_tracked(entity) {
            self.local_diff_model
                .track_and_upload_entity(entity, &mut self.entity_manager)?;
        }
        Ok(())
    }

    pub(crate) fn cross_death_notify(
        &mut self,
        entity_killed: EntityID,
        wait_on_kill: bool,
        pos: WorldPos,
        file: String,
        entity_responsible: Option<EntityID>,
    ) -> eyre::Result<()> {
        self.local_diff_model.death_notify(
            entity_killed,
            wait_on_kill,
            pos,
            file,
            entity_responsible,
        );
        Ok(())
    }

    pub(crate) fn sync_projectile(
        &mut self,
        entity: EntityID,
        gid: Gid,
        peer: PeerId,
    ) -> eyre::Result<()> {
        if peer == my_peer_id() {
            self.dont_kill.insert(entity);
            let lid = self
                .local_diff_model
                .track_entity(entity, gid, &mut self.entity_manager)?;
            self.local_diff_model.dont_save(lid);
        } else if let Some(remote) = self.remote_models.get_mut(&peer) {
            remote.wait_for_gid(entity, gid);
        }
        Ok(())
    }
}

impl Module for EntitySync {
    fn on_world_init(&mut self, ctx: &mut super::ModuleCtx) -> eyre::Result<()> {
        send_remotedes(ctx.net, true, Destination::Broadcast, RemoteDes::Reset)?;
        Ok(())
    }

    /// Looks for newly spawned entities that might need to be tracked.
    fn on_new_entity(&mut self, ent: isize, kill: bool) -> eyre::Result<()> {
        let entity = EntityID::try_from(ent)?;
        if !kill && !entity.is_alive() {
            return Ok(());
        }
        if entity.0 <= self.look_current_entity.0 {
            return Ok(());
        }
        if self.dont_track.remove(&entity) {
            return Ok(());
        }
        if let Ok(Some(gid)) = entity.handle_poly() {
            self.dont_kill_by_gid.insert(gid);
            self.local_diff_model.got_polied(gid);
        }
        if self.should_be_tracked(entity)? {
            self.entity_manager.set_current_entity(entity)?;
            if self
                .entity_manager
                .has_tag(const { CachedTag::from_tag(DES_TAG) })
                && !self.dont_kill.remove(&entity)
                && self
                    .entity_manager
                    .get_var(const { VarName::from_str("ew_gid_lid") })
                    .map(|var| {
                        if let Ok(n) = var.value_string().unwrap_or("NA".into()).parse::<u64>() {
                            !self.dont_kill_by_gid.remove(&Gid(n))
                        } else {
                            true
                        }
                    })
                    .unwrap_or(true)
            {
                if kill {
                    entity.kill();
                }
            } else {
                if self
                    .entity_manager
                    .has_tag(const { CachedTag::from_tag("card_action") })
                {
                    if let Some(cost) = self
                        .entity_manager
                        .try_get_first_component::<ItemCostComponent>(ComponentTag::None)?
                    {
                        if cost.stealable()? {
                            cost.set_stealable(false)?;
                            self.entity_manager.get_var_or_default(
                                const { VarName::from_str("ew_was_stealable") },
                            )?;
                        }
                    }
                    if let Some(vel) = self
                        .entity_manager
                        .try_get_first_component::<VelocityComponent>(ComponentTag::None)?
                    {
                        vel.set_gravity_y(0.0)?;
                        vel.set_air_friction(10.0)?;
                    }
                }
                self.to_track.push(entity);
            }
        } else if kill
            && !self.dont_kill.remove(&entity)
            && entity.has_tag(DES_TAG)
            && entity
                .get_var("ew_gid_lid")
                .map(|var| {
                    if let Ok(n) = var.value_string().unwrap_or("NA".into()).parse::<u64>() {
                        !self.dont_kill_by_gid.remove(&Gid(n))
                    } else {
                        true
                    }
                })
                .unwrap_or(true)
        {
            entity.kill();
        }
        Ok(())
    }

    fn on_world_update(&mut self, ctx: &mut super::ModuleCtx) -> eyre::Result<()> {
        let start = std::time::Instant::now();
        let (x, y) = noita_api::raw::game_get_camera_pos()?;
        let pos = WorldPos::from_f64(x, y);
        self.interest_tracker.set_center(x, y);
        let frame_num = game_get_frame_num()? as usize;
        if frame_num < 10 {
            return Ok(());
        }
        if frame_num % 5 == 0 {
            send_remotedes(
                ctx.net,
                false,
                Destination::Broadcast,
                RemoteDes::InterestRequest(InterestRequest { pos }),
            )?;
        }
        if frame_num % 5 == 1 {
            let iter = self.iter_peers(ctx.player_map);
            for (_, peer) in iter {
                send_remotedes(
                    ctx.net,
                    false,
                    Destination::Peer(peer),
                    RemoteDes::CameraPos(pos),
                )?;
            }
        }

        for lost in self.interest_tracker.drain_lost_interest() {
            send_remotedes(
                ctx.net,
                true,
                Destination::Peer(lost),
                RemoteDes::ExitedInterest,
            )?;
        }

        self.look_current_entity = EntityID::max_in_use()?;
        self.local_diff_model
            .enable_later(&mut self.entity_manager)?;
        self.local_diff_model.phys_later(&mut self.entity_manager)?;
        let mut times = vec![0; 4];
        if self.log_performance {
            times[0] = start.elapsed().as_micros();
        }
        self.local_diff_model
            .update_pending_authority(start, &mut self.entity_manager)?;
        if self.log_performance {
            times[1] = start.elapsed().as_micros() - times[0];
        }
        for ent in self.look_current_entity.0.get() + 1..=EntityID::max_in_use()?.0.get() {
            self.on_new_entity(ent, false)?;
        }
        while let Some(entity) = self.to_track.pop() {
            if entity.is_alive() {
                self.local_diff_model
                    .track_and_upload_entity(entity, &mut self.entity_manager)?;
                if start.elapsed().as_micros() > 2000 {
                    break;
                }
            } else {
                self.entity_manager.remove_ent(&entity);
            }
        }
        if self.log_performance {
            times[2] = start.elapsed().as_micros() - (times[0] + times[1]);
        }
        {
            let new_intersects = self.interest_tracker.got_any_new_interested();
            if !new_intersects.is_empty() {
                self.local_diff_model.make_init();
                let res = std::mem::take(&mut self.local_diff_model.init_buffer);
                let (RemoteDes::EntityInit(diff), err) = send_remotedes_ret(
                    ctx,
                    true,
                    Destination::Peers(new_intersects.clone()),
                    RemoteDes::EntityInit(res),
                ) else {
                    unreachable!()
                };
                self.local_diff_model.init_buffer = diff;
                self.local_diff_model.uninit();
                err?;
            }
            let dead;
            (dead, self.local_index) = match self
                .local_diff_model
                .update_tracked_entities(ctx, self.local_index, start, &mut self.entity_manager)
                .wrap_err("Failed to update locally tracked entities")
            {
                Ok(ret) => ret,
                Err(s) => {
                    self.clear_buffer(ctx, &new_intersects)?;
                    return Err(s);
                }
            };
            if self.log_performance {
                times[3] = start.elapsed().as_micros() - (times[0] + times[1] + times[2]);
            }
            {
                let proj = &mut self.pending_fired_projectiles.lock().unwrap();
                if !proj.is_empty() {
                    let data = proj
                        .drain(..)
                        .map(|(ent, mut proj)| {
                            if ent.is_alive() {
                                if let Ok(Some(vel)) = ent
                                    .try_get_first_component_including_disabled::<VelocityComponent>(
                                        None,
                                    )
                                {
                                    proj.vel = vel.m_velocity().ok()
                                }
                            }
                            proj
                        })
                        .collect();
                    send_remotedes(
                        ctx.net,
                        true,
                        Destination::Peers(self.interest_tracker.iter_interested().collect()),
                        RemoteDes::Projectiles(data),
                    )?;
                }
            }
            self.clear_buffer(ctx, &new_intersects)?;
            if !dead.is_empty() {
                send_remotedes(
                    ctx.net,
                    true,
                    Destination::Peers(
                        ctx.player_map
                            .left_values()
                            .filter(|p| {
                                !self.interest_tracker.contains(**p)
                                    && **p != my_peer_id()
                                    && !dead.is_empty()
                            })
                            .cloned()
                            .collect(),
                    ),
                    RemoteDes::DeadEntities(dead),
                )?;
            }
        }
        if frame_num > 120 {
            let mut to_remove = Vec::new();
            for peer in self.remote_models.keys() {
                if !self.peer_order.contains(peer) {
                    self.peer_order.insert(0, *peer);
                }
            }
            for (i, owner) in self.peer_order.iter().enumerate() {
                match self.remote_models.get_mut(owner) {
                    Some(remote_model) => {
                        let vi = self.remote_index.entry(*owner).or_insert(0);
                        let v = remote_model
                            .apply_entities(ctx, *vi, start, &mut self.entity_manager)
                            .wrap_err("Failed to apply entity infos")?;
                        self.remote_index.insert(*owner, v);
                        if self.log_performance {
                            times.push(start.elapsed().as_micros() - times.iter().sum::<u128>());
                        }
                        for lid in remote_model.drain_grab_request() {
                            send_remotedes(
                                ctx.net,
                                true,
                                Destination::Peer(*owner),
                                RemoteDes::RequestGrab(lid),
                            )?;
                        }
                    }
                    None => {
                        to_remove.insert(0, i);
                    }
                }
            }
            for i in to_remove {
                self.peer_order.remove(i);
            }
            if self.peer_order.len() > 1 {
                let p = self.peer_order.remove(0);
                self.peer_order.push(p)
            }
        }
        // These entities shouldn't be tracked by us, as they were spawned by remote.
        self.look_current_entity = EntityID::max_in_use()?;
        for (_, remote_model) in self.remote_models.iter_mut() {
            remote_model.kill_entities(ctx, &mut self.entity_manager)?;
        }
        for (entity, offending_peer) in self.kill_later.drain(..) {
            if entity.is_alive() {
                let responsible_entity = offending_peer
                    .and_then(|peer| ctx.player_map.get_by_left(&peer))
                    .copied();
                if let Some(damage) =
                    entity.try_get_first_component::<DamageModelComponent>(None)?
                {
                    damage.object_set_value("damage_multipliers", "curse", 1.0)?;
                    entity.inflict_damage(
                        damage.max_hp()? * 100.0,
                        DamageType::DamageCurse,
                        "kill sync",
                        responsible_entity,
                    )?;
                }
            }
        }
        if let Err(s) = self.spawn_once(ctx, frame_num) {
            crate::print_error(s)?;
        }

        if frame_num % 7 == 3 {
            ctx.net.send(&NoitaOutbound::DesToProxy(
                shared::des::DesToProxy::RequestAuthority {
                    pos,
                    //radius: REQUEST_AUTHORITY_RADIUS,
                },
            ))?;
        }
        let pos_data = self.local_diff_model.get_pos_data(frame_num);
        if !pos_data.is_empty() {
            ctx.net
                .send(&NoitaOutbound::DesToProxy(UpdatePositions(pos_data)))?;
        }
        if self.log_performance {
            times.push(start.elapsed().as_micros() - times.iter().sum::<u128>());
            crate::print(&format!("{:?}", times))?;
        }
        Ok(())
    }

    fn on_projectile_fired(
        &mut self,
        _ctx: &mut super::ModuleCtx,
        shooter: Option<EntityID>,
        projectile: Option<EntityID>,
        _initial_rng: i32,
        position: (f32, f32),
        target: (f32, f32),
        _multicast_index: i32,
    ) -> eyre::Result<()> {
        // This also checks that we do own the entity and that entity_sync is supposed to work on it.
        let Some(shooter_lid) = shooter.and_then(|e| self.local_diff_model.lid_by_entity(e)) else {
            return Ok(());
        };
        let Some(projectile) = projectile else {
            // How is that supposed to happen, anyway?
            return Ok(());
        };
        let Some(proj_component) =
            projectile.try_get_first_component::<ProjectileComponent>(None)?
        else {
            return Ok(());
        };

        let filename = projectile.filename()?;
        if proj_component.m_entity_that_shot()?.is_some()
            || [
                "data/entities/animals/boss_wizard/summon.xml",
                "data/entities/projectiles/bat.xml",
                "data/entities/items/pickup/potion_aggressive.xml",
                "data/entities/projectiles/pebble.xml",
                "data/entities/projectiles/cocktail.xml",
                "data/entities/projectiles/fungus.xml",
            ]
            .contains(&&*filename)
        {
            return Ok(());
        }

        let serialized = serialize_entity(projectile)?;

        self.pending_fired_projectiles.lock().unwrap().push((
            projectile,
            ProjectileFired {
                shooter_lid,
                position,
                target,
                serialized,
                vel: None,
            },
        ));

        //TODO initial_rng might need to be handled with np.SetProjectileSpreadRNG?

        Ok(())
    }
}

fn send_remotedes(
    ctx: &mut NetManager,
    reliable: bool,
    destination: Destination<PeerId>,
    remote_des: RemoteDes,
) -> Result<(), eyre::Error> {
    let message = NoitaOutbound::RemoteMessage {
        reliable,
        destination,
        message: RemoteMessage::RemoteDes(remote_des),
    };
    ctx.send(&message)
}

fn send_remotedes_ret(
    ctx: &mut ModuleCtx<'_>,
    reliable: bool,
    destination: Destination<PeerId>,
    remote_des: RemoteDes,
) -> (RemoteDes, Result<(), eyre::Error>) {
    let message = NoitaOutbound::RemoteMessage {
        reliable,
        destination,
        message: RemoteMessage::RemoteDes(remote_des),
    };
    let err = ctx.net.send(&message);
    let NoitaOutbound::RemoteMessage {
        message: RemoteMessage::RemoteDes(des),
        ..
    } = message
    else {
        unreachable!()
    };
    (des, err)
}
