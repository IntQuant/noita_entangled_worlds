use std::mem;

use bitcode::{Decode, Encode};
use rstar::{RTree, primitives::GeomWithData};
use rustc_hash::FxHashMap;
use shared::WorldPos;
use shared::des::{
    DesToProxy, EntitySpawnInfo, FullEntityData, Gid, PhysBodyInfo, ProxyToDes,
    REQUEST_AUTHORITY_RADIUS, UpdateOrUpload, UpdatePosition,
};
use tracing::{info, warn};

use crate::bookkeeping::save_state::{SaveState, SaveStateEntry};

use super::omni::OmniPeerId;

#[derive(Encode, Decode, Default)]
struct EntityStorage {
    entities: FxHashMap<Gid, FullEntityData>,
}

impl SaveStateEntry for EntityStorage {
    /// Deliberately not the name the pre max_hp builds used. This storage is bitcode encoded and
    /// bitcode is not self describing, so which struct a file holds cannot be recovered from its
    /// bytes: an old payload usually fails to decode as the new shape, but it can also succeed and
    /// hand back entities with a shuffled counter, an emptied phys list or a garbage max hp. The
    /// filename is the only honest discriminator, so each format gets its own.
    const FILENAME: &'static str = "des_entity_storage_v2";
}

/// `FullEntityData` as it was before entities carried their owner's max hp.
///
/// Kept so a save from an older build can be migrated instead of dropped. Losing it is not a
/// cosmetic reset: on load Noita restores DES entities from its own world save and `on_new_entity`
/// kills every one of them on sight, trusting this store to hand them back through `GotAuthority`.
/// With nothing to hand back, a continued run would quietly lose every synced enemy, wand and
/// ground item on every peer, permanently.
#[derive(Encode, Decode)]
struct OldFullEntityData {
    gid: Gid,
    pos: WorldPos,
    data: EntitySpawnInfo,
    wand: Option<Vec<u8>>,
    hp: f32,
    drops_gold: bool,
    is_charmed: bool,
    counter: u8,
    phys: Vec<Option<PhysBodyInfo>>,
    synced_var: Vec<(String, String, i32, f32, bool)>,
}

#[derive(Encode, Decode, Default)]
struct OldEntityStorage {
    entities: FxHashMap<Gid, OldFullEntityData>,
}

impl SaveStateEntry for OldEntityStorage {
    const FILENAME: &'static str = "des_entity_storage";
}

impl From<OldFullEntityData> for FullEntityData {
    fn from(old: OldFullEntityData) -> Self {
        Self {
            gid: old.gid,
            pos: old.pos,
            data: old.data,
            wand: old.wand,
            hp: old.hp,
            // These entities predate max hp sync, so nobody has anything to say about theirs until
            // an owner collects one - which is exactly what `None` already means.
            max_hp: None,
            drops_gold: old.drops_gold,
            is_charmed: old.is_charmed,
            counter: old.counter,
            phys: old.phys,
            synced_var: old.synced_var,
        }
    }
}

pub(crate) struct DesManager {
    is_host: bool,
    entity_storage: EntityStorage,
    rtree: RTree<GeomWithData<[i64; 2], Gid>>,
    authority: FxHashMap<Gid, OmniPeerId>,
    pending_messages: Vec<(OmniPeerId, ProxyToDes)>,
    save_state: SaveState,
}

impl DesManager {
    pub(crate) fn new(is_host: bool, save_state: SaveState) -> Self {
        info!("Loading EntityStorage...");
        let entity_storage: EntityStorage = save_state
            .load()
            .or_else(|| {
                let old: OldEntityStorage = save_state.load()?;
                info!(
                    "Migrating {} entities from the pre max_hp save format",
                    old.entities.len()
                );
                Some(EntityStorage {
                    entities: old
                        .entities
                        .into_iter()
                        .map(|(gid, ent)| (gid, ent.into()))
                        .collect(),
                })
            })
            .unwrap_or_default();

        info!("Preparing elements...");
        let elements: Vec<_> = entity_storage
            .entities
            .iter()
            .map(|(&gid, ent)| GeomWithData::new(ent.pos.as_array(), gid))
            .collect();
        info!("Building RTree of {} elements...", elements.len());
        let rtree = RTree::bulk_load(elements);
        info!("DesManager created!");
        Self {
            entity_storage,
            rtree,
            authority: Default::default(),
            pending_messages: Vec::new(),
            save_state,
            is_host,
        }
    }

    fn remove_gid_from_tree(&mut self, gid: Gid) {
        if let Some(entity) = self.entity_storage.entities.get(&gid) {
            self.rtree
                .remove(&GeomWithData::new(entity.pos.as_array(), gid));
        }
    }

    fn add_gid_to_tree(&mut self, gid: Gid) {
        if let Some(entity) = self.entity_storage.entities.get(&gid) {
            let t = GeomWithData::new(entity.pos.as_array(), gid);
            self.rtree.remove(&t); // Makes sure there isn't a way to add the same point twice somehow.
            self.rtree.insert(t);
        }
    }

    fn handle_update(&mut self, update: UpdateOrUpload, source: OmniPeerId) {
        match update {
            UpdateOrUpload::Upload(full_entity_data) => {
                self.authority.insert(full_entity_data.gid, source);
                self.entity_storage
                    .entities
                    .insert(full_entity_data.gid, full_entity_data);
            }
            UpdateOrUpload::Update(update) => {
                let UpdatePosition {
                    gid,
                    pos,
                    counter,
                    is_charmed,
                    hp,
                    max_hp,
                    phys,
                    synced_var,
                } = update;
                self.remove_gid_from_tree(gid);
                if let Some(entity) = self.entity_storage.entities.get_mut(&gid) {
                    entity.pos = pos;
                    entity.is_charmed = is_charmed;
                    entity.hp = hp;
                    entity.max_hp = max_hp;
                    entity.counter = counter;
                    entity.phys = phys;
                    entity.synced_var = synced_var;
                } else {
                    warn!("Failed to find entity {gid:?} to update");
                }
            }
        }
    }

    pub(crate) fn handle_noita_msg(&mut self, source: OmniPeerId, msg: DesToProxy) {
        // TODO maybe check that authorities are correct?
        match msg {
            DesToProxy::UpdateWand(gid, wand) => {
                self.entity_storage
                    .entities
                    .entry(gid)
                    .and_modify(|data| data.wand = wand);
            }
            DesToProxy::DeleteEntity(gid, ent) => {
                if self.entity_storage.entities.contains_key(&gid) {
                    self.authority.remove(&gid);
                    self.entity_storage.entities.remove(&gid);
                    self.remove_gid_from_tree(gid);
                } else if let Some(ent) = ent {
                    self.pending_messages
                        .push((source, ProxyToDes::DeleteEntity(ent)));
                }
            }
            DesToProxy::ReleaseAuthority(gid) => {
                self.authority.remove(&gid);
                self.add_gid_to_tree(gid);
            }
            DesToProxy::RequestAuthority { pos } => {
                // drain_within_distance panics without this check. Funny.
                if self.rtree.size() == 0 {
                    return;
                }
                let radius = REQUEST_AUTHORITY_RADIUS;
                let mut auths = Vec::new();
                for point in self
                    .rtree
                    .drain_within_distance(pos.as_array(), i64::from(radius).pow(2))
                {
                    let gid = point.data;
                    self.authority.insert(gid, source);
                    if let Some(entity) = self.entity_storage.entities.get(&gid).cloned() {
                        auths.push(entity)
                    } else {
                        warn!("Expected to find entity data to give authority");
                    }
                }
                if !auths.is_empty() {
                    if auths.len() == 1 {
                        self.pending_messages
                            .push((source, ProxyToDes::GotAuthority(auths.pop().unwrap())));
                    } else {
                        self.pending_messages
                            .push((source, ProxyToDes::GotAuthoritys(auths)))
                    }
                }
            }
            DesToProxy::UpdatePosition(update) => self.handle_update(update, source),
            DesToProxy::UpdatePositions(updates) => {
                for update in updates {
                    self.handle_update(update, source)
                }
            }
            DesToProxy::TransferAuthorityTo(gid, peer_id) => {
                if let Some(entity) = self.entity_storage.entities.get(&gid).cloned() {
                    //info!("Transferring authority over entity from {source:?} to {peer_id:?}");
                    self.authority.insert(gid, peer_id.into());
                    self.pending_messages
                        .push((peer_id.into(), ProxyToDes::GotAuthority(entity)));
                } else {
                    warn!("Failed to find entity {gid:?} to transfer authority");
                }
            }
        }
    }

    pub(crate) fn noita_disconnected(&mut self, source: OmniPeerId) {
        // TODO also remove entities from affected clients faster.
        info!("Peer {source} disconnected, freeing entities that were under authority");
        let mut free_again = Vec::new();
        self.authority.retain(|gid, authority| {
            let remove = source == *authority;
            if remove {
                free_again.push(*gid);
            }
            !remove
        });
        for gid in free_again {
            self.add_gid_to_tree(gid);
        }
    }

    pub(crate) fn pending_messages(&mut self) -> Vec<(OmniPeerId, ProxyToDes)> {
        mem::take(&mut self.pending_messages)
    }

    pub(crate) fn reset(&mut self) {
        self.entity_storage = Default::default();
        self.rtree = RTree::default();
        self.authority.clear();
        self.pending_messages.clear();
    }
}

impl Drop for DesManager {
    fn drop(&mut self) {
        if self.is_host {
            self.save_state.save(&self.entity_storage);
        }
    }
}
