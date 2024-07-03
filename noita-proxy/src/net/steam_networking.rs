use std::{fmt::Display, sync::Mutex};

use crossbeam::channel;
use fluent_bundle::FluentValue;
use steamworks::{
    networking_types::{NetworkingIdentity, SendFlags},
    CallbackHandle, LobbyChatUpdate, LobbyId, LobbyType, SteamError, SteamId,
};
use tangled::{PeerState, Reliability};
use tracing::{info, warn};

use crate::{
    lang::{tr, tr_a},
    releases::Version,
};

use super::omni::{self, OmniNetworkEvent};

#[derive(Clone, Copy, Debug)]
pub enum ConnectError {
    VersionMismatch { remote_version: Version },
    VersionMissing,
    LobbyDoesNotExist,
}

impl Display for ConnectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectError::VersionMismatch { remote_version } => {
                let translated = tr_a(
                    "error_version_mismatch",
                    &[
                        (
                            "remote_version",
                            FluentValue::from(remote_version.to_string()),
                        ),
                        (
                            "current_version",
                            FluentValue::from(Version::current().to_string()),
                        ),
                    ],
                );
                write!(f, "{}", translated)
            }
            ConnectError::VersionMissing => write!(f, "{}", tr("error_missing_version_field")),
            ConnectError::LobbyDoesNotExist => write!(f, "{}", tr("error_lobby_does_not_exist")),
        }
    }
}

enum SteamEvent {
    LobbyCreated(LobbyId),
    LobbyError(SteamError),
    LobbyJoinError(ConnectError),
    PeerConnected(SteamId),
    PeerDisconnected(SteamId),
    PeerStateChanged,
}

#[derive(Debug, Clone, Copy)]
pub enum ExtraPeerState {
    Tangled(PeerState),
    CouldNotConnect(ConnectError),
}

pub struct InnerState {
    lobby_id: Option<LobbyId>,
    host_id: SteamId,
    remote_peers: Vec<SteamId>,
    state: ExtraPeerState,
}

pub struct SteamPeer {
    client: steamworks::Client,
    events: channel::Receiver<SteamEvent>,
    sender: channel::Sender<SteamEvent>,
    my_id: SteamId,
    is_host: bool,
    inner: Mutex<InnerState>,
    _cbs: Vec<CallbackHandle>,
}

impl SteamPeer {
    pub fn new_host(lobby_type: LobbyType, client: steamworks::Client) -> Self {
        let (sender, events) = channel::unbounded();
        let matchmaking = client.matchmaking();
        {
            let sender = sender.clone();
            matchmaking.create_lobby(lobby_type, 100, {
                let client = client.clone();
                move |lobby| {
                    let matchmaking = client.matchmaking();
                    let event = match lobby {
                        Ok(id) => {
                            matchmaking.set_lobby_data(
                                id,
                                "ew_version",
                                &Version::current().to_string(),
                            );
                            SteamEvent::LobbyCreated(id)
                        }
                        Err(err) => SteamEvent::LobbyError(err),
                    };
                    sender.send(event).ok();
                }
            });
        }

        let my_id = client.user().steam_id();
        let _cbs = make_callbacks(&sender, &client);
        Self {
            my_id,
            client,

            events,
            sender,
            is_host: true,
            inner: Mutex::new(InnerState {
                lobby_id: None,
                host_id: my_id,
                remote_peers: Vec::new(),
                state: ExtraPeerState::Tangled(PeerState::PendingConnection),
            }),
            _cbs,
        }
    }

    pub fn new_connect(lobby: LobbyId, client: steamworks::Client) -> Self {
        let (sender, events) = channel::unbounded();
        let matchmaking = client.matchmaking();
        {
            let sender = sender.clone();

            matchmaking.join_lobby(lobby, {
                let client = client.clone();
                move |lobby| {
                    let matchmaking = client.matchmaking();

                    let event = match lobby {
                        Ok(id) => {
                            match matchmaking
                                .lobby_data(id, "ew_version")
                                .and_then(Version::parse_from_diplay)
                            {
                                Some(remote_version) => {
                                    if remote_version != Version::current() {
                                        warn!(
                                            "Could not connect: version mismatch, remote: {}, current: {}",
                                            remote_version,
                                            Version::current()
                                        );
                                        SteamEvent::LobbyJoinError(ConnectError::VersionMismatch { remote_version })
                                    } else {
                                        SteamEvent::LobbyCreated(id)
                                    }
                                }
                                None => {
                                    warn!("Could not connect: version data missing/could not be parsed");
                                    SteamEvent::LobbyJoinError(ConnectError::VersionMissing)
                                }
                            }
                        }
                        Err(_) => SteamEvent::LobbyJoinError(ConnectError::LobbyDoesNotExist),
                    };

                    sender.send(event).ok();
                }
            });
        }

        let my_id = client.user().steam_id();
        let _cbs = make_callbacks(&sender, &client);
        Self {
            my_id,
            client,

            events,
            sender,
            is_host: false,
            inner: Mutex::new(InnerState {
                lobby_id: None,
                remote_peers: Vec::new(),
                host_id: my_id,
                state: ExtraPeerState::Tangled(PeerState::PendingConnection),
            }),
            _cbs,
        }
    }

    pub fn send_message(&self, peer: SteamId, msg: &[u8], reliability: Reliability) -> bool {
        let send_type = if reliability == Reliability::Reliable {
            SendFlags::RELIABLE
        } else {
            SendFlags::UNRELIABLE
        };
        let networking = self.client.networking_messages();
        let res = networking
            .send_message_to_user(NetworkingIdentity::new_steam_id(peer), send_type, msg, 0)
            .inspect_err(|err| {
                warn!(
                    "Couldn't send a packet to {:?}, st {:?}, err {}",
                    peer, send_type, err
                )
            });
        res.is_ok()
    }

    pub fn broadcast_message(&self, msg: &[u8], reliability: Reliability) {
        let peers = self.inner.lock().unwrap().remote_peers.clone();
        for peer in peers {
            self.send_message(peer, msg, reliability);
        }
    }

    pub fn recv(&self) -> Vec<OmniNetworkEvent> {
        let mut returned_events = Vec::new();
        for event in self.events.try_iter() {
            match event {
                SteamEvent::LobbyCreated(id) => {
                    info!("Lobby ready");
                    self.inner.lock().unwrap().lobby_id = Some(id);
                    if !self.is_host {
                        let host_id = self.client.matchmaking().lobby_owner(id);
                        self.inner.lock().unwrap().host_id = host_id;
                        info!("Got host id: {:?}", host_id)
                    }
                    self.update_remote_peers();
                    self.inner.lock().unwrap().state =
                        ExtraPeerState::Tangled(PeerState::Connected);
                }
                SteamEvent::LobbyError(err) => {
                    warn!("Could not create lobby: {}", err);
                    self.inner.lock().unwrap().state =
                        ExtraPeerState::Tangled(PeerState::Disconnected);
                }
                SteamEvent::LobbyJoinError(err) => {
                    warn!("Could not join lobby");
                    self.inner.lock().unwrap().state = ExtraPeerState::CouldNotConnect(err);
                }
                SteamEvent::PeerConnected(id) => {
                    returned_events.push(omni::OmniNetworkEvent::PeerConnected(id.into()))
                }
                SteamEvent::PeerDisconnected(id) => {
                    returned_events.push(omni::OmniNetworkEvent::PeerDisconnected(id.into()))
                }
                SteamEvent::PeerStateChanged => self.update_remote_peers(),
            }
        }
        let networking = self.client.networking_messages();
        let messages = networking.receive_messages_on_channel(0, 1024);
        for message in messages {
            let steam_id = message
                .identity_peer()
                .steam_id()
                .expect("only steam ids are supported");
            returned_events.push(omni::OmniNetworkEvent::Message {
                src: steam_id.into(),
                data: message.data().to_vec(), // TODO eliminate clone here.
            })
        }
        returned_events
    }

    fn update_remote_peers(&self) {
        let matchmaking = self.client.matchmaking();
        let lobby = self.inner.lock().unwrap().lobby_id;
        let mut peers = match lobby {
            Some(lobby_id) => matchmaking.lobby_members(lobby_id),
            None => Vec::new(),
        };
        peers.retain(|x| *x != self.my_id);
        peers.sort();
        let current_peers = &mut self.inner.lock().unwrap().remote_peers;

        // TODO: could be done more efficiently
        for peer in &peers {
            if !current_peers.contains(peer) {
                self.sender.send(SteamEvent::PeerConnected(*peer)).ok();
            }
        }
        for peer in &mut *current_peers {
            if !peers.contains(peer) {
                self.sender.send(SteamEvent::PeerDisconnected(*peer)).ok();
            }
        }

        *current_peers = peers;
    }

    pub fn get_peer_ids(&self) -> Vec<SteamId> {
        // let matchmaking = self.client.matchmaking();
        // let lobby = self.inner.lock().unwrap().lobby_id;
        // match lobby {
        //     Some(lobby_id) => matchmaking.lobby_members(lobby_id),
        //     None => Vec::new(),
        // }
        let mut peers = self.inner.lock().unwrap().remote_peers.clone();
        peers.push(self.my_id);
        peers
    }

    pub fn my_id(&self) -> SteamId {
        self.my_id
    }

    pub fn host_id(&self) -> SteamId {
        if self.is_host {
            self.my_id
        } else {
            self.inner.lock().unwrap().host_id
        }
    }

    pub fn lobby_id(&self) -> Option<LobbyId> {
        self.inner.lock().unwrap().lobby_id
    }

    pub fn state(&self) -> ExtraPeerState {
        self.inner.lock().unwrap().state
    }
}

fn make_callbacks(
    sender: &channel::Sender<SteamEvent>,
    client: &steamworks::Client,
) -> Vec<CallbackHandle> {
    {
        client
            .networking_messages()
            .session_request_callback(|req| {
                info!("Accepting connection with {:?}", req.remote());
                req.accept();
            });
    };
    let cb_ch = {
        let sender = sender.clone();
        client.register_callback(move |update: LobbyChatUpdate| {
            info!("User state changed {:?}", update);
            sender.send(SteamEvent::PeerStateChanged).ok();
        })
    };
    vec![cb_ch]
}
