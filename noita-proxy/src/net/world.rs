use std::{env, mem};

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
    DebugMarker,
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
        priority: u8,
        can_wait: bool,
    },
    // have peer make Authority request
    AskForAuthority {
        chunk: ChunkCoord,
        priority: u8,
    },
    // switch peer to temp authority
    LoseAuthority {
        chunk: ChunkCoord,
        new_priority: u8,
        new_authority: OmniPeerId,
    },
    // Change priority
    ChangePriority {
        chunk: ChunkCoord,
        priority: u8,
    },
    // When got authority
    GotAuthority {
        chunk: ChunkCoord,
        chunk_data: Option<ChunkData>,
        priority: u8,
    },
    // Tell host that someone is losing authority
    RelinquishAuthority {
        chunk: ChunkCoord,
        chunk_data: Option<ChunkData>,
    },
    // Ttell how to update a chunk storage
    UpdateStorage {
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
        priority: u8,
    },
    ListenUpdate {
        delta: ChunkDelta,
        priority: u8,
        take_auth: bool,
    },
    ListenAuthorityRelinquished {
        chunk: ChunkCoord,
    },
    // Authority transfer stuff (due to priority)
    GetAuthorityFrom {
        chunk: ChunkCoord,
        current_authority: OmniPeerId,
    },
    RequestAuthorityTransfer {
        chunk: ChunkCoord,
    },
    TransferOk {
        chunk: ChunkCoord,
        chunk_data: Option<ChunkData>,
        listeners: FxHashSet<OmniPeerId>,
    },
    TransferFailed {
        chunk: ChunkCoord,
    },
    NotifyNewAuthority {
        chunk: ChunkCoord,
    },
}

#[derive(Debug, PartialEq, Eq)]
enum ChunkState {
    /// Chunk isn't synced yet, but will request authority for it.
    RequestAuthority { priority: u8, can_wait: bool },
    /// Transitioning into Listening or Authority state.
    WaitingForAuthority,
    /// Listening for chunk updates from this peer.
    Listening { authority: OmniPeerId, priority: u8 },
    /// Sending chunk updates to these listeners.
    Authority {
        listeners: FxHashSet<OmniPeerId>,
        priority: u8,
        new_authority: Option<(OmniPeerId, u8)>,
        stop_sending: bool,
    },
    /// Chunk is to be cleaned up.
    UnloadPending,
    /// We've requested to take authority from someone else, and waiting for transfer to complete.
    Transfer,
    /// Has higher priority and is waiting for next chunk update
    WantToGetAuth {
        authority: OmniPeerId,
        auth_priority: u8,
        my_priority: u8,
    },
}
impl ChunkState {
    fn authority(priority: u8) -> ChunkState {
        ChunkState::Authority {
            listeners: Default::default(),
            priority,
            new_authority: None,
            stop_sending: false,
        }
    }
}
// TODO handle exits.
pub(crate) struct WorldManager {
    is_host: bool,
    my_pos: (i32, i32),
    cam_pos: (i32, i32),
    is_notplayer: bool,
    my_peer_id: OmniPeerId,
    save_state: SaveState,
    /// We receive changes from other clients here, intending to send them to Noita.
    inbound_model: WorldModel,
    /// We use that to create changes to be sent to other clients.
    outbound_model: WorldModel,
    /// Stores chunks that aren't under any authority.
    chunk_storage: FxHashMap<ChunkCoord, ChunkData>,
    /// Who is the current chunk authority.
    authority_map: FxHashMap<ChunkCoord, (OmniPeerId, u8)>,
    /// Chunk states, according to docs/distributed_world_sync.drawio
    chunk_state: FxHashMap<ChunkCoord, ChunkState>,
    emitted_messages: Vec<MessageRequest<WorldNetMessage>>,
    /// Which update it is?
    /// Incremented every time `add_end()` gets called.
    current_update: u64,
    /// Update number in which chunk has been updated locally.
    /// Used to track which chunks can be unloaded.
    chunk_last_update: FxHashMap<ChunkCoord, u64>,
    /// Stores last priority we used for that chunk, in case transfer fails and we'll need to request authority normally.
    last_request_priority: FxHashMap<ChunkCoord, u8>,
}

impl WorldManager {
    pub(crate) fn new(is_host: bool, my_peer_id: OmniPeerId, save_state: SaveState) -> Self {
        let chunk_storage = save_state.load().unwrap_or_default();
        WorldManager {
            is_host,
            my_pos: (i32::MIN / 2, i32::MIN / 2),
            cam_pos: (i32::MIN / 2, i32::MIN / 2),
            is_notplayer: false,
            my_peer_id,
            save_state,
            inbound_model: Default::default(),
            outbound_model: Default::default(),
            authority_map: Default::default(),
            chunk_storage,
            chunk_state: Default::default(),
            emitted_messages: Default::default(),
            current_update: 0,
            chunk_last_update: Default::default(),
            last_request_priority: Default::default(),
        }
    }

    pub(crate) fn add_update(&mut self, update: NoitaWorldUpdate) {
        self.outbound_model.apply_noita_update(&update);
    }

    pub(crate) fn add_end(&mut self, priority: u8, pos: Option<&[i32]>) {
        let updated_chunks = self
            .outbound_model
            .updated_chunks()
            .iter()
            .copied()
            .collect::<Vec<_>>();
        self.current_update += 1;
        for chunk in updated_chunks {
            self.chunk_updated_locally(chunk, priority, pos);
        }
        self.outbound_model.reset_change_tracking();
    }

    fn chunk_updated_locally(&mut self, chunk: ChunkCoord, priority: u8, pos: Option<&[i32]>) {
        if let Some(data) = pos {
            self.my_pos = (data[0], data[1]);
            self.cam_pos = (data[2], data[3]);
            self.is_notplayer = data[4] == 1;
        }
        let entry = self.chunk_state.entry(chunk).or_insert_with(|| {
            debug!("Created entry for {chunk:?}");
            ChunkState::RequestAuthority {
                priority,
                can_wait: true,
            }
        });
        let mut emit_queue = Vec::new();
        self.chunk_last_update.insert(chunk, self.current_update);
        match entry {
            ChunkState::Listening {
                authority,
                priority: pri,
            } => {
                if *pri > priority {
                    let cs = ChunkState::WantToGetAuth {
                        authority: *authority,
                        auth_priority: *pri,
                        my_priority: priority,
                    };
                    emit_queue.push((
                        Destination::Peer(*authority),
                        WorldNetMessage::LoseAuthority {
                            chunk,
                            new_priority: priority,
                            new_authority: self.my_peer_id,
                        },
                    ));
                    self.chunk_state.insert(chunk, cs);
                }
            }
            ChunkState::WantToGetAuth {
                authority,
                auth_priority: auth_pri,
                my_priority: my_pri,
            } => {
                if *my_pri != priority {
                    *my_pri = priority;
                    if *auth_pri <= priority {
                        let cs = ChunkState::Listening {
                            authority: *authority,
                            priority: *auth_pri,
                        };
                        self.chunk_state.insert(chunk, cs);
                    } else {
                        emit_queue.push((
                            Destination::Peer(*authority),
                            WorldNetMessage::LoseAuthority {
                                chunk,
                                new_priority: priority,
                                new_authority: self.my_peer_id,
                            },
                        ));
                    }
                }
            }
            ChunkState::Authority {
                listeners,
                priority: pri,
                new_authority,
                stop_sending,
            } => {
                let Some(delta) = self.outbound_model.get_chunk_delta(chunk, false) else {
                    return;
                };
                if *pri != priority {
                    *pri = priority;
                    emit_queue.push((
                        Destination::Host,
                        WorldNetMessage::ChangePriority { chunk, priority },
                    ));
                }
                let mut new_auth = None;
                if let Some(new) = new_authority {
                    if new.1 >= priority {
                        *new_authority = None;
                        *stop_sending = false
                    } else {
                        new_auth = Some(new.0)
                    }
                } else {
                    *stop_sending = false
                }
                let mut new_auth_got = false;
                if !*stop_sending {
                    for &listener in listeners.iter() {
                        let take_auth = new_auth == Some(listener);
                        if take_auth {
                            new_auth_got = true
                        }
                        emit_queue.push((
                            Destination::Peer(listener),
                            WorldNetMessage::ListenUpdate {
                                delta: delta.clone(),
                                priority,
                                take_auth,
                            },
                        ));
                    }
                }
                if new_auth_got && new_auth.is_some() {
                    *stop_sending = true
                }
            }
            _ => {}
        }
        for (dst, msg) in emit_queue {
            self.emit_msg(dst, msg)
        }
    }

    pub(crate) fn update(&mut self) {
        fn should_kill(my_pos: (i32, i32), cam_pos: (i32, i32), chx: i32, chy: i32, is_notplayer: bool) -> bool {
            let (x, y) = my_pos;
            let (cx, cy) = cam_pos;
            if (x - cx).abs() > 2 || (y - cy).abs() > 2 {
                !(chx <= x + 2 && chx >= x - 2 && chy <= y + 2 && chy >= y - 2)
                    && !(chx <= cx + 2 && chx >= cx - 2 && chy <= cy + 2 && chy >= cy - 2)
            } else if is_notplayer {
                !(chx <= x + 2 && chx >= x - 2 && chy <= y + 2 && chy >= y - 2)
            } else {
                !(chx <= x + 3 && chx >= x - 3 && chy <= y + 3 && chy >= y - 3)
            }
        }
        let mut emit_queue = Vec::new();
        for (&chunk, state) in self.chunk_state.iter_mut() {
            let chunk_last_update = self
                .chunk_last_update
                .get(&chunk)
                .copied()
                .unwrap_or_default();
            match state {
                ChunkState::RequestAuthority { priority, can_wait } => {
                    let priority = *priority;
                    emit_queue.push((
                        Destination::Host,
                        WorldNetMessage::RequestAuthority {
                            chunk,
                            priority,
                            can_wait: *can_wait,
                        },
                    ));
                    *state = ChunkState::WaitingForAuthority;
                    self.last_request_priority.insert(chunk, priority);
                    debug!("Requested authority for {chunk:?}")
                }
                // This state doesn't have much to do.
                ChunkState::WaitingForAuthority => {
                    if should_kill(self.my_pos, self.cam_pos, chunk.0, chunk.1, self.is_notplayer) {
                        *state = ChunkState::UnloadPending;
                    }
                }
                ChunkState::Listening { authority, .. } => {
                    if should_kill(self.my_pos, self.cam_pos, chunk.0, chunk.1, self.is_notplayer) {
                        debug!("Unloading [listening] chunk {chunk:?}");
                        emit_queue.push((
                            Destination::Peer(*authority),
                            WorldNetMessage::ListenStopRequest { chunk },
                        ));
                        *state = ChunkState::UnloadPending;
                    }
                }
                ChunkState::Authority { new_authority, .. } => {
                    if should_kill(self.my_pos, self.cam_pos, chunk.0, chunk.1, self.is_notplayer) {
                        if let Some(new) = new_authority {
                            emit_queue.push((
                                Destination::Peer(new.0),
                                WorldNetMessage::AskForAuthority {
                                    chunk,
                                    priority: new.1,
                                },
                            ));
                        }
                        debug!("Unloading [authority] chunk {chunk:?} (updates: {chunk_last_update} {})", self.current_update);
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
                ChunkState::WantToGetAuth { .. } => {
                    if should_kill(self.my_pos, self.cam_pos, chunk.0, chunk.1, self.is_notplayer) {
                        debug!("Unloading [want to get auth] chunk {chunk:?}");
                        *state = ChunkState::UnloadPending;
                    }
                }
                ChunkState::UnloadPending => {}
                ChunkState::Transfer => {}
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
        // Sends random data to noita to check if it crashes.
        if env::var_os("NP_WORLD_SYNC_TEST").is_some() && self.current_update % 10 == 0 {
            let chunk_data = ChunkData::make_random();
            self.inbound_model
                .apply_chunk_data(ChunkCoord(0, 0), &chunk_data)
        }
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

    fn emit_got_authority(&mut self, chunk: ChunkCoord, source: OmniPeerId, priority: u8) {
        let auth = self.authority_map.get(&chunk).cloned();
        self.authority_map.insert(chunk, (source, priority));
        let chunk_data = if auth.map(|a| a.0 != source).unwrap_or(true) {
            self.chunk_storage.get(&chunk).cloned()
        } else {
            None
        };
        self.emit_msg(
            Destination::Peer(source),
            WorldNetMessage::GotAuthority {
                chunk,
                chunk_data,
                priority,
            },
        );
    }

    fn emit_transfer_authority(
        &mut self,
        chunk: ChunkCoord,
        source: OmniPeerId,
        priority: u8,
        current_authority: OmniPeerId,
    ) {
        self.authority_map.insert(chunk, (source, priority));
        self.emit_msg(
            Destination::Peer(source),
            WorldNetMessage::GetAuthorityFrom {
                chunk,
                current_authority,
            },
        );
    }

    pub(crate) fn handle_msg(&mut self, source: OmniPeerId, msg: WorldNetMessage) {
        match msg {
            WorldNetMessage::RequestAuthority {
                chunk,
                priority,
                can_wait,
            } => {
                if !self.is_host {
                    warn!("{} sent RequestAuthority to not-host.", source);
                    return;
                }
                let current_authority = self.authority_map.get(&chunk).copied();
                match current_authority {
                    Some((authority, priority_state)) => {
                        if source == authority {
                            info!("{source} already has authority of {chunk:?}");
                            self.emit_got_authority(chunk, source, priority);
                        } else if priority_state > priority && !can_wait {
                            info!("{source} is gaining priority over {chunk:?} from {authority}");
                            self.emit_transfer_authority(chunk, source, priority, authority);
                        } else {
                            debug!("{source} requested authority for {chunk:?}, but it's already taken by {authority}");
                            self.emit_msg(
                                Destination::Peer(source),
                                WorldNetMessage::AuthorityAlreadyTaken { chunk, authority },
                            );
                        }
                    }
                    None => {
                        debug!("Granting {source} authority of {chunk:?}");
                        self.emit_got_authority(chunk, source, priority);
                    }
                }
            }
            WorldNetMessage::AskForAuthority { chunk, priority } => {
                self.emit_msg(
                    Destination::Host,
                    WorldNetMessage::RequestAuthority {
                        chunk,
                        priority,
                        can_wait: false,
                    },
                );
                self.chunk_state
                    .insert(chunk, ChunkState::WaitingForAuthority);
            }
            WorldNetMessage::LoseAuthority {
                chunk,
                new_authority,
                new_priority,
            } => {
                if let Some(ChunkState::Authority {
                    new_authority: new_auth,
                    ..
                }) = self.chunk_state.get_mut(&chunk)
                {
                    if new_authority == self.my_peer_id {
                        *new_auth = None;
                    } else if let Some(new) = new_auth {
                        if new.1 > new_priority {
                            *new_auth = Some((new_authority, new_priority));
                        }
                    } else {
                        *new_auth = Some((new_authority, new_priority))
                    }
                }
            }
            WorldNetMessage::ChangePriority { chunk, priority } => {
                if !self.is_host {
                    warn!("{} sent RequestAuthority to not-host.", source);
                    return;
                }
                let current_authority = self.authority_map.get(&chunk).copied();
                match current_authority {
                    Some((authority, _)) => {
                        if source == authority {
                            self.authority_map.insert(chunk, (source, priority));
                        } else {
                            debug!("{source} requested authority for {chunk:?}, but it's already taken by {authority}");
                        }
                    }
                    None => {
                        debug!("Granting {source} authority of {chunk:?}");
                    }
                }
            }
            WorldNetMessage::GotAuthority {
                chunk,
                chunk_data,
                priority,
            } => {
                self.chunk_state
                    .insert(chunk, ChunkState::authority(priority));
                self.last_request_priority.remove(&chunk);
                if let Some(chunk_data) = chunk_data {
                    self.inbound_model.apply_chunk_data(chunk, &chunk_data);
                    self.outbound_model.apply_chunk_data(chunk, &chunk_data);
                }
            }
            WorldNetMessage::UpdateStorage { chunk, chunk_data } => {
                if !self.is_host {
                    warn!("{} sent RelinquishAuthority to not-host.", source);
                    return;
                }
                if let Some(chunk_data) = chunk_data {
                    self.chunk_storage.insert(chunk, chunk_data);
                }
            }
            WorldNetMessage::RelinquishAuthority { chunk, chunk_data } => {
                if !self.is_host {
                    warn!("{} sent RelinquishAuthority to not-host.", source);
                    return;
                }
                if let Some(state) = self.authority_map.get(&chunk) {
                    if state.0 != source {
                        warn!("{source} sent RelinquishAuthority for {chunk:?}, but isn't currently an authority");
                        return;
                    }
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
                self.last_request_priority.remove(&chunk);
            }
            WorldNetMessage::ListenRequest { chunk } => {
                let Some(ChunkState::Authority {
                    listeners,
                    priority,
                    ..
                }) = self.chunk_state.get_mut(&chunk)
                else {
                    //warn!("Can't listen for {chunk:?} - not an authority");
                    return;
                };
                listeners.insert(source);
                let chunk_data = self.outbound_model.get_chunk_data(chunk);
                let priority = *priority;
                self.emit_msg(
                    Destination::Peer(source),
                    WorldNetMessage::ListenInitialResponse {
                        chunk,
                        chunk_data,
                        priority,
                    },
                );
            }
            WorldNetMessage::ListenStopRequest { chunk } => {
                let Some(ChunkState::Authority { listeners, .. }) =
                    self.chunk_state.get_mut(&chunk)
                else {
                    //warn!("Can't stop listen for {chunk:?} - not an authority");
                    return;
                };
                listeners.remove(&source);
            }
            WorldNetMessage::ListenInitialResponse {
                chunk,
                chunk_data,
                priority,
            } => {
                self.chunk_state.insert(
                    chunk,
                    ChunkState::Listening {
                        authority: source,
                        priority,
                    },
                );
                if let Some(chunk_data) = chunk_data {
                    self.inbound_model.apply_chunk_data(chunk, &chunk_data);
                } else {
                    warn!("Initial listen response has None chunk_data. It's generally supposed to have some.");
                }
            }
            WorldNetMessage::ListenUpdate {
                delta,
                priority,
                take_auth,
            } => {
                match self.chunk_state.get_mut(&delta.chunk_coord) {
                    Some(ChunkState::Listening { priority: pri, .. }) => {
                        *pri = priority;
                        if take_auth {
                            self.emit_msg(
                                Destination::Peer(source),
                                WorldNetMessage::LoseAuthority {
                                    chunk: delta.chunk_coord,
                                    new_priority: priority,
                                    new_authority: source,
                                },
                            );
                        }
                    }
                    Some(ChunkState::WantToGetAuth {
                        authority,
                        my_priority,
                        ..
                    }) => {
                        if priority > *my_priority {
                            if take_auth {
                                let rq = WorldNetMessage::RequestAuthority {
                                    chunk: delta.chunk_coord,
                                    priority: *my_priority,
                                    can_wait: false,
                                };
                                self.emit_msg(Destination::Host, rq);
                                self.chunk_state
                                    .insert(delta.chunk_coord, ChunkState::WaitingForAuthority);
                            }
                        } else {
                            let cs = ChunkState::Listening {
                                authority: *authority,
                                priority,
                            };
                            self.chunk_state.insert(delta.chunk_coord, cs);
                        }
                    }
                    _ if take_auth => {
                        self.emit_msg(
                            Destination::Peer(source),
                            WorldNetMessage::LoseAuthority {
                                chunk: delta.chunk_coord,
                                new_priority: priority,
                                new_authority: source,
                            },
                        );
                    }
                    _ => return,
                }
                self.inbound_model.apply_chunk_delta(&delta);
            }
            WorldNetMessage::ListenAuthorityRelinquished { chunk } => {
                self.chunk_state.insert(chunk, ChunkState::UnloadPending);
            }
            WorldNetMessage::GetAuthorityFrom {
                chunk,
                current_authority,
            } => {
                if self.chunk_state.get(&chunk) != Some(&ChunkState::UnloadPending) {
                    info!("Will request authority transfer");
                    self.chunk_state.insert(chunk, ChunkState::Transfer);
                    self.emit_msg(
                        Destination::Peer(current_authority),
                        WorldNetMessage::RequestAuthorityTransfer { chunk },
                    );
                } else {
                    self.emit_msg(
                        Destination::Host,
                        WorldNetMessage::RelinquishAuthority { chunk, chunk_data: None },
                    );
                }
            }
            WorldNetMessage::RequestAuthorityTransfer { chunk } => {
                info!("Got a request for authority transfer");
                let state = self.chunk_state.get(&chunk);
                if let Some(ChunkState::Authority {
                    listeners,
                    ..
                }) = state
                {
                    let chunk_data = self.outbound_model.get_chunk_data(chunk);
                    self.emit_msg(
                        Destination::Peer(source),
                        WorldNetMessage::TransferOk {
                            chunk,
                            chunk_data,
                            listeners: listeners.clone(),
                        },
                    );
                    self.chunk_state.insert(chunk, ChunkState::UnloadPending);
                    let chunk_data = self.outbound_model.get_chunk_data(chunk);
                    self.emit_msg(
                        Destination::Host,
                        WorldNetMessage::UpdateStorage { chunk, chunk_data },
                    );
                } else {
                    self.emit_msg(
                        Destination::Peer(source),
                        WorldNetMessage::TransferFailed { chunk },
                    );
                }
            }
            WorldNetMessage::TransferOk {
                chunk,
                chunk_data,
                listeners,
            } => {
                info!("Transfer ok");
                if let Some(chunk_data) = chunk_data {
                    self.inbound_model.apply_chunk_data(chunk, &chunk_data);
                    self.outbound_model.apply_chunk_data(chunk, &chunk_data);
                }
                for listener in listeners.iter() {
                    self.emit_msg(
                        Destination::Peer(*listener),
                        WorldNetMessage::NotifyNewAuthority { chunk },
                    );
                }
                self.chunk_state.insert(
                    chunk,
                    ChunkState::Authority {
                        listeners,
                        priority: self.last_request_priority.remove(&chunk).unwrap_or(0),
                        new_authority: None,
                        stop_sending: false,
                    },
                );
            }
            WorldNetMessage::TransferFailed { chunk } => {
                warn!("Transfer failed, requesting authority normally");
                let priority = self
                    .last_request_priority
                    .get(&chunk)
                    .copied()
                    .unwrap_or(255);
                self.chunk_state.insert(
                    chunk,
                    ChunkState::RequestAuthority {
                        priority,
                        can_wait: true,
                    },
                );
            }
            WorldNetMessage::NotifyNewAuthority { chunk } => {
                info!("Notified of new authority");
                let state = self.chunk_state.get_mut(&chunk);
                if let Some(ChunkState::Listening { authority, .. }) = state {
                    *authority = source;
                } else {
                    warn!("Got notified of new authority, but not a listener");
                }
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
            if peer.0 == source {
                info!("Removing authority from disconnected peer: {chunk:?}");
                pending_messages.push(WorldNetMessage::ListenAuthorityRelinquished { chunk });
            }
        }
        self.authority_map.retain(|_, peer| peer.0 != source);

        for message in pending_messages {
            self.emit_msg(Destination::Broadcast, message)
        }
    }

    pub(crate) fn get_debug_markers(&self) -> Vec<DebugMarker> {
        self.chunk_state
            .iter()
            .map(|(&chunk, state)| {
                let message = match state {
                    ChunkState::RequestAuthority { .. } => "req auth",
                    ChunkState::WaitingForAuthority => "wai auth",
                    ChunkState::Listening { .. } => "list",
                    ChunkState::Authority { .. } => "auth",
                    ChunkState::UnloadPending => "unl",
                    ChunkState::Transfer => "tran",
                    ChunkState::WantToGetAuth { .. } => "want auth",
                };
                let mut priority = String::new();
                if let Some(n) = self.authority_map.get(&chunk).copied() {
                    priority = n.1.to_string()
                }
                DebugMarker {
                    x: (chunk.0 * 128) as f64,
                    y: (chunk.1 * 128) as f64,
                    message: message.to_owned() + &priority,
                }
            })
            .collect()
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