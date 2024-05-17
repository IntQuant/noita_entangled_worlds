use std::{
    fmt::Display,
    io::{self, Write},
    net::{TcpListener, TcpStream},
    sync::{atomic::AtomicBool, Arc, Mutex},
    thread,
    time::Duration,
};
use steamworks::{LobbyId, SteamId};
use tracing::debug;

use tangled::{PeerId, PeerState, Reliability};
use tracing::{error, info, warn};
use tungstenite::{accept, WebSocket};

use crate::{messages::NetMsg, GameSettings};

pub mod steam_networking;

pub(crate) fn ws_encode_proxy(key: &'static str, value: impl Display) -> tungstenite::Message {
    let mut buf = Vec::new();
    buf.push(2);
    write!(buf, "{} {}", key, value).unwrap();
    tungstenite::Message::Binary(buf)
}

pub(crate) fn ws_encode_mod(peer: OmniPeerId, data: &[u8]) -> tungstenite::Message {
    let mut buf = Vec::new();
    buf.push(1u8);
    buf.extend_from_slice(&peer.0.to_le_bytes());
    buf.extend_from_slice(data);
    tungstenite::Message::Binary(buf)
}

pub(crate) struct NetInnerState {
    pub(crate) ws: Option<WebSocket<TcpStream>>,
}

impl NetInnerState {
    pub(crate) fn try_ws_write(&mut self, data: tungstenite::Message) {
        if let Some(ws) = &mut self.ws {
            if let Err(err) = ws.write(data) {
                error!("Error occured while sending to websocket: {}", err);
                self.ws = None;
            };
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OmniPeerId(u64);

impl From<PeerId> for OmniPeerId {
    fn from(value: PeerId) -> Self {
        Self(value.0.into())
    }
}

impl From<SteamId> for OmniPeerId {
    fn from(value: SteamId) -> Self {
        Self(value.raw())
    }
}

impl From<OmniPeerId> for PeerId {
    fn from(value: OmniPeerId) -> Self {
        Self(
            value
                .0
                .try_into()
                .expect("Assuming PeerId was stored here, so conversion should succeed"),
        )
    }
}

impl From<OmniPeerId> for SteamId {
    fn from(value: OmniPeerId) -> Self {
        Self::from_raw(value.0)
    }
}

impl Display for OmniPeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub enum OmniNetworkEvent {
    PeerConnected(OmniPeerId),
    PeerDisconnected(OmniPeerId),
    Message { src: OmniPeerId, data: Vec<u8> },
}

impl From<tangled::NetworkEvent> for OmniNetworkEvent {
    fn from(value: tangled::NetworkEvent) -> Self {
        match value {
            tangled::NetworkEvent::PeerConnected(id) => Self::PeerConnected(id.into()),
            tangled::NetworkEvent::PeerDisconnected(id) => Self::PeerDisconnected(id.into()),
            tangled::NetworkEvent::Message(msg) => Self::Message {
                src: msg.src.into(),
                data: msg.data,
            },
        }
    }
}

pub enum PeerVariant {
    Tangled(tangled::Peer),
    Steam(steam_networking::SteamPeer),
}

impl PeerVariant {
    fn send(
        &self,
        peer: OmniPeerId,
        msg: Vec<u8>,
        reliability: Reliability,
    ) -> Result<(), tangled::NetError> {
        match self {
            PeerVariant::Tangled(p) => p.send(peer.into(), msg, reliability),
            PeerVariant::Steam(p) => {
                p.send_message(peer.into(), &msg, reliability);
                Ok(())
            }
        }
    }

    fn broadcast(&self, msg: Vec<u8>, reliability: Reliability) -> Result<(), tangled::NetError> {
        match self {
            PeerVariant::Tangled(p) => p.broadcast(msg, reliability),
            PeerVariant::Steam(p) => Ok(p.broadcast_message(&msg, reliability)),
        }
    }

    fn my_id(&self) -> Option<OmniPeerId> {
        match self {
            PeerVariant::Tangled(p) => p.my_id().map(OmniPeerId::from),
            PeerVariant::Steam(p) => Some(p.my_id().into()),
        }
    }

    pub fn iter_peer_ids(&self) -> Vec<OmniPeerId> {
        match self {
            PeerVariant::Tangled(p) => p.iter_peer_ids().map(OmniPeerId::from).collect(),
            PeerVariant::Steam(p) => p.get_peer_ids().into_iter().map(OmniPeerId::from).collect(),
        }
    }

    fn recv(&self) -> Vec<OmniNetworkEvent> {
        match self {
            PeerVariant::Tangled(p) => p.recv().map(OmniNetworkEvent::from).collect(),
            PeerVariant::Steam(p) => p.recv(),
        }
    }

    pub fn state(&self) -> PeerState {
        match self {
            PeerVariant::Tangled(p) => p.state(),
            PeerVariant::Steam(p) => p.state(),
        }
    }

    pub fn host_id(&self) -> OmniPeerId {
        match self {
            PeerVariant::Tangled(_) => PeerId::HOST.into(),
            PeerVariant::Steam(p) => p.host_id().into(),
        }
    }

    pub fn lobby_id(&self) -> Option<LobbyId> {
        match self {
            PeerVariant::Tangled(_) => None,
            PeerVariant::Steam(p) => p.lobby_id(),
        }
    }
}

pub struct NetManager {
    pub(crate) peer: PeerVariant,
    pub(crate) settings: Mutex<GameSettings>,
    pub(crate) continue_running: AtomicBool, // TODO stop on drop
    pub(crate) accept_local: AtomicBool,
    pub(crate) local_connected: AtomicBool,
}

impl NetManager {
    pub fn new(peer: PeerVariant) -> Arc<Self> {
        Self {
            peer,
            settings: Mutex::new(GameSettings {
                // seed: 1663107061,
                seed: 1663107066,
                debug_mode: false,
            }),
            continue_running: AtomicBool::new(true),
            accept_local: AtomicBool::new(false),
            local_connected: AtomicBool::new(false),
        }
        .into()
    }

    pub(crate) fn send(&self, peer: OmniPeerId, msg: &NetMsg, reliability: Reliability) {
        let encoded = bitcode::encode(msg);
        self.peer.send(peer, encoded.clone(), reliability).ok(); // TODO log
    }

    pub(crate) fn broadcast(&self, msg: &NetMsg, reliability: Reliability) {
        let encoded = bitcode::encode(msg);
        let len = encoded.len();
        if let Err(err) = self.peer.broadcast(encoded, reliability) {
            warn!("Error while broadcasting message of len {}: {}", len, err)
        }
    }

    pub(crate) fn start_inner(self: Arc<NetManager>) {
        let local_server = TcpListener::bind("127.0.0.1:21251").unwrap();
        // let local_server = TcpListener::bind("127.0.0.1:0").unwrap();
        local_server
            .set_nonblocking(true)
            .expect("can set nonblocking");

        let mut state = NetInnerState { ws: None };

        while self
            .continue_running
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            self.local_connected
                .store(state.ws.is_some(), std::sync::atomic::Ordering::Relaxed);
            if state.ws.is_none() && self.accept_local.load(std::sync::atomic::Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(10));
                if let Ok((stream, addr)) = local_server.accept() {
                    info!("New stream incoming from {}", addr);
                    stream.set_nodelay(true).ok();
                    stream
                        .set_read_timeout(Some(Duration::from_millis(1)))
                        .expect("can set read timeout");

                    state.ws = accept(stream).ok();
                    if state.ws.is_some() {
                        info!("New stream connected");

                        let settings = self.settings.lock().unwrap();
                        state.try_ws_write(ws_encode_proxy("seed", settings.seed));
                        state.try_ws_write(ws_encode_proxy(
                            "peer_id",
                            self.peer.my_id().expect("Has peer id at this point"),
                        ));
                        state.try_ws_write(ws_encode_proxy("host_id", self.peer.host_id()));
                        state.try_ws_write(ws_encode_proxy("name", "test_name"));
                        state.try_ws_write(ws_encode_proxy(
                            "debug",
                            if settings.debug_mode { "true" } else { "false" },
                        ));
                        state.try_ws_write(ws_encode_proxy("ready", ""));
                        // TODO? those are currently ignored by mod
                        for id in self.peer.iter_peer_ids() {
                            state.try_ws_write(ws_encode_proxy("join", id));
                        }
                    }
                }
            }
            if let Some(ws) = &mut state.ws {
                ws.flush().ok();
            }
            for net_event in self.peer.recv() {
                match net_event {
                    OmniNetworkEvent::PeerConnected(id) => {
                        info!("Peer connected");
                        if self.peer.my_id() == Some(self.peer.host_id()) {
                            info!("Sending start game message");
                            self.send(
                                id,
                                &NetMsg::StartGame {
                                    settings: self.settings.lock().unwrap().clone(),
                                },
                                tangled::Reliability::Reliable,
                            );
                        }
                        state.try_ws_write(ws_encode_proxy("join", id));
                    }
                    OmniNetworkEvent::PeerDisconnected(id) => {
                        state.try_ws_write(ws_encode_proxy("leave", id));
                    }
                    OmniNetworkEvent::Message { src, data } => {
                        let Ok(net_msg) = bitcode::decode::<NetMsg>(&data) else {
                            continue;
                        };
                        match net_msg {
                            NetMsg::StartGame { settings } => {
                                *self.settings.lock().unwrap() = settings;
                                info!("Settings updated");
                                self.accept_local
                                    .store(true, std::sync::atomic::Ordering::SeqCst);
                            }
                            NetMsg::ModRaw { data } => {
                                state.try_ws_write(ws_encode_mod(src, &data));
                            }
                            NetMsg::ModCompressed { data } => {
                                if let Ok(decompressed) = lz4_flex::decompress_size_prepended(&data)
                                {
                                    state.try_ws_write(ws_encode_mod(src, &decompressed));
                                }
                            }
                        }
                    }
                }
            }
            if let Some(ws) = &mut state.ws {
                let msg = ws.read();
                self.handle_mod_message(msg, &mut state);
            }
        }
    }

    pub(crate) fn handle_mod_message(
        &self,
        msg: Result<tungstenite::Message, tungstenite::Error>,
        state: &mut NetInnerState,
    ) {
        match msg {
            Ok(msg) => match msg {
                tungstenite::Message::Binary(msg) => {
                    match msg[0] & 0b11 {
                        // Message to proxy
                        1 => {
                            self.handle_message_to_proxy(&msg[1..]);
                        }
                        // Broadcast
                        2 => {
                            // Somewhat arbitrary limit to begin compressing messages.
                            // Messages shorter than this many bytes probably won't be compressed as much
                            let msg_to_send = if msg.len() > 140 {
                                let compressed = lz4_flex::compress_prepend_size(&msg[1..]);

                                debug!(
                                    "Compressed {} bytes to {} bytes",
                                    msg.len(),
                                    compressed.len()
                                );

                                NetMsg::ModCompressed { data: compressed }
                            } else {
                                NetMsg::ModRaw {
                                    data: msg[1..].to_owned(),
                                }
                            };
                            let reliable = msg[0] & 4 > 0;
                            self.broadcast(
                                &msg_to_send,
                                if reliable {
                                    Reliability::Reliable
                                } else {
                                    Reliability::Unreliable
                                },
                            );
                        }
                        msg_variant => {
                            error!("Unknown msg variant from mod: {}", msg_variant)
                        }
                    }
                }
                _ => {}
            },
            Err(tungstenite::Error::Io(io_err)) if io_err.kind() == io::ErrorKind::WouldBlock => {}
            Err(err) => {
                error!("Error occured while reading from websocket: {}", err);
                state.ws = None;
            }
        }
    }

    pub fn start(self: Arc<NetManager>) {
        info!("Starting netmanager");
        thread::spawn(move || self.start_inner());
    }

    fn resend_game_settings(&self) {
        let settings = self.settings.lock().unwrap().clone();
        self.broadcast(&NetMsg::StartGame { settings }, Reliability::Reliable);
    }

    fn is_host(&self) -> bool {
        self.peer.my_id() == Some(self.peer.host_id())
    }

    pub(crate) fn handle_message_to_proxy(&self, msg: &[u8]) {
        let msg = String::from_utf8_lossy(msg);
        let mut msg = msg.split_ascii_whitespace();
        let key = msg.next();
        match key {
            Some("game_over") => {
                if self.is_host() {
                    info!("Game over, resending game settings");
                    {
                        let mut setting = self.settings.lock().unwrap();
                        if setting.debug_mode {
                            setting.seed += 1;
                        } else {
                            setting.seed = rand::random();
                        }
                        info!("New seed: {}", setting.seed);
                    }
                    self.resend_game_settings();
                }
            }
            key => {
                error!("Unknown msg from mod: {:?}", key)
            }
        }
    }
}
