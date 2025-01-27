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
        }
    }
}

fn entity_is_excluded(entity: EntityID) -> eyre::Result<bool> {
    let good = "data/entities/items/wands/wand_good/wand_good_";
    let filename = entity.filename()?;
    Ok(entity.has_tag("ew_no_enemy_sync")
        || entity.has_tag("polymorphed_player")
        || entity.has_tag("gold_nugget")
        || ENTITY_EXCLUDES.contains(&filename)
        || filename.starts_with(good))
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
        let file_name = entity.filename().unwrap_or_default();
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
            || [
                "data/entities/buildings/essence_eater.xml",
                "data/entities/animals/boss_fish/fish_giga.xml",
                "data/entities/buildings/spittrap_left.xml",
                "data/entities/buildings/spittrap_right.xml",
                "data/entities/buildings/thundertrap_left.xml",
                "data/entities/buildings/thundertrap_right.xml",
                "data/entities/buildings/arrowtrap_left.xml",
                "data/entities/buildings/arrowtrap_right.xml",
                "data/entities/buildings/firetrap_left.xml",
                "data/entities/buildings/firetrap_right.xml",
                "data/entities/buildings/statue_trap_left.xml",
                "data/entities/buildings/statue_trap_right.xml",
                "data/entities/animals/boss_limbs/boss_limbs_trigger.xml",
                "data/entities/animals/boss_spirit/spawner.xml",
                "data/entities/misc/orb_07_pitcheck_a.xml",
                "data/entities/misc/orb_07_pitcheck_b.xml",
                "data/entities/buildings/maggotspot.xml",
                "data/entities/buildings/dragonspot.xml",
                "data/entities/buildings/wizardcave_gate.xml",
                "data/entities/buildings/wallmouth.xml",
                "data/entities/buildings/walleye.xml",
                "data/entities/buildings/bunker.xml",
                "data/entities/buildings/bunker2.xml",
            ]
            .contains(&file_name.as_str())
            || entity_is_item(entity)?;

        Ok(should_be_tracked && !entity_is_excluded(entity)?)
    }

    /// Looks for newly spawned entities that might need to be tracked.
    fn look_for_tracked(&mut self, ctx: &mut super::ModuleCtx) -> eyre::Result<()> {
        let max_entity = EntityID::max_in_use()?;
        for i in (self.look_current_entity.raw() + 1)..=max_entity.raw() {
            let entity = EntityID::try_from(i)?;
            if !entity.is_alive() || self.dont_track.remove(&entity) {
                continue;
            }
            if let Ok(Some(gid)) = entity.handle_poly() {
                self.dont_kill_by_gid.insert(gid);
            }
            if entity.has_tag(DES_TAG)
                && !self.dont_kill.remove(&entity)
                && entity
                    .iter_all_components_of_type_including_disabled::<VariableStorageComponent>(
                        None,
                    )?
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
                continue;
            }
            if self.should_be_tracked(entity)? {
                let gid = Gid(rand::random());
                self.local_diff_model
                    .track_and_upload_entity(ctx.net, entity, gid)?;
            }
        }

        self.look_current_entity = max_entity;
        Ok(())
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
                        .find(|l| {
                            !l.script_physics_body_modified()
                                .unwrap_or("".into())
                                .is_empty()
                        })
                        .map(|l| l.script_physics_body_modified().unwrap_or("".into()))
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
                self.remote_models
                    .entry(source)
                    .or_insert(RemoteDiffModel::new(source))
                    .apply_diff(&vec);
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
        self.local_diff_model
            .track_and_upload_entity(net, entity, Gid(rand::random()))?;
        Ok(())
    }

    pub(crate) fn cross_death_notify(
        &mut self,
        entity_killed: EntityID,
        wait_on_kill: bool,
        drops_gold: bool,
        pos: WorldPos,
        file: String,
        entity_responsible: Option<EntityID>,
    ) -> eyre::Result<()> {
        self.local_diff_model.death_notify(
            entity_killed,
            wait_on_kill,
            drops_gold,
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

    fn on_world_update(&mut self, ctx: &mut super::ModuleCtx) -> eyre::Result<()> {
        self.look_for_tracked(ctx)
            .wrap_err("Error in look_for_tracked")?;

        let (x, y) = noita_api::raw::game_get_camera_pos()?;
        self.interest_tracker.set_center(x, y);
        let frame_num = noita_api::raw::game_get_frame_num()?;
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

        let mut i = self.spawn_once.len();
        while i != 0 {
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
                                diff_model::init_remote_entity(entity, None, None, *drops_gold)?;
                                if let Some(damage) =
                                    entity.try_get_first_component::<DamageModelComponent>(None)?
                                {
                                    damage.set_ui_report_damage(false)?;
                                    damage.set_hp(f32::MIN_POSITIVE as f64)?;
                                    let responsible_entity = offending_peer
                                        .and_then(|peer| ctx.player_map.get_by_left(&peer))
                                        .copied();
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

        self.local_diff_model.update_pending_authority()?;

        if ctx.sync_rate == 1 || frame_num % ctx.sync_rate == 0 {
            self.local_diff_model
                .update_tracked_entities(ctx)
                .wrap_err("Failed to update locally tracked entities")?;
            if self.interest_tracker.got_any_new_interested() {
                //game_print("Got new interested");
                self.local_diff_model.reset_diff_encoding();
            }
            let (diff, dead) = self.local_diff_model.make_diff(ctx);
            // FIXME (perf): allow a Destination that can send to several peers at once, to prevent cloning and stuff.
            for peer in self.interest_tracker.iter_interested() {
                send_remotedes(
                    ctx,
                    true,
                    Destination::Peer(peer),
                    RemoteDes::Projectiles(self.pending_fired_projectiles.clone()),
                )?;
                send_remotedes(
                    ctx,
                    true,
                    Destination::Peer(peer),
                    RemoteDes::EntityUpdate(diff.clone()),
                )?;
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
        if ctx.sync_rate == 1 || frame_num % ctx.sync_rate == 1 {
            for (owner, remote_model) in &mut self.remote_models {
                remote_model
                    .apply_entities(ctx)
                    .wrap_err("Failed to apply entity infos")?;
                for entity in remote_model.drain_backtrack() {
                    self.local_diff_model.track_and_upload_entity(
                        ctx.net,
                        entity,
                        Gid(rand::random()),
                    )?;
                }
                for lid in remote_model.drain_grab_request() {
                    send_remotedes(
                        ctx,
                        true,
                        Destination::Peer(*owner),
                        RemoteDes::RequestGrab(lid),
                    )?;
                }
            }
        }

        if frame_num % 60 == 0 {
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
        // These entities shouldn't be tracked by us, as they were spawned by remote.
        self.look_current_entity = EntityID::max_in_use()?;

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
