use socket2::{Domain, Socket, Type};
use std::{
    env,
    fmt::Display,
    io::{self, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{atomic::AtomicBool, Arc, Mutex},
    thread,
    time::Duration,
};
use tracing::debug;
use world::{NoitaWorldUpdate, WorldManager};

use tangled::Reliability;
use tracing::{error, info, warn};
use tungstenite::{accept, WebSocket};

use crate::{messages::NetMsg, GameSettings};

pub mod steam_networking;
pub mod world;

pub(crate) fn ws_encode_proxy(key: &'static str, value: impl Display) -> tungstenite::Message {
    let mut buf = Vec::new();
    buf.push(2);
    write!(buf, "{} {}", key, value).unwrap();
    tungstenite::Message::Binary(buf)
}

pub(crate) fn ws_encode_proxy_opt(key: &'static str, value: impl Display) -> tungstenite::Message {
    let mut buf = Vec::new();
    buf.push(2);
    write!(buf, "proxy_opt {} {}", key, value).unwrap();
    tungstenite::Message::Binary(buf)
}

pub fn ws_encode_proxy_bin(key: u8, data: &[u8]) -> tungstenite::Message {
    let mut buf = Vec::new();
    buf.push(3);
    buf.push(key);
    buf.extend(data);
    tungstenite::Message::Binary(buf)
}

pub(crate) fn ws_encode_mod(peer: omni::OmniPeerId, data: &[u8]) -> tungstenite::Message {
    let mut buf = Vec::new();
    buf.push(1u8);
    buf.extend_from_slice(&peer.0.to_le_bytes());
    buf.extend_from_slice(data);
    tungstenite::Message::Binary(buf)
}

pub(crate) struct NetInnerState {
    pub(crate) ws: Option<WebSocket<TcpStream>>,
    world: WorldManager,
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

pub mod omni;

pub struct NetManagerInit {
    pub my_nickname: Option<String>,
}

pub struct NetManager {
    pub peer: omni::PeerVariant,
    pub settings: Mutex<GameSettings>,
    pub continue_running: AtomicBool, // TODO stop on drop
    pub accept_local: AtomicBool,
    pub local_connected: AtomicBool,
    pub stopped: AtomicBool,
    pub error: Mutex<Option<io::Error>>,
    pub init_settings: NetManagerInit,
}

impl NetManager {
    pub fn new(peer: omni::PeerVariant, init: NetManagerInit) -> Arc<Self> {
        Self {
            peer,
            settings: Mutex::new(GameSettings {
                // seed: 1663107061,
                seed: 1663107066,
                debug_mode: false,
                world_sync_version: 1,
            }),
            continue_running: AtomicBool::new(true),
            accept_local: AtomicBool::new(false),
            local_connected: AtomicBool::new(false),
            stopped: AtomicBool::new(false),
            error: Default::default(),
            init_settings: init,
        }
        .into()
    }

    pub(crate) fn send(&self, peer: omni::OmniPeerId, msg: &NetMsg, reliability: Reliability) {
        let encoded = lz4_flex::compress_prepend_size(&bitcode::encode(msg));
        self.peer.send(peer, encoded.clone(), reliability).ok(); // TODO log
    }

    pub(crate) fn broadcast(&self, msg: &NetMsg, reliability: Reliability) {
        let encoded = lz4_flex::compress_prepend_size(&bitcode::encode(msg));
        let len = encoded.len();
        if let Err(err) = self.peer.broadcast(encoded, reliability) {
            warn!("Error while broadcasting message of len {}: {}", len, err)
        }
    }

    pub(crate) fn start_inner(self: Arc<NetManager>) -> io::Result<()> {
        let socket = Socket::new(Domain::IPV4, Type::STREAM, None)?;

        // This allows several proxies to listen on the same address.
        // While this works, I couldn't get Noita to reliably connect to correct proxy instances on my os (linux).
        if env::var_os("NP_ALLOW_REUSE_ADDR").is_some() {
            info!("Address reuse allowed");
            if let Err(err) = socket.set_reuse_address(true) {
                error!("Could not allow to reuse address: {}", err)
            }
            #[cfg(target_os = "linux")]
            if let Err(err) = socket.set_reuse_port(true) {
                error!("Could not allow to reuse port: {}", err)
            }
        }

        let address: SocketAddr = env::var("NP_NOITA_ADDR")
            .ok()
            .and_then(|x| x.parse().ok())
            .unwrap_or_else(|| "127.0.0.1:21251".parse().unwrap());

        info!("Listening for noita connection on {}", address);

        let address = address.into();
        socket.bind(&address)?;
        socket.listen(1)?;
        socket.set_nonblocking(true)?;

        let local_server: TcpListener = socket.into();

        let mut state = NetInnerState {
            ws: None,
            world: WorldManager::new(),
        };

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
                    stream.set_nonblocking(false).ok();

                    state.ws = accept(stream)
                        .inspect_err(|e| error!("Could not init websocket: {}", e))
                        .ok();
                    if state.ws.is_some() {
                        self.on_ws_connection(&mut state);
                    }
                }
            }
            if let Some(ws) = &mut state.ws {
                ws.flush().ok();
            }
            for net_event in self.peer.recv() {
                match net_event {
                    omni::OmniNetworkEvent::PeerConnected(id) => {
                        self.broadcast(&NetMsg::Welcome, Reliability::Reliable);
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
                    omni::OmniNetworkEvent::PeerDisconnected(id) => {
                        state.try_ws_write(ws_encode_proxy("leave", id));
                    }
                    omni::OmniNetworkEvent::Message { src, data } => {
                        let Some(net_msg) = lz4_flex::decompress_size_prepended(&data)
                            .ok()
                            .and_then(|decomp| bitcode::decode::<NetMsg>(&decomp).ok())
                        else {
                            continue;
                        };
                        match net_msg {
                            NetMsg::Welcome => {
                                // info!("Got Welcome message from {}", src);
                                // let limit = if self.peer.is_steam() {
                                //     1024 * 1024
                                // } else {
                                //     30000
                                // };
                                // let deltas = state.world.send_world().split(limit);
                                // for delta in deltas {
                                //     self.send(
                                //         src,
                                //         &NetMsg::WorldDelta { delta },
                                //         Reliability::Reliable,
                                //     );
                                // }
                            }
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
                            NetMsg::WorldDelta { delta: deltas } => {
                                state.world.handle_deltas(deltas);
                            }
                            NetMsg::WorldFrame => {
                                let updates = state.world.get_noita_updates();
                                for update in updates {
                                    state.try_ws_write(ws_encode_proxy_bin(0, &update));
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
        Ok(())
    }

    fn on_ws_connection(self: &Arc<NetManager>, state: &mut NetInnerState) {
        info!("New stream connected");
        let stream_ref = &state.ws.as_ref().unwrap().get_ref();
        stream_ref.set_nonblocking(true).ok();
        stream_ref
            .set_read_timeout(Some(Duration::from_millis(1)))
            .expect("can set read timeout");

        let settings = self.settings.lock().unwrap();
        state.try_ws_write(ws_encode_proxy("seed", settings.seed));
        let value = self.peer.my_id().expect("Has peer id at this point");
        state.try_ws_write(ws_encode_proxy("peer_id", format!("{:016x}", value.0)));
        state.try_ws_write(ws_encode_proxy(
            "host_id",
            format!("{:016x}", self.peer.host_id().0),
        ));
        if let Some(nickname) = &self.init_settings.my_nickname {
            info!("Chosen nickname: {}", nickname);
            state.try_ws_write(ws_encode_proxy("name", nickname));
        } else {
            info!("No nickname chosen");
        }
        state.try_ws_write(ws_encode_proxy(
            "debug",
            if settings.debug_mode { "true" } else { "false" },
        ));
        state.try_ws_write(ws_encode_proxy_opt(
            "world_sync_version",
            settings.world_sync_version,
        ));

        state.try_ws_write(ws_encode_proxy("ready", ""));
        // TODO? those are currently ignored by mod
        for id in self.peer.iter_peer_ids() {
            state.try_ws_write(ws_encode_proxy("join", id));
        }

        info!("Settings sent")
    }

    pub(crate) fn handle_mod_message(
        &self,
        msg: Result<tungstenite::Message, tungstenite::Error>,
        state: &mut NetInnerState,
    ) {
        match msg {
            Ok(msg) => {
                if let tungstenite::Message::Binary(msg) = msg {
                    match msg[0] & 0b11 {
                        // Message to proxy
                        1 => {
                            self.handle_message_to_proxy(&msg[1..]);
                        }
                        // Broadcast
                        2 => {
                            let msg_to_send = if false {
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
                        // Binary message to proxy
                        3 => self.handle_bin_message_to_proxy(&msg[1..], state),
                        msg_variant => {
                            error!("Unknown msg variant from mod: {}", msg_variant)
                        }
                    }
                }
            }
            Err(tungstenite::Error::Io(io_err))
                if io_err.kind() == io::ErrorKind::WouldBlock
                    || io_err.kind() == io::ErrorKind::TimedOut => {}
            Err(err) => {
                error!("Error occured while reading from websocket: {}", err);
                state.ws = None;
            }
        }
    }

    pub fn start(self: Arc<NetManager>) {
        info!("Starting netmanager");
        thread::spawn(move || {
            let result = self.clone().start_inner();
            if let Err(err) = result {
                error!("Error in netmanager: {}", err);
                *self.error.lock().unwrap() = Some(err);
            }
            self.stopped
                .store(true, std::sync::atomic::Ordering::Relaxed);
        });
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
                            // setting.seed += 1;
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

    fn handle_bin_message_to_proxy(&self, msg: &[u8], state: &mut NetInnerState) {
        let key = msg[0];
        let data = &msg[1..];
        match key {
            // world frame
            0 => {
                let update = NoitaWorldUpdate::load(data);
                state.world.add_update(update);
            }
            // world end
            1 => {
                let deltas = state.world.add_end();
                let limit = if self.peer.is_steam() {
                    1024 * 1024
                } else {
                    30000
                };
                for delta in deltas.split(limit) {
                    self.broadcast(&NetMsg::WorldDelta { delta }, Reliability::Reliable);
                }
                self.broadcast(&NetMsg::WorldFrame, Reliability::Reliable);
            }
            key => {
                error!("Unknown bin msg from mod: {:?}", key)
            }
        }
    }
}
