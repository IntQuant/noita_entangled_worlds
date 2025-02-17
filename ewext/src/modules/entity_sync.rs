//! Distibuted Entity Sync, a.k.a. DES.
//! The idea is that we completely disregard the normal saving system for entities we sync.
//! Also, each entity gets an owner.
//! Each peer broadcasts an "Interest" zone. If it intersects any peer they receive all information about entities this peer owns.

use super::{Module, NetManager};
use crate::my_peer_id;
use bimap::BiHashMap;
use diff_model::{entity_is_item, LocalDiffModel, RemoteDiffModel, DES_TAG};
use eyre::{Context, OptionExt};
use interest::InterestTracker;
use noita_api::serialize::serialize_entity;
use noita_api::{
    DamageModelComponent, EntityID, LuaComponent, PositionSeedComponent, ProjectileComponent,
    VariableStorageComponent,
};
use rustc_hash::{FxHashMap, FxHashSet};
use shared::des::DesToProxy::UpdatePositions;
use shared::{
    des::{
        Gid, InterestRequest, ProjectileFired, RemoteDes, INTEREST_REQUEST_RADIUS,
        REQUEST_AUTHORITY_RADIUS,
    },
    Destination, NoitaOutbound, PeerId, RemoteMessage, WorldPos,
};
use std::sync::{Arc, LazyLock};
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

    pending_fired_projectiles: Arc<Vec<ProjectileFired>>,
    dont_kill: FxHashSet<EntityID>,
    dont_kill_by_gid: FxHashSet<Gid>,
    dont_track: FxHashSet<EntityID>,
    spawn_once: Vec<(WorldPos, shared::SpawnOnce)>,
    real_sync_rate: usize,
    delta_sync_rate: usize,
    kill_later: Vec<(EntityID, Option<PeerId>)>,
}
impl EntitySync {
    /*pub(crate) fn has_gid(&self, gid: Gid) -> bool {
        self.local_diff_model.has_gid(gid) || self.remote_models.values().any(|r| r.has_gid(gid))
    }*/
    pub(crate) fn track_entity(&mut self, net: &mut NetManager, ent: EntityID) {
        let _ = self
            .local_diff_model
            .track_and_upload_entity(net, ent, Gid(rand::random()));
    }
    pub(crate) fn notrack_entity(&mut self, ent: EntityID) {
        self.dont_track.insert(ent);
    }
    pub(crate) fn find_by_gid(&self, gid: Gid) -> Option<EntityID> {
        self.local_diff_model
            .find_by_gid(gid)
            .or(self.remote_models.values().find_map(|r| r.find_by_gid(gid)))
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
            real_sync_rate: 2,
            delta_sync_rate: 2,
            kill_later: Vec::new(),
        }
    }
}

fn entity_is_excluded(entity: EntityID) -> eyre::Result<bool> {
    let good = "data/entities/items/wands/wand_good/wand_good_";
    let filename = entity.filename()?;
    Ok(entity.has_tag("ew_no_enemy_sync")
        || entity.has_tag("polymorphed_player")
        || entity.has_tag("gold_nugget")
        || entity.has_tag("nightmare_starting_wand")
        || ENTITY_EXCLUDES.contains(&filename)
        || filename.starts_with(good)
        || entity.has_tag("player_unit")
        || entity.root()? != Some(entity))
}

impl EntitySync {
    pub fn iter_peers(&self, player_map: BiHashMap<PeerId, EntityID>) -> Vec<(bool, PeerId)> {
        player_map
            .left_values()
            .filter_map(|p| {
                if *p != my_peer_id() {
                    Some((self.interest_tracker.contains(*p), *p))
                } else {
                    None
                }
            })
            .collect::<Vec<(bool, PeerId)>>()
    }
    fn should_be_tracked(&mut self, entity: EntityID) -> eyre::Result<bool> {
        let should_be_tracked = [
            "enemy",
            "ew_synced",
            "plague_rat",
            "seed_f",
            "seed_e",
            "seed_d",
            "seed_c",
            "perk_fungus_tiny",
            "helpless_animal",
            "nest",
        ]
        .iter()
        .any(|tag| entity.has_tag(tag))
            || entity_is_item(entity)?;

        Ok(should_be_tracked && !entity_is_excluded(entity)?)
    }

    pub(crate) fn handle_proxytodes(&mut self, proxy_to_des: shared::des::ProxyToDes) {
        match proxy_to_des {
            shared::des::ProxyToDes::GotAuthority(full_entity_data) => {
                self.local_diff_model.got_authority(full_entity_data);
            }
            shared::des::ProxyToDes::RemoveEntities(peer) => {
                if let Some(remote) = self.remote_models.remove(&peer) {
                    remote.remove_entities()
                }
            }
            shared::des::ProxyToDes::DeleteEntity(entity) => {
                EntityID(entity).kill();
            }
        }
    }

    pub(crate) fn handle_remotedes(
        &mut self,
        source: PeerId,
        remote_des: RemoteDes,
        net: &mut NetManager,
    ) -> eyre::Result<Option<Gid>> {
        match remote_des {
            RemoteDes::ChestOpen(gid) => {
                if let Some(ent) = self.find_by_gid(gid) {
                    if let Some(file) = ent
                        .iter_all_components_of_type_including_disabled::<LuaComponent>(None)?
                        .find(|l| !l.script_item_picked_up().unwrap_or("".into()).is_empty())
                        .map(|l| l.script_item_picked_up().unwrap_or("".into()))
                    {
                        ent.add_lua_init_component::<LuaComponent>(&file)?;
                    }
                }
                return Ok(Some(gid));
            }
            RemoteDes::SpawnOnce(pos, data) => self.spawn_once.push((pos, data)),
            RemoteDes::DeadEntities(vec) => self.spawn_once.extend(vec),
            RemoteDes::InterestRequest(interest_request) => self
                .interest_tracker
                .handle_interest_request(source, interest_request),
            RemoteDes::EntityUpdate(vec) => {
                self.dont_kill.extend(
                    self.remote_models
                        .entry(source)
                        .or_insert(RemoteDiffModel::new(source))
                        .apply_diff(&vec),
                );
            }
            RemoteDes::ExitedInterest => {
                self.remote_models.remove(&source);
            }
            RemoteDes::Reset => self.interest_tracker.reset_interest_for(source),
            RemoteDes::Projectiles(vec) => {
                self.remote_models
                    .entry(source)
                    .or_insert(RemoteDiffModel::new(source))
                    .spawn_projectiles(&vec);
            }
            RemoteDes::RequestGrab(lid) => {
                self.local_diff_model.entity_grabbed(source, lid, net);
            }
        }
        Ok(None)
    }

    pub(crate) fn cross_item_thrown(
        &mut self,
        net: &mut NetManager,
        entity: Option<EntityID>,
    ) -> eyre::Result<()> {
        let entity = entity.ok_or_eyre("Passed entity 0 into cross call")?;
        // It might be already tracked in case of tablet telekinesis, no need to track it again.
        if !self.local_diff_model.is_entity_tracked(entity) {
            self.local_diff_model
                .track_and_upload_entity(net, entity, Gid(rand::random()))?;
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
            self.local_diff_model.track_entity(entity, gid)?;
        } else if let Some(remote) = self.remote_models.get_mut(&peer) {
            remote.wait_for_gid(entity, gid);
        }
        Ok(())
    }
}

impl Module for EntitySync {
    fn on_world_init(&mut self, ctx: &mut super::ModuleCtx) -> eyre::Result<()> {
        send_remotedes(ctx, true, Destination::Broadcast, RemoteDes::Reset)?;
        Ok(())
    }

    /// Looks for newly spawned entities that might need to be tracked.
    fn on_new_entity(&mut self, entity: EntityID, ctx: &mut super::ModuleCtx) -> eyre::Result<()> {
        if entity.0 <= self.look_current_entity.0 {
            return Ok(());
        }
        if !entity.is_alive() || self.dont_track.remove(&entity) {
            return Ok(());
        }
        if let Ok(Some(gid)) = entity.handle_poly() {
            self.dont_kill_by_gid.insert(gid);
        }
        if entity.has_tag(DES_TAG)
            && !self.dont_kill.remove(&entity)
            && entity
                .iter_all_components_of_type_including_disabled::<VariableStorageComponent>(None)?
                .find(|var| var.name().unwrap_or("".into()) == "ew_gid_lid")
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
            return Ok(());
        }
        if self.should_be_tracked(entity)? {
            let gid = Gid(rand::random());
            self.local_diff_model
                .track_and_upload_entity(ctx.net, entity, gid)?;
        }
        Ok(())
    }

    fn on_world_update(&mut self, ctx: &mut super::ModuleCtx) -> eyre::Result<()> {
        let (x, y) = noita_api::raw::game_get_camera_pos()?;
        self.interest_tracker.set_center(x, y);
        let frame_num = noita_api::raw::game_get_frame_num()? as usize;
        if frame_num % 20 == 0 {
            send_remotedes(
                ctx,
                false,
                Destination::Broadcast,
                RemoteDes::InterestRequest(InterestRequest {
                    pos: WorldPos::from_f64(x, y),
                    radius: INTEREST_REQUEST_RADIUS,
                }),
            )?;
        }

        for lost in self.interest_tracker.drain_lost_interest() {
            send_remotedes(
                ctx,
                true,
                Destination::Peer(lost),
                RemoteDes::ExitedInterest,
            )?;
        }

        for (entity, offending_peer) in self.kill_later.drain(..) {
            if entity.is_alive() {
                let responsible_entity = offending_peer
                    .and_then(|peer| ctx.player_map.get_by_left(&peer))
                    .copied();
                noita_api::raw::entity_inflict_damage(
                    entity.raw() as i32,
                    32768.0,
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
        }
        let len = self.spawn_once.len();
        if len > 0 {
            let batch_size = (len / 60).max(1);
            let start_index = (frame_num % 60) * batch_size;
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
                                if let Ok(Some(entity)) =
                                    noita_api::raw::entity_load(file.into(), Some(x), Some(y))
                                {
                                    diff_model::init_remote_entity(
                                        entity,
                                        None,
                                        None,
                                        *drops_gold,
                                    )?;
                                    if let Some(damage) = entity
                                        .try_get_first_component::<DamageModelComponent>(None)?
                                    {
                                        for lua in entity
                                            .iter_all_components_of_type::<LuaComponent>(None)?
                                        {
                                            if !lua.script_damage_received()?.is_empty() {
                                                entity.remove_component(*lua)?;
                                            }
                                        }
                                        if entity.has_tag("boss_centipede") {
                                            entity.set_components_with_tag_enabled(
                                                "enabled_at_start".into(),
                                                false,
                                            )?;
                                            entity.set_components_with_tag_enabled(
                                                "disabled_at_start".into(),
                                                true,
                                            )?;
                                            entity
                                                .children(Some("protection".into()))
                                                .iter()
                                                .for_each(|ent| ent.kill());
                                            damage.set_ui_report_damage(false)?;
                                            self.kill_later.push((entity, *offending_peer))
                                        } else {
                                            damage.set_ui_report_damage(false)?;
                                            let responsible_entity = offending_peer
                                                .and_then(|peer| ctx.player_map.get_by_left(&peer))
                                                .copied();
                                            noita_api::raw::entity_inflict_damage(
                                                entity.raw() as i32,
                                                32768.0,
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
                                    }
                                }
                            }
                            shared::SpawnOnce::Chest(file, rx, ry) => {
                                if let Ok(Some(ent)) =
                                    noita_api::raw::entity_load(file.into(), Some(x), Some(y))
                                {
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
                        }
                        self.spawn_once.remove(i);
                        i += 1;
                    }
                }
            }
        }

        self.local_diff_model.update_pending_authority()?;
        let tmr = std::time::Instant::now();
        {
            let total_parts = self.real_sync_rate.max(1);
            self.local_diff_model
                .update_tracked_entities(
                    ctx,
                    frame_num.saturating_sub(self.delta_sync_rate) % total_parts,
                    total_parts,
                )
                .wrap_err("Failed to update locally tracked entities")?;
            let new_intersects = self.interest_tracker.got_any_new_interested();
            if !new_intersects.is_empty() {
                let init = self.local_diff_model.make_init();
                for peer in &new_intersects {
                    send_remotedes(
                        ctx,
                        true,
                        Destination::Peer(*peer),
                        RemoteDes::EntityUpdate(init.clone()),
                    )?;
                }
            }
            let (diff, dead) = self.local_diff_model.make_diff(ctx);
            // FIXME (perf): allow a Destination that can send to several peers at once, to prevent cloning and stuff.
            for peer in self.interest_tracker.iter_interested() {
                if !self.pending_fired_projectiles.is_empty() {
                    send_remotedes(
                        ctx,
                        true,
                        Destination::Peer(peer),
                        RemoteDes::Projectiles(self.pending_fired_projectiles.clone()),
                    )?;
                }
                if new_intersects.contains(&peer) {
                    continue;
                }
                if !diff.is_empty() {
                    send_remotedes(
                        ctx,
                        true,
                        Destination::Peer(peer),
                        RemoteDes::EntityUpdate(diff.clone()),
                    )?;
                }
            }
            for peer in ctx.player_map.clone().left_values() {
                if !self.interest_tracker.contains(*peer) && *peer != my_peer_id() {
                    send_remotedes(
                        ctx,
                        true,
                        Destination::Peer(*peer),
                        RemoteDes::DeadEntities(dead.clone()),
                    )?;
                }
            }
            Arc::make_mut(&mut self.pending_fired_projectiles).clear();
        }
        for (owner, remote_model) in self.remote_models.iter_mut() {
            let total_parts = self.real_sync_rate.max(1);
            remote_model
                .apply_entities(
                    ctx,
                    frame_num.saturating_sub(self.delta_sync_rate) % total_parts,
                    total_parts,
                )
                .wrap_err("Failed to apply entity infos")?;
            /*for entity in remote_model.drain_backtrack() {
                self.local_diff_model.track_and_upload_entity(
                    ctx.net,
                    entity,
                    Gid(rand::random()),
                )?;
            }*/
            for lid in remote_model.drain_grab_request() {
                send_remotedes(
                    ctx,
                    true,
                    Destination::Peer(*owner),
                    RemoteDes::RequestGrab(lid),
                )?;
            }
            remote_model.kill_entities(ctx)?;
        }
        // These entities shouldn't be tracked by us, as they were spawned by remote.
        self.look_current_entity = EntityID::max_in_use()?;

        if frame_num.saturating_sub(self.delta_sync_rate) % self.real_sync_rate
            == self.real_sync_rate - 1
        {
            let ms = tmr.elapsed().as_micros();
            if ms > 3000 {
                self.real_sync_rate = self.real_sync_rate.saturating_add(1)
            } else if ms < 2000 {
                self.real_sync_rate = self.real_sync_rate.saturating_sub(1)
            };
            self.real_sync_rate = self
                .real_sync_rate
                .clamp(ctx.sync_rate, 10 * (ctx.sync_rate + 1));
            self.delta_sync_rate = (frame_num + 1) % self.real_sync_rate;
        }

        if frame_num % 60 == 47 {
            let (x, y) = noita_api::raw::game_get_camera_pos()?;
            ctx.net.send(&NoitaOutbound::DesToProxy(
                shared::des::DesToProxy::RequestAuthority {
                    pos: WorldPos::from_f64(x, y),
                    radius: REQUEST_AUTHORITY_RADIUS,
                },
            ))?;
            let pos_data = self.local_diff_model.get_pos_data();
            ctx.net
                .send(&NoitaOutbound::DesToProxy(UpdatePositions(pos_data)))?;
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
            ]
            .contains(&&*filename)
        {
            return Ok(());
        }

        let serialized = serialize_entity(projectile)?;

        Arc::make_mut(&mut self.pending_fired_projectiles).push(ProjectileFired {
            shooter_lid,
            position,
            target,
            serialized,
        });

        //TODO initial_rng might need to be handled with np.SetProjectileSpreadRNG?

        Ok(())
    }
}

fn send_remotedes(
    ctx: &mut super::ModuleCtx<'_>,
    reliable: bool,
    destination: Destination<PeerId>,
    remote_des: RemoteDes,
) -> Result<(), eyre::Error> {
    ctx.net.send(&NoitaOutbound::RemoteMessage {
        reliable,
        destination,
        message: RemoteMessage::RemoteDes(remote_des),
    })?;
    Ok(())
}
