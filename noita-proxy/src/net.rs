use bitcode::{Decode, Encode};
use image::Rgb;
use messages::{MessageRequest, NetMsg};
use omni::OmniPeerId;
use proxy_opt::ProxyOpt;
use socket2::{Domain, Socket, Type};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::{
    env,
    fmt::Display,
    io::{self, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        atomic::{self, AtomicBool},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use world::{world_info::WorldInfo, NoitaWorldUpdate, WorldManager};

use tangled::Reliability;
use tracing::{error, info, warn};
use tungstenite::{accept, WebSocket};

use crate::{
    bookkeeping::save_state::{SaveState, SaveStateEntry},
    recorder::Recorder,
    replace_color, GameSettings,
};
pub mod messages;
mod proxy_opt;
pub mod steam_networking;
pub mod world;

pub(crate) fn ws_encode_proxy(key: &'static str, value: impl Display) -> tungstenite::Message {
    let mut buf = Vec::new();
    buf.push(2);
    write!(buf, "{} {}", key, value).unwrap();
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

#[derive(Encode, Decode)]
pub(crate) struct RunInfo {
    pub(crate) seed: u64,
}

impl SaveStateEntry for RunInfo {
    const FILENAME: &'static str = "run_info";
}

pub(crate) struct NetInnerState {
    pub(crate) ws: Option<WebSocket<TcpStream>>,
    world: WorldManager,
    recorder: Option<Recorder>,
}

impl NetInnerState {
    pub(crate) fn try_ws_write(&mut self, data: tungstenite::Message) {
        if let Some(ws) = &mut self.ws {
            if let Some(recorder) = &mut self.recorder {
                recorder.record_msg(&data);
            }
            if let Err(err) = ws.write(data) {
                error!("Error occured while sending to websocket: {}", err);
                self.ws = None;
                self.recorder = None;
            };
        }
    }
    pub(crate) fn try_ws_write_option(&mut self, key: &str, value: impl ProxyOpt) {
        let mut buf = Vec::new();
        buf.push(2);
        value.write_opt(&mut buf, key);
        let message = tungstenite::Message::Binary(buf);
        self.try_ws_write(message);
    }
}

pub mod omni;

pub struct NetManagerInit {
    pub my_nickname: Option<String>,
    pub save_state: SaveState,
    pub player_main_color: [u8; 3],
    pub player_alt_color: [u8; 3],
}

pub struct NetManager {
    pub peer: omni::PeerVariant,
    pub pending_settings: Mutex<GameSettings>,
    pub settings: Mutex<GameSettings>,
    pub continue_running: AtomicBool, // TODO stop on drop
    pub accept_local: AtomicBool,
    pub local_connected: AtomicBool,
    pub stopped: AtomicBool,
    pub error: Mutex<Option<io::Error>>,
    pub init_settings: NetManagerInit,
    pub world_info: WorldInfo,
    pub enable_recorder: AtomicBool,
}

impl NetManager {
    pub fn new(peer: omni::PeerVariant, init: NetManagerInit) -> Arc<Self> {
        Self {
            peer,
            pending_settings: Mutex::new(GameSettings::default()),
            settings: Mutex::new(GameSettings::default()),
            continue_running: AtomicBool::new(true),
            accept_local: AtomicBool::new(false),
            local_connected: AtomicBool::new(false),
            stopped: AtomicBool::new(false),
            error: Default::default(),
            init_settings: init,
            world_info: Default::default(),
            enable_recorder: AtomicBool::new(false),
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

    pub(crate) fn start_inner(self: Arc<NetManager>, player_path: PathBuf) -> io::Result<()> {
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

        for _ in 1..3 {
            if self.peer.my_id().is_none() {
                info!("Waiting on my_id...");
                thread::sleep(Duration::from_millis(100));
            } else {
                break;
            }
        }
        if self.peer.my_id().is_none() {
            std::process::exit(1)
        }

        let is_host = self.is_host();
        info!("Is host: {is_host}");
        let mut state = NetInnerState {
            ws: None,
            recorder: None,
            world: WorldManager::new(
                is_host,
                self.peer.my_id().unwrap(),
                self.init_settings.save_state.clone(),
            ),
        };

        let mut last_iter = Instant::now();

        while self.continue_running.load(atomic::Ordering::Relaxed) {
            self.local_connected
                .store(state.ws.is_some(), atomic::Ordering::Relaxed);
            if state.ws.is_none() && self.accept_local.load(atomic::Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(10));
                if let Ok((stream, addr)) = local_server.accept() {
                    info!("New stream incoming from {}", addr);
                    stream.set_nodelay(true).ok();
                    stream.set_nonblocking(false).ok();

                    state.ws = accept(stream)
                        .inspect_err(|e| error!("Could not init websocket: {}", e))
                        .ok();
                    if state.ws.is_some() {
                        if self.enable_recorder.load(atomic::Ordering::Relaxed) {
                            state.recorder = Some(Recorder::default());
                        }
                        self.on_ws_connection(&mut state);
                    }
                }
            }
            if let Some(ws) = &mut state.ws {
                if let Err(err) = ws.flush() {
                    warn!("Websocket flush not ok: {err}");
                }
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
                        state.try_ws_write(ws_encode_proxy("join", id.as_hex()));
                        self.create_player_png(player_path.clone());
                    }
                    omni::OmniNetworkEvent::PeerDisconnected(id) => {
                        state.try_ws_write(ws_encode_proxy("leave", id.as_hex()));
                        state.world.handle_peer_left(id);
                    }
                    omni::OmniNetworkEvent::Message { src, data } => {
                        let Some(net_msg) = lz4_flex::decompress_size_prepended(&data)
                            .ok()
                            .and_then(|decomp| bitcode::decode::<NetMsg>(&decomp).ok())
                        else {
                            continue;
                        };
                        match net_msg {
                            NetMsg::Welcome => {}
                            NetMsg::StartGame { settings } => {
                                *self.settings.lock().unwrap() = settings;
                                info!("Settings updated");
                                self.accept_local.store(true, atomic::Ordering::SeqCst);
                                state.world.reset();
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
                            NetMsg::WorldMessage(msg) => state.world.handle_msg(src, msg),
                        }
                    }
                }
            }

            // Handle all available messages from Noita.

            while let Some(ws) = &mut state.ws {
                let msg = ws.read();

                match msg {
                    Ok(msg) => {
                        if let tungstenite::Message::Binary(msg) = msg {
                            self.handle_mod_message_2(msg, &mut state);
                        }
                    }
                    Err(tungstenite::Error::Io(io_err))
                        if io_err.kind() == io::ErrorKind::WouldBlock
                            || io_err.kind() == io::ErrorKind::TimedOut =>
                    {
                        break
                    }
                    Err(err) => {
                        error!("Error occured while reading from websocket: {}", err);
                        state.ws = None;
                    }
                }
            }

            for msg in state.world.get_emitted_msgs() {
                self.do_message_request(msg)
            }
            state.world.update();
            let updates = state.world.get_noita_updates();
            for update in updates {
                state.try_ws_write(ws_encode_proxy_bin(0, &update));
            }
            // Don't do excessive busy-waiting;
            let min_update_time = Duration::from_millis(1);
            let elapsed = last_iter.elapsed();
            if elapsed < min_update_time {
                thread::sleep(min_update_time - elapsed);
            }
            last_iter = Instant::now();
        }
        Ok(())
    }

    fn create_player_png(&self, player_path: PathBuf) {
        let mut img = image::open(player_path.clone().into_os_string())
            .unwrap()
            .into_rgb8();
        replace_color(
            &mut img,
            Rgb::from(self.init_settings.player_main_color),
            Rgb::from(self.init_settings.player_alt_color),
        );
        let path = player_path
            .clone()
            .parent()
            .unwrap()
            .join(format!("{}.png", self.peer.my_id().unwrap()));
        img.save(path).unwrap();

        let mut img = image::open(
            player_path
                .clone()
                .parent()
                .unwrap()
                .join("unmodified_arm.png")
                .into_os_string(),
        )
        .unwrap()
        .into_rgb8();
        replace_color(
            &mut img,
            Rgb::from(self.init_settings.player_main_color),
            Rgb::from(self.init_settings.player_alt_color),
        );
        let path = player_path
            .clone()
            .parent()
            .unwrap()
            .join(format!("{}_arm.png", self.peer.my_id().unwrap()));
        img.save(path).unwrap();

        let file = File::open(
            player_path
                .clone()
                .parent()
                .unwrap()
                .join("unmodified_cape.xml")
                .into_os_string(),
        )
        .unwrap();
        let reader = BufReader::new(file);
        let mut lines = reader.lines().map(|l| l.unwrap()).collect::<Vec<String>>();
        lines.insert(
            5,
            format!(
                "cloth_color=\"0x{}FF\"",
                Self::rgb_to_hex(self.init_settings.player_alt_color)
            ),
        );
        lines.insert(
            5,
            format!(
                "cloth_color_edge=\"0x{}FF\"",
                Self::rgb_to_hex(self.init_settings.player_main_color)
            ),
        );

        let path = player_path
            .clone()
            .parent()
            .unwrap()
            .join(format!("{}_cape.xml", self.peer.my_id().unwrap()));
        let mut file = File::create(path).unwrap();
        for line in lines {
            writeln!(file, "{}", line).unwrap();
        }
    }
    fn rgb_to_hex(rgb: [u8; 3]) -> String {
        format!("{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
    }

    fn do_message_request(&self, request: impl Into<MessageRequest<NetMsg>>) {
        let request: MessageRequest<NetMsg> = request.into();
        match request.dst {
            messages::Destination::Peer(peer) => {
                self.send(peer, &request.msg, request.reliability);
            }
            messages::Destination::Host => {
                self.send(self.peer.host_id(), &request.msg, request.reliability);
            }
            messages::Destination::Broadcast => self.broadcast(&request.msg, request.reliability),
        }
    }

    fn on_ws_connection(self: &Arc<NetManager>, state: &mut NetInnerState) {
        self.init_settings.save_state.mark_game_started();
        info!("New stream connected");
        let stream_ref = &state.ws.as_ref().unwrap().get_ref();
        stream_ref.set_nonblocking(true).ok();
        stream_ref
            .set_read_timeout(Some(Duration::from_millis(1)))
            .expect("can set read timeout");
        // Set write timeout to a somewhat high value just in case.
        stream_ref
            .set_write_timeout(Some(Duration::from_secs(5)))
            .expect("can set write timeout");

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
            state.try_ws_write_option("name", nickname.as_str());
        } else {
            info!("No nickname chosen");
        }
        state.try_ws_write_option("debug", settings.debug_mode);
        state.try_ws_write_option("world_sync_version", settings.world_sync_version);
        state.try_ws_write_option("player_tether", settings.player_tether);
        state.try_ws_write_option("tether_length", settings.tether_length);
        state.try_ws_write_option("item_dedup", settings.item_dedup);
        state.try_ws_write_option("enemy_hp_scale", settings.enemy_hp_mult);
        state.try_ws_write_option("world_sync_interval", settings.world_sync_interval);
        state.try_ws_write_option("game_mode", settings.game_mode);

        state.try_ws_write(ws_encode_proxy("ready", ""));
        // TODO? those are currently ignored by mod
        for id in self.peer.iter_peer_ids() {
            state.try_ws_write(ws_encode_proxy("join", id.as_hex()));
        }
        info!("Settings sent")
    }

    fn handle_mod_message_2(&self, msg: Vec<u8>, state: &mut NetInnerState) {
        match msg[0] & 0b11 {
            // Message to proxy
            1 => {
                self.handle_message_to_proxy(&msg[1..], state);
            }
            // Broadcast
            2 => {
                let msg_to_send =/* if false {
                    let compressed = lz4_flex::compress_prepend_size(&msg[1..]);

                    debug!(
                        "Compressed {} bytes to {} bytes",
                        msg.len(),
                        compressed.len()
                    );

                    NetMsg::ModCompressed { data: compressed }
                } else*/ {
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

    pub fn start(self: Arc<NetManager>, player_path: PathBuf) -> JoinHandle<()> {
        info!("Starting netmanager");
        thread::spawn(move || {
            let result = self.clone().start_inner(player_path);
            if let Err(err) = result {
                error!("Error in netmanager: {}", err);
                *self.error.lock().unwrap() = Some(err);
            }
            self.stopped.store(true, atomic::Ordering::Relaxed);
        })
    }

    fn resend_game_settings(&self) {
        let settings = self.settings.lock().unwrap().clone();
        self.broadcast(&NetMsg::StartGame { settings }, Reliability::Reliable);
    }

    fn is_host(&self) -> bool {
        self.peer.is_host()
    }

    pub(crate) fn handle_message_to_proxy(&self, msg: &[u8], state: &mut NetInnerState) {
        let msg = String::from_utf8_lossy(msg);
        let mut msg = msg.split_ascii_whitespace();
        let key = msg.next();
        match key {
            Some("game_over") => {
                if self.is_host() {
                    info!("Game over, resending game settings");
                    self.init_settings.save_state.reset();
                    {
                        let mut settings = self.pending_settings.lock().unwrap().clone();
                        if !settings.use_constant_seed {
                            settings.seed = rand::random();
                        }
                        info!("New seed: {}", settings.seed);
                        *self.settings.lock().unwrap() = settings;
                        state.world.reset()
                    }
                    self.resend_game_settings();
                }
            }
            Some("peer_pos") => {
                let peer_id = msg.next().and_then(OmniPeerId::from_hex);
                let x: Option<f64> = msg.next().and_then(|s| s.parse().ok());
                let y: Option<f64> = msg.next().and_then(|s| s.parse().ok());
                if let (Some(peer_id), Some(x), Some(y)) = (peer_id, x, y) {
                    self.world_info.update_player_pos(peer_id, x, y);
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
                state.world.add_end();
            }
            key => {
                error!("Unknown bin msg from mod: {:?}", key)
            }
        }
    }
}

impl Drop for NetManager {
    fn drop(&mut self) {
        if self.is_host() {
            let run_info = RunInfo {
                seed: self.settings.lock().unwrap().seed,
            };
            self.init_settings.save_state.save(&run_info);
            info!("Saved run info");
        } else {
            info!("Skip saving run info: not a host");
        }
    }
}
