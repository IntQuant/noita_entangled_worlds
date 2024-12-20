//! Distibuted Entity Sync, a.k.a. DES.
//! The idea is that we completely disregard the normal saving system for entities we sync.
//! Also, each entity gets an owner.
//! Each peer broadcasts an "Interest" zone. If it intersects any peer they receive all information about entities this peer owns.

use diff_model::{LocalDiffModel, RemoteDiffModel, DES_TAG};
use eyre::Context;
use interest::InterestTracker;
use noita_api::{game_print, EntityID};
use rustc_hash::FxHashMap;
use shared::{
    des::{Gid, InterestRequest, RemoteDes},
    Destination, NoitaOutbound, PeerId, RemoteMessage, WorldPos,
};

use super::Module;

mod diff_model;
mod interest;

pub(crate) struct EntitySync {
    /// Last entity we stopped looking for new tracked entities at.
    look_current_entity: EntityID,

    interest_tracker: InterestTracker,
    local_diff_model: LocalDiffModel,
    remote_models: FxHashMap<PeerId, RemoteDiffModel>,
}

impl Default for EntitySync {
    fn default() -> Self {
        Self {
            look_current_entity: EntityID::try_from(1).unwrap(),

            interest_tracker: InterestTracker::new(512.0),
            local_diff_model: LocalDiffModel::default(),
            remote_models: Default::default(),
        }
    }
}

impl EntitySync {
    fn should_be_tracked(&mut self, entity: EntityID) -> eyre::Result<bool> {
        Ok(entity.has_tag("enemy"))
    }

    /// Looks for newly spawned entities that might need to be tracked.
    fn look_for_tracked(&mut self) -> eyre::Result<()> {
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
                game_print(format!("Tracking {entity:?}"));
                self.local_diff_model.track_entity(entity)?;
            }
        }

        self.look_current_entity = max_entity;
        Ok(())
    }

    pub(crate) fn handle_proxytodes(&mut self, proxy_to_des: shared::des::ProxyToDes) {
        todo!()
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
        }
    }
}

impl Module for EntitySync {
    fn on_world_init(&mut self, ctx: &mut super::ModuleCtx) -> eyre::Result<()> {
        send_remotedes(ctx, true, Destination::Broadcast, RemoteDes::Reset)?;
        Ok(())
    }

    fn on_world_update(&mut self, ctx: &mut super::ModuleCtx) -> eyre::Result<()> {
        self.look_for_tracked()
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
                    radius: 1024,
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

        if frame_num % 2 == 0 {
            self.local_diff_model
                .update_tracked_entities()
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
                    RemoteDes::EntityUpdate(diff.clone()),
                )?;
            }
        } else {
            for remote_model in self.remote_models.values_mut() {
                remote_model.apply_entities()?;
            }
        }
        // These entities shouldn't be tracked by us, as they were spawned by remote.
        self.look_current_entity = EntityID::max_in_use()?;

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
