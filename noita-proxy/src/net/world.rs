use std::mem;

use bitcode::{Decode, Encode};
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use world_model::{ChunkCoord, ChunkData, ChunkDelta, WorldModel};

pub use world_model::encoding::NoitaWorldUpdate;

use crate::bookkeeping::save_state::{SaveState, SaveStateEntry};

use super::{
    messages::{Destination, MessageRequest},
    omni::OmniPeerId,
};

pub mod world_info;
pub mod world_model;

#[derive(Debug, Serialize, Deserialize)]
pub enum WorldUpdateKind {
    Update(NoitaWorldUpdate),
    End,
}

#[derive(Debug, Decode, Encode, Clone)]
pub(crate) enum WorldNetMessage {
    // Authority request
    RequestAuthority {
        chunk: ChunkCoord,
    },
    // When got authority
    GotAuthority {
        chunk: ChunkCoord,
        chunk_data: Option<ChunkData>,
    },
    RelinquishAuthority {
        chunk: ChunkCoord,
        chunk_data: Option<ChunkData>,
    },
    // When listening
    AuthorityAlreadyTaken {
        chunk: ChunkCoord,
        authority: OmniPeerId,
    },
    ListenRequest {
        chunk: ChunkCoord,
    },
    ListenStopRequest {
        chunk: ChunkCoord,
    },
    // Listen responses/messages
    ListenInitialResponse {
        chunk: ChunkCoord,
        chunk_data: Option<ChunkData>,
    },
    ListenUpdate {
        delta: ChunkDelta,
    },
    ListenAuthorityRelinquished {
        chunk: ChunkCoord,
    },
}

#[derive(Debug, Decode, Encode)]
pub struct WorldDelta(Vec<ChunkDelta>);

impl WorldDelta {
    pub fn split(self, limit: usize) -> Vec<WorldDelta> {
        let mut res = Vec::new();
        let mut current = Vec::new();
        let mut current_size = 0;
        for delta in self.0 {
            if current_size < limit || current.is_empty() {
                current_size += delta.estimate_size();
                current.push(delta);
            } else {
                res.push(WorldDelta(mem::take(&mut current)));
                current_size = 0;
            }
        }
        if !current.is_empty() {
            res.push(WorldDelta(mem::take(&mut current)));
        }
        res
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ChunkState {
    /// Chunk isn't synced yet, but will request authority for it.
    RequestAuthority,
    /// Transitioning into Listening or Authority state.
    WaitingForAuthority,
    /// Listening for chunk updates from this peer.
    Listening { authority: OmniPeerId },
    /// Sending chunk updates to these listeners.
    Authority { listeners: FxHashSet<OmniPeerId> },
    /// Chunk is to be cleaned up.
    UnloadPending,
}
impl ChunkState {
    fn authority() -> ChunkState {
        ChunkState::Authority {
            listeners: Default::default(),
        }
    }
}
// TODO handle exits.
pub(crate) struct WorldManager {
    is_host: bool,
    my_peer_id: OmniPeerId,
    save_state: SaveState,
    /// We receive changes from other clients here, intending to send them to Noita.
    inbound_model: WorldModel,
    /// We use that to create changes to be sent to other clients.
    outbound_model: WorldModel,
    /// Stores chunks that aren't under any authority.
    chunk_storage: FxHashMap<ChunkCoord, ChunkData>,
    /// Who is the current chunk authority.
    authority_map: FxHashMap<ChunkCoord, OmniPeerId>,
    /// Chunk states, according to docs/distributed_world_sync.drawio
    chunk_state: FxHashMap<ChunkCoord, ChunkState>,
    emitted_messages: Vec<MessageRequest<WorldNetMessage>>,
    /// Which update it is?
    /// Incremented every time `add_end()` gets called.
    current_update: u64,
    /// Update number in which chunk has been updated locally.
    /// Used to track which chunks can be unloaded.
    chunk_last_update: FxHashMap<ChunkCoord, u64>,
}

impl WorldManager {
    pub(crate) fn new(is_host: bool, my_peer_id: OmniPeerId, save_state: SaveState) -> Self {
        let chunk_storage = save_state.load().unwrap_or_default();
        WorldManager {
            is_host,
            my_peer_id,
            save_state,
            inbound_model: Default::default(),
            outbound_model: Default::default(),
            authority_map: Default::default(),
            // TODO this needs to be persisted between proxy restarts.
            chunk_storage,
            chunk_state: Default::default(),
            emitted_messages: Default::default(),
            current_update: 0,
            chunk_last_update: Default::default(),
        }
    }

    pub(crate) fn add_update(&mut self, update: NoitaWorldUpdate) {
        self.outbound_model.apply_noita_update(&update);
    }

    pub(crate) fn add_end(&mut self) {
        let updated_chunks = self
            .outbound_model
            .updated_chunks()
            .iter()
            .copied()
            .collect::<Vec<_>>();
        self.current_update += 1;
        for chunk in updated_chunks {
            self.chunk_updated_locally(chunk);
        }
        self.outbound_model.reset_change_tracking();
    }

    fn chunk_updated_locally(&mut self, chunk: ChunkCoord) {
        let entry = self.chunk_state.entry(chunk).or_insert_with(|| {
            debug!("Created entry for {chunk:?}");
            ChunkState::RequestAuthority
        });
        let mut emit_queue = Vec::new();
        self.chunk_last_update.insert(chunk, self.current_update);
        if let ChunkState::Authority { listeners } = entry {
            let Some(delta) = self.outbound_model.get_chunk_delta(chunk, false) else {
                return;
            };
            for &listener in listeners.iter() {
                emit_queue.push((
                    Destination::Peer(listener),
                    WorldNetMessage::ListenUpdate {
                        delta: delta.clone(),
                    },
                ));
            }
        }
        for (dst, msg) in emit_queue {
            self.emit_msg(dst, msg)
        }
    }

    pub(crate) fn update(&mut self) {
        let mut emit_queue = Vec::new();

        // How many updates till we relinquish authority/stop listening.
        let unload_limit = 6;

        for (&chunk, state) in self.chunk_state.iter_mut() {
            let chunk_last_update = self
                .chunk_last_update
                .get(&chunk)
                .copied()
                .unwrap_or_default();
            match state {
                ChunkState::RequestAuthority => {
                    emit_queue.push((
                        Destination::Host,
                        WorldNetMessage::RequestAuthority { chunk },
                    ));
                    *state = ChunkState::WaitingForAuthority;
                    info!("Requested authority for {chunk:?}")
                }
                // This state doesn't have much to do.
                ChunkState::WaitingForAuthority => {}
                ChunkState::Listening { authority } => {
                    if self.current_update - chunk_last_update > unload_limit {
                        info!("Unloading [listening] chunk {chunk:?}");
                        emit_queue.push((
                            Destination::Peer(*authority),
                            WorldNetMessage::ListenStopRequest { chunk },
                        ));
                        *state = ChunkState::UnloadPending;
                    }
                }
                ChunkState::Authority { listeners: _ } => {
                    if self.current_update - chunk_last_update > unload_limit {
                        info!("Unloading [authority] chunk {chunk:?} (updates: {chunk_last_update} {})", self.current_update);
                        emit_queue.push((
                            Destination::Host,
                            WorldNetMessage::RelinquishAuthority {
                                chunk,
                                chunk_data: self.outbound_model.get_chunk_data(chunk),
                            },
                        ));
                        *state = ChunkState::UnloadPending;
                    }
                }
                ChunkState::UnloadPending => {}
            }
        }

        for (dst, msg) in emit_queue {
            self.emit_msg(dst, msg)
        }
        self.chunk_state.retain(|chunk, state| {
            let retain = *state != ChunkState::UnloadPending;
            if !retain {
                // Models are basically caches, no need to keep the chunk around in them.
                self.inbound_model.forget_chunk(*chunk);
                self.outbound_model.forget_chunk(*chunk);
            }
            retain
        });
    }

    pub(crate) fn get_noita_updates(&mut self) -> Vec<Vec<u8>> {
        let updates = self.inbound_model.get_all_noita_updates();
        self.inbound_model.reset_change_tracking();
        updates
    }

    pub(crate) fn reset(&mut self) {
        self.inbound_model.reset();
        self.outbound_model.reset();
        self.chunk_storage.clear();
    }

    pub(crate) fn get_emitted_msgs(&mut self) -> Vec<MessageRequest<WorldNetMessage>> {
        mem::take(&mut self.emitted_messages)
    }

    fn emit_msg(&mut self, dst: Destination, msg: WorldNetMessage) {
        // Short-circuit for messages intended for myself
        if (self.is_host && dst == Destination::Host) || dst == Destination::Peer(self.my_peer_id) {
            self.handle_msg(self.my_peer_id, msg);
            return;
        }
        // Also handle broadcast messages this way.
        if dst == Destination::Broadcast {
            self.handle_msg(self.my_peer_id, msg.clone());
        }

        self.emitted_messages.push(MessageRequest {
            reliability: tangled::Reliability::Reliable,
            dst,
            msg,
        })
    }

    fn emit_got_authority(&mut self, chunk: ChunkCoord, source: OmniPeerId) {
        self.authority_map.insert(chunk, source);
        let chunk_data = self.chunk_storage.get(&chunk).cloned();
        self.emit_msg(
            Destination::Peer(source),
            WorldNetMessage::GotAuthority { chunk, chunk_data },
        );
    }

    pub(crate) fn handle_msg(&mut self, source: OmniPeerId, msg: WorldNetMessage) {
        match msg {
            WorldNetMessage::RequestAuthority { chunk } => {
                if !self.is_host {
                    warn!("{} sent RequestAuthority to not-host.", source);
                    return;
                }
                let current_authority = self.authority_map.get(&chunk).copied();
                match current_authority {
                    Some(authority) => {
                        if source == authority {
                            info!("{source} already has authority of {chunk:?}");
                            self.emit_got_authority(chunk, source);
                        } else {
                            info!("{source} requested authority for {chunk:?}, but it's already taken by {authority}");
                            self.emit_msg(
                                Destination::Peer(source),
                                WorldNetMessage::AuthorityAlreadyTaken { chunk, authority },
                            );
                        }
                    }
                    None => {
                        info!("Granting {source} authority of {chunk:?}");
                        self.emit_got_authority(chunk, source);
                    }
                }
            }
            WorldNetMessage::GotAuthority { chunk, chunk_data } => {
                if let Some(chunk_data) = chunk_data {
                    self.inbound_model.apply_chunk_data(chunk, chunk_data);
                }
                self.chunk_state.insert(chunk, ChunkState::authority());
            }
            WorldNetMessage::RelinquishAuthority { chunk, chunk_data } => {
                if !self.is_host {
                    warn!("{} sent RelinquishAuthority to not-host.", source);
                    return;
                }
                if self.authority_map.get(&chunk) != Some(&source) {
                    warn!("{source} sent RelinquishAuthority for {chunk:?}, but isn't currently an authority");
                    return;
                }
                self.authority_map.remove(&chunk);
                if let Some(chunk_data) = chunk_data {
                    self.chunk_storage.insert(chunk, chunk_data);
                }
                self.emit_msg(
                    Destination::Broadcast,
                    WorldNetMessage::ListenAuthorityRelinquished { chunk },
                )
            }

            WorldNetMessage::AuthorityAlreadyTaken { chunk, authority } => {
                // TODO what to do in case we won't get a response?
                self.emit_msg(
                    Destination::Peer(authority),
                    WorldNetMessage::ListenRequest { chunk },
                );
            }
            WorldNetMessage::ListenRequest { chunk } => {
                let Some(ChunkState::Authority { listeners }) = self.chunk_state.get_mut(&chunk)
                else {
                    warn!("Can't listen for {chunk:?} - not an authority");
                    return;
                };
                listeners.insert(source);
                let chunk_data = self.outbound_model.get_chunk_data(chunk);
                self.emit_msg(
                    Destination::Peer(source),
                    WorldNetMessage::ListenInitialResponse { chunk, chunk_data },
                );
            }
            WorldNetMessage::ListenStopRequest { chunk } => {
                let Some(ChunkState::Authority { listeners }) = self.chunk_state.get_mut(&chunk)
                else {
                    warn!("Can't stop listen for {chunk:?} - not an authority");
                    return;
                };
                listeners.remove(&source);
            }
            WorldNetMessage::ListenInitialResponse { chunk, chunk_data } => {
                self.chunk_state
                    .insert(chunk, ChunkState::Listening { authority: source });
                if let Some(chunk_data) = chunk_data {
                    self.inbound_model.apply_chunk_data(chunk, chunk_data);
                } else {
                    warn!("Initial listen response has None chunk_data. It's generally supposed to have some.");
                }
            }
            WorldNetMessage::ListenUpdate { delta } => {
                let Some(ChunkState::Listening { authority: _ }) =
                    self.chunk_state.get_mut(&delta.chunk_coord)
                else {
                    warn!(
                        "Can't handle ListenUpdate for {:?} - not a listener",
                        delta.chunk_coord
                    );
                    return;
                };
                self.inbound_model.apply_chunk_delta(&delta);
            }
            WorldNetMessage::ListenAuthorityRelinquished { chunk } => {
                self.chunk_state.insert(chunk, ChunkState::UnloadPending);
            }
        }
    }

    /// Should be called when player disconnects.
    /// This frees up any authority that player had.
    pub(crate) fn handle_peer_left(&mut self, source: OmniPeerId) {
        if !self.is_host {
            return;
        }
        let mut pending_messages = Vec::new();

        for (&chunk, peer) in self.authority_map.iter() {
            if *peer == source {
                info!("Removing authority from disconnected peer: {chunk:?}");
                pending_messages.push(WorldNetMessage::ListenAuthorityRelinquished { chunk });
            }
        }
        self.authority_map.retain(|_, peer| *peer != source);

        for message in pending_messages {
            self.emit_msg(Destination::Broadcast, message)
        }
    }
}

impl Drop for WorldManager {
    fn drop(&mut self) {
        if self.is_host {
            self.save_state.save(&self.chunk_storage);
            info!("Saved chunk data");
        }
    }
}

impl SaveStateEntry for FxHashMap<ChunkCoord, ChunkData> {
    const FILENAME: &'static str = "world_chunks";
}

#[cfg(test)]
mod test {
    use std::{fs::File, io::BufReader};

    use super::{world_model::WorldModel, NoitaWorldUpdate, WorldUpdateKind};

    #[test]
    fn read_replay() {
        let mut file = BufReader::new(File::open("worldlog.bin").unwrap());
        let mut model = WorldModel::new();
        let mut model2 = WorldModel::new();
        let mut entry_id = 0;
        let mut deltas_size = 0;

        while let Ok(entry) = bincode::deserialize_from::<_, WorldUpdateKind>(&mut file)
            .inspect_err(|e| println!("{}", e))
        {
            match entry {
                WorldUpdateKind::Update(entry) => {
                    let saved = entry.save();
                    let loaded = NoitaWorldUpdate::load(&saved);
                    assert_eq!(entry, loaded);

                    model.apply_noita_update(&entry);
                    let new_update = model.get_noita_update(
                        entry.header.x,
                        entry.header.y,
                        entry.header.w as u32 + 1,
                        entry.header.h as u32 + 1,
                    );

                    assert_eq!(entry, new_update);
                }
                WorldUpdateKind::End => {
                    entry_id += 1;
                    if entry_id % 10000 == 0 {
                        let (x, y) = model.get_start();
                        let img = model.gen_image(x, y, 2048, 2048);
                        img.save(format!("/tmp/img_{}.png", entry_id)).unwrap();
                    }
                    let deltas = model.get_all_deltas();
                    deltas_size += lz4_flex::compress_prepend_size(&bitcode::encode(&deltas)).len();

                    model.reset_change_tracking();
                    model2.apply_all_deltas(&deltas);
                }
            }
        }

        let (x, y) = model.get_start();
        let img = model.gen_image(x, y, 2048 * 2, 2048 * 2);
        img.save(format!("/tmp/img_{}.png", entry_id)).unwrap();

        let img = model2.gen_image(x, y, 2048 * 2, 2048 * 2);
        img.save(format!("/tmp/img_model2.png")).unwrap();

        println!("Deltas: {} bytes", deltas_size)
    }
}
