use std::{fmt::Display, mem, sync::Mutex};

use crossbeam::channel;
use dashmap::DashMap;
use fluent_bundle::FluentValue;
use steamworks::{
    networking_sockets::{ListenSocket, NetPollGroup},
    networking_types::{
        ListenSocketEvent, NetworkingConnectionState, NetworkingIdentity, SendFlags,
    },
    CallbackHandle, ClientManager, LobbyChatUpdate, LobbyId, LobbyType, SteamError, SteamId,
};
use tangled::{PeerState, Reliability};
use tracing::{info, warn};

use crate::{
    lang::{tr, tr_a},
    releases::Version,
};

use super::omni::OmniNetworkEvent;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    LobbyCreatedOrJoined(LobbyId),
    LobbyError(SteamError),
    LobbyJoinError(ConnectError),
    PeerConnectedToLobby(SteamId),
    PeerDisconnectedFromLobby(SteamId),
    PeerStateChanged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtraPeerState {
    Tangled(PeerState),
    CreatingMesh,
    CouldNotConnect(ConnectError),
}

pub struct InnerState {
    lobby_id: Option<LobbyId>,
    host_id: SteamId,
    remote_peers: Vec<SteamId>,
    state: ExtraPeerState,
}

enum ConnectionState {
    AwaitingIncoming,
    NetConnectionPending(steamworks::networking_sockets::NetConnection<ClientManager>),
    NetConnection(steamworks::networking_sockets::NetConnection<ClientManager>),
}
impl ConnectionState {
    fn switch_to_connected(&mut self) {
        let current = mem::replace(self, ConnectionState::AwaitingIncoming);
        if let ConnectionState::NetConnectionPending(conn) = current {
            *self = ConnectionState::NetConnection(conn);
        }
    }
    fn connection(&self) -> Option<&steamworks::networking_sockets::NetConnection<ClientManager>> {
        if let ConnectionState::NetConnection(conn) = self {
            Some(conn)
        } else {
            None
        }
    }
}

struct Connections {
    client: steamworks::Client,

    my_id: SteamId,
    listen_socket: ListenSocket<ClientManager>,
    poll_group: Mutex<NetPollGroup<ClientManager>>,

    peers: DashMap<SteamId, ConnectionState>,
    connected: Mutex<Vec<SteamId>>,
}

impl Connections {
    fn new(client: &steamworks::Client) -> Self {
        let my_id = client.user().steam_id();
        let networking_sockets = client.networking_sockets();
        let listen_socket = networking_sockets
            .create_listen_socket_p2p(0, None)
            .expect("handle to be valid"); // Unclear in what cases this can fail.
        let poll_group = networking_sockets.create_poll_group().into();
        Connections {
            client: client.clone(),

            my_id,
            listen_socket,
            poll_group,

            peers: Default::default(),
            connected: Default::default(),
        }
    }

    fn poll_listener(&self) {
        while let Some(event) = self.listen_socket.try_receive_event() {
            match event {
                ListenSocketEvent::Connecting(event) => {
                    info!("Peer {:?} connecting", event.remote());
                    event
                        .accept()
                        .inspect_err(|e| warn!("Error when accepting connection: {}", e))
                        .ok();
                }
                ListenSocketEvent::Connected(event) => {
                    if let Some(mut connection) = event
                        .remote()
                        .steam_id()
                        .and_then(|x| self.peers.get_mut(&x))
                    {
                        info!("Peer {:?} got connected event", event.remote());
                        let taken_connection = event.take_connection();
                        taken_connection.set_poll_group(&self.poll_group.lock().unwrap());
                        *connection = ConnectionState::NetConnectionPending(taken_connection);
                    }
                }
                ListenSocketEvent::Disconnected(event) => {
                    info!(
                        "Peer {:?} disconnected, reason: {:?}",
                        event.remote(),
                        event.end_reason(),
                    )
                }
            }
        }
    }
    fn poll_status(&self) -> bool {
        let networking_sockets = self.client.networking_sockets();
        let mut all_connected = true;
        for mut state in self.peers.iter_mut() {
            match state.value() {
                ConnectionState::AwaitingIncoming => {
                    all_connected = false;
                }
                ConnectionState::NetConnectionPending(connection) => {
                    let info = networking_sockets
                        .get_connection_info(connection)
                        .expect("handle to be valid");
                    match info.state().expect("assuming state is always valid") {
                        // Wait.
                        NetworkingConnectionState::None
                        | NetworkingConnectionState::Connecting
                        | NetworkingConnectionState::FindingRoute => {
                            all_connected = false;
                        }
                        // Switch to connected state.
                        NetworkingConnectionState::Connected => {
                            info!(
                                "Connection of peer {:?} swithed to connected state",
                                *state.key()
                            );
                            self.connected.lock().unwrap().push(*state.key());
                            state.value_mut().switch_to_connected();
                            all_connected = false;
                        }

                        NetworkingConnectionState::ClosedByPeer
                        | NetworkingConnectionState::ProblemDetectedLocally => {
                            info!(
                                "Some problem happened for peer {:?}. Will try to connect again.",
                                *state.key()
                            );
                            self.connect(*state.key());
                            all_connected = false;
                        }
                    }
                }
                ConnectionState::NetConnection(_) => {}
            }
        }
        all_connected
    }

    /// Returns true if everyone is connected.
    fn poll(&self) -> bool {
        self.poll_listener();
        self.poll_status()
    }

    fn connect(&self, peer: SteamId) {
        let networking_sockets = self.client.networking_sockets();
        let peer_identity = NetworkingIdentity::new_steam_id(peer);
        if peer > self.my_id {
            info!("Awaiting incoming connection from {:?}", peer);
            self.peers.insert(peer, ConnectionState::AwaitingIncoming);
        } else {
            info!("Initiating connection to {:?}", peer);
            let connection = networking_sockets
                .connect_p2p(peer_identity, 0, None)
                .expect("handle to be valid");
            connection.set_poll_group(&self.poll_group.lock().unwrap());
            self.peers
                .insert(peer, ConnectionState::NetConnectionPending(connection));
        }
    }
    fn disconnect(&self, id: SteamId) {
        info!("Removing connection to peer {:?}", id);
        self.peers.remove(&id);
    }

    fn recv(&self) -> Vec<steamworks::networking_types::NetworkingMessage<ClientManager>> {
        self.poll_group.lock().unwrap().receive_messages(1024)
    }

    fn send_message(
        &self,
        peer: SteamId,
        send_flags: SendFlags,
        msg: &[u8],
    ) -> Result<(), SteamError> {
        if let Some(peer) = self.peers.get(&peer) {
            if let Some(connection) = peer.value().connection() {
                connection.send_message(msg, send_flags)?;
                Ok(())
            } else {
                // TODO: Not exactly the right thing to do, but this can only happen before we properly connected, so it's probably fine..?
                // Might result in a packet loss in case we ever go reconnecting.
                Ok(())
            }
        } else {
            Err(SteamError::InvalidSteamID)
        }
    }
}

pub struct SteamPeer {
    client: steamworks::Client,
    events: channel::Receiver<SteamEvent>,
    sender: channel::Sender<SteamEvent>,
    my_id: SteamId,
    is_host: bool,
    inner: Mutex<InnerState>,
    _cbs: Vec<CallbackHandle>,

    connections: Connections,
}

impl SteamPeer {
    pub fn new_host(lobby_type: LobbyType, client: steamworks::Client) -> Self {
        let (sender, events) = channel::unbounded();

        let connections = Connections::new(&client);

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
                            SteamEvent::LobbyCreatedOrJoined(id)
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
            connections,
        }
    }

    pub fn new_connect(lobby: LobbyId, client: steamworks::Client) -> Self {
        let (sender, events) = channel::unbounded();
        let connections = Connections::new(&client);
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
                                        SteamEvent::LobbyCreatedOrJoined(id)
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
            connections,
        }
    }

    pub fn send_message(&self, peer: SteamId, msg: &[u8], reliability: Reliability) -> bool {
        let send_type = if reliability == Reliability::Reliable {
            SendFlags::RELIABLE
        } else {
            SendFlags::UNRELIABLE
        };

        let res = self
            .connections
            .send_message(peer, send_type, msg)
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
        let all_connected = self.connections.poll();
        if all_connected && self.inner.lock().unwrap().state == ExtraPeerState::CreatingMesh {
            info!("Switched to `all connected` state");
            self.inner.lock().unwrap().state = ExtraPeerState::Tangled(PeerState::Connected);
        }
        let mut returned_events = Vec::new();
        for event in self.events.try_iter() {
            match event {
                SteamEvent::LobbyCreatedOrJoined(id) => {
                    info!("Lobby ready");
                    self.inner.lock().unwrap().lobby_id = Some(id);
                    if !self.is_host {
                        let host_id = self.client.matchmaking().lobby_owner(id);
                        self.inner.lock().unwrap().host_id = host_id;
                        info!("Got host id: {:?}", host_id)
                    }
                    self.update_lobby_list();
                    info!("Switched to `creating mesh` state");
                    self.inner.lock().unwrap().state = ExtraPeerState::CreatingMesh;
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
                SteamEvent::PeerConnectedToLobby(id) => {
                    self.connections.connect(id);
                }
                SteamEvent::PeerDisconnectedFromLobby(id) => {
                    self.connections.disconnect(id);
                    returned_events.push(OmniNetworkEvent::PeerDisconnected(id.into()))
                }
                SteamEvent::PeerStateChanged => self.update_lobby_list(),
            }
        }

        let messages = self.connections.recv();
        for message in messages {
            let steam_id = message
                .identity_peer()
                .steam_id()
                .expect("only steam ids are supported");
            returned_events.push(OmniNetworkEvent::Message {
                src: steam_id.into(),
                data: message.data().to_vec(), // TODO eliminate clone here.
            })
        }
        let mut fully_connected = self.connections.connected.lock().unwrap();
        for steam_id in fully_connected.iter() {
            returned_events.push(OmniNetworkEvent::PeerConnected((*steam_id).into()))
        }
        fully_connected.clear();

        returned_events
    }

    fn update_lobby_list(&self) {
        info!("Updating peer list");
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
                self.sender
                    .send(SteamEvent::PeerConnectedToLobby(*peer))
                    .ok();
            }
        }
        for peer in &mut *current_peers {
            if !peers.contains(peer) {
                self.sender
                    .send(SteamEvent::PeerDisconnectedFromLobby(*peer))
                    .ok();
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

    pub fn is_host(&self) -> bool {
        self.is_host
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
