use std::mem;

use bitcode::{Decode, Encode};
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use world_model::{ChunkCoord, ChunkData, ChunkDelta, WorldModel};

pub use world_model::encoding::NoitaWorldUpdate;

use super::{messages::Destination, omni::OmniPeerId};

pub mod world_info;
pub mod world_model;

#[derive(Debug, Serialize, Deserialize)]
pub enum WorldUpdateKind {
    Update(NoitaWorldUpdate),
    End,
}

#[derive(Debug, Decode, Encode)]
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
        chunk_data: ChunkData,
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

enum ChunkState {
    Unsynced,
    Listening { authority: OmniPeerId },
    Authority { listeners: FxHashSet<OmniPeerId> },
}
impl ChunkState {
    fn authority() -> ChunkState {
        ChunkState::Authority {
            listeners: Default::default(),
        }
    }
}

pub struct WorldManager {
    is_host: bool,
    /// We receive changes from other clients here, intending to send them to Noita.
    inbound_model: WorldModel,
    /// We use that to create changes to be sent to other clients.
    outbound_model: WorldModel,
    /// Current
    chunk_storage: FxHashMap<ChunkCoord, ChunkData>,
    /// Who is the current chunk authority.
    authority_map: FxHashMap<ChunkCoord, OmniPeerId>,
    /// Which chunks we have authority of.
    //authority_of: FxHashSet<ChunkCoord>,
    chunk_state: FxHashMap<ChunkCoord, ChunkState>,
}

impl WorldManager {
    pub fn new(is_host: bool) -> Self {
        WorldManager {
            is_host,
            inbound_model: Default::default(),
            outbound_model: Default::default(),
            authority_map: Default::default(),
            // TODO this needs to be persisted between proxy restarts.
            chunk_storage: Default::default(),
            chunk_state: Default::default(),
        }
    }

    pub(crate) fn add_update(&mut self, update: NoitaWorldUpdate) {
        self.outbound_model.apply_noita_update(&update);
    }

    pub(crate) fn add_end(&mut self) -> WorldDelta {
        let deltas = self.inbound_model.get_all_deltas();
        self.outbound_model.reset_change_tracking();
        WorldDelta(deltas)
    }

    pub(crate) fn handle_deltas(&mut self, deltas: WorldDelta) {
        self.inbound_model.apply_all_deltas(&deltas.0);
    }

    pub(crate) fn get_noita_updates(&mut self) -> Vec<Vec<u8>> {
        let updates = self.inbound_model.get_all_noita_updates();
        self.inbound_model.reset_change_tracking();
        updates
    }

    pub(crate) fn reset(&mut self) {
        self.inbound_model.reset();
        self.outbound_model.reset();
    }

    fn emit_msg(&mut self, dst: Destination, msg: WorldNetMessage) {
        todo!()
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
                // TODO these chunks will need to be returned to host.
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
                // TODO notify others?
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
                self.emit_msg(
                    Destination::Peer(source),
                    WorldNetMessage::ListenInitialResponse {
                        chunk,
                        chunk_data: self.outbound_model.get_chunk_data(chunk),
                    },
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
                let Some(ChunkState::Listening { authority }) =
                    self.chunk_state.get_mut(&delta.chunk_coord)
                else {
                    warn!(
                        "Can't handle ListenUpdate for {:?} - not a listener",
                        delta.chunk_coord
                    );
                    return;
                };
            }
            WorldNetMessage::ListenAuthorityRelinquished { chunk } => {
                self.chunk_state.insert(chunk, ChunkState::Unsynced);
            }
        }
    }
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
