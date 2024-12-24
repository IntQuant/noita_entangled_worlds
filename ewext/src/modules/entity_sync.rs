//! Distibuted Entity Sync, a.k.a. DES.
//! The idea is that we completely disregard the normal saving system for entities we sync.
//! Also, each entity gets an owner.
//! Each peer broadcasts an "Interest" zone. If it intersects any peer they receive all information about entities this peer owns.

use std::sync::{Arc, LazyLock};

use diff_model::{entity_is_item, LocalDiffModel, RemoteDiffModel, DES_TAG};
use eyre::{Context, OptionExt};
use interest::InterestTracker;
use noita_api::{game_print, EntityID, ProjectileComponent};
use rustc_hash::{FxHashMap, FxHashSet};
use shared::{
    des::{
        Gid, InterestRequest, ProjectileFired, RemoteDes, INTEREST_REQUEST_RADIUS,
        REQUEST_AUTHORITY_RADIUS,
    },
    Destination, NoitaOutbound, PeerId, RemoteMessage, WorldPos,
};

use crate::serialize::serialize_entity;

use super::{Module, NetManager};

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
}

impl Default for EntitySync {
    fn default() -> Self {
        Self {
            look_current_entity: EntityID::try_from(1).unwrap(),

            interest_tracker: InterestTracker::new(512.0),
            local_diff_model: LocalDiffModel::default(),
            remote_models: Default::default(),

            pending_fired_projectiles: Vec::new().into(),
        }
    }
}

fn entity_is_excluded(entity: EntityID) -> eyre::Result<bool> {
    let filename = entity.filename()?;
    Ok(ENTITY_EXCLUDES.contains(&filename))
}

impl EntitySync {
    fn should_be_tracked(&mut self, entity: EntityID) -> eyre::Result<bool> {
        let should_be_tracked =
            entity.has_tag("enemy") || entity.has_tag("ew_synced") || entity_is_item(entity)?;

        Ok(should_be_tracked && !entity_is_excluded(entity)?)
    }

    /// Looks for newly spawned entities that might need to be tracked.
    fn look_for_tracked(&mut self, ctx: &mut super::ModuleCtx) -> eyre::Result<()> {
        let max_entity = EntityID::max_in_use()?;
        for i in (self.look_current_entity.raw() + 1)..=max_entity.raw() {
            let entity = EntityID::try_from(i)?;
            if !entity.is_alive() {
                continue;
            }
            if entity.has_tag(DES_TAG) {
                entity.kill();
                continue;
            }
            if self.should_be_tracked(entity)? {
                let gid = shared::des::Gid(rand::random());
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
        }
    }

    pub(crate) fn handle_remotedes(&mut self, source: PeerId, remote_des: RemoteDes) {
        match remote_des {
            RemoteDes::InterestRequest(interest_request) => self
                .interest_tracker
                .handle_interest_request(source, interest_request),
            RemoteDes::EntityUpdate(vec) => {
                self.remote_models
                    .entry(source)
                    .or_default()
                    .apply_diff(&vec);
            }
            RemoteDes::ExitedInterest => {
                self.remote_models.remove(&source);
            }
            RemoteDes::Reset => self.interest_tracker.reset_interest_for(source),
            RemoteDes::Projectiles(vec) => {
                self.remote_models
                    .entry(source)
                    .or_default()
                    .spawn_projectiles(&vec);
            }
            RemoteDes::RequestGrab(lid) => {
                self.local_diff_model.entity_grabbed(source, lid);
            }
        }
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

        self.local_diff_model.update_pending_authority()?;

        if frame_num % 2 == 0 {
            self.local_diff_model
                .update_tracked_entities(ctx)
                .wrap_err("Failed to update locally tracked entities")?;
            if self.interest_tracker.got_any_new_interested() {
                game_print("Got new interested");
                self.local_diff_model.reset_diff_encoding();
            }
            let diff = self.local_diff_model.make_diff();
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
            Arc::make_mut(&mut self.pending_fired_projectiles).clear();
        } else {
            for (owner, remote_model) in &mut self.remote_models {
                remote_model.apply_entities()?;
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
            // TODO also send positions periodically.
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

        if proj_component.m_entity_that_shot()?.is_some() {
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
