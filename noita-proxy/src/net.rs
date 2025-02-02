use bitcode::{Decode, Encode};
use des::DesManager;
use image::DynamicImage::ImageRgba8;
use image::{ImageBuffer, Rgba, RgbaImage};
use messages::{MessageRequest, NetMsg};
use omni::OmniPeerId;
use proxy_opt::ProxyOpt;
use shared::message_socket::MessageSocket;
use shared::{Destination, NoitaInbound, NoitaOutbound, RemoteMessage};
use socket2::{Domain, Socket, Type};
use std::collections::HashMap;
use std::fs::{create_dir, remove_dir_all, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::{Arc, Mutex};
use std::{
    env,
    fmt::Display,
    io::{self},
    net::{SocketAddr, TcpListener},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use world::{world_info::WorldInfo, NoitaWorldUpdate, WorldManager};

use crate::mod_manager::{get_mods, ModmanagerSettings};
use crate::net::world::world_model::chunk::{Pixel, PixelFlags};
use crate::player_cosmetics::{create_player_png, get_player_skin, PlayerPngDesc};
use crate::{
    bookkeeping::save_state::{SaveState, SaveStateEntry},
    DefaultSettings, GameMode, GameSettings, LocalHealthMode,
};
use shared::des::ProxyToDes;
use tangled::Reliability;
use tracing::{error, info, warn};
mod des;
pub mod messages;
mod proxy_opt;
pub mod steam_networking;
pub mod world;

pub(crate) fn ws_encode_proxy(key: &'static str, value: impl Display) -> NoitaInbound {
    let mut buf = Vec::new();
    buf.push(2);
    write!(buf, "{} {}", key, value).unwrap();
    NoitaInbound::RawMessage(buf)
}

pub fn ws_encode_proxy_bin(key: u8, data: &[u8]) -> NoitaInbound {
    let mut buf = Vec::new();
    buf.push(3);
    buf.push(key);
    buf.extend(data);
    NoitaInbound::RawMessage(buf)
}

pub(crate) fn ws_encode_mod(peer: OmniPeerId, data: &[u8]) -> NoitaInbound {
    let mut buf = Vec::new();
    buf.push(1u8);
    buf.extend_from_slice(&peer.0.to_le_bytes());
    buf.extend_from_slice(data);
    NoitaInbound::RawMessage(buf)
}

#[derive(Encode, Decode)]
pub(crate) struct RunInfo {
    pub(crate) seed: u64,
}

impl SaveStateEntry for RunInfo {
    const FILENAME: &'static str = "run_info";
}

pub(crate) struct NetInnerState {
    pub(crate) ms: Option<MessageSocket<NoitaOutbound, NoitaInbound>>,
    world: WorldManager,
    explosion_data: Vec<ExplosionData>,
    des: DesManager,
    had_a_disconnect: bool,
}

impl NetInnerState {
    pub(crate) fn try_ms_write(&mut self, data: &NoitaInbound) {
        if let Some(ws) = &mut self.ms {
            if let Err(err) = ws.write(data) {
                error!("Error occured while sending to websocket: {}", err);
                self.ms = None;
                self.had_a_disconnect = true;
            };
        }
    }
    pub(crate) fn try_ws_write_option(&mut self, key: &str, value: impl ProxyOpt) {
        let mut buf = Vec::new();
        buf.push(2);
        value.write_opt(&mut buf, key);
        let message = NoitaInbound::RawMessage(buf);
        self.try_ms_write(&message);
    }
}

pub mod omni;

pub struct NetManagerInit {
    pub my_nickname: String,
    pub save_state: SaveState,
    pub cosmetics: (bool, bool, bool),
    pub mod_path: PathBuf,
    pub player_path: PathBuf,
    pub modmanager_settings: ModmanagerSettings,
    pub player_png_desc: PlayerPngDesc,
    pub noita_port: u16,
}

pub struct NetManager {
    pub peer: omni::PeerVariant,
    pub pending_settings: Mutex<GameSettings>,
    pub settings: Mutex<GameSettings>,
    pub continue_running: AtomicBool,
    pub accept_local: AtomicBool,
    pub local_connected: AtomicBool,
    pub stopped: AtomicBool,
    pub error: Mutex<Option<io::Error>>,
    pub init_settings: NetManagerInit,
    pub world_info: WorldInfo,
    pub enable_recorder: AtomicBool,
    pub end_run: AtomicBool,
    pub ban_list: Mutex<Vec<OmniPeerId>>,
    pub kick_list: Mutex<Vec<OmniPeerId>>,
    pub no_more_players: AtomicBool,
    dont_kick: Mutex<Vec<OmniPeerId>>,
    pub dirty: AtomicBool,
    pub actual_noita_port: AtomicU16,
    pub active_mods: Mutex<Vec<String>>,
    pub nicknames: Mutex<HashMap<OmniPeerId, String>>,
    pub minas: Mutex<HashMap<OmniPeerId, RgbaImage>>,
    pub new_desc: Mutex<Option<PlayerPngDesc>>,
    loopback_channel: (
        crossbeam::channel::Sender<NetMsg>,
        crossbeam::channel::Receiver<NetMsg>,
    ),
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
            end_run: AtomicBool::new(false),
            ban_list: Default::default(),
            kick_list: Default::default(),
            no_more_players: AtomicBool::new(false),
            dont_kick: Default::default(),
            dirty: AtomicBool::new(false),
            actual_noita_port: AtomicU16::new(0),
            active_mods: Default::default(),
            nicknames: Default::default(),
            minas: Default::default(),
            new_desc: Default::default(),
            loopback_channel: crossbeam::channel::unbounded(),
        }
        .into()
    }

    pub(crate) fn send(&self, peer: OmniPeerId, msg: &NetMsg, reliability: Reliability) {
        if peer == self.peer.my_id() {
            // Shortcut for sending stuff to myself
            let _ = self.loopback_channel.0.send(msg.clone());
        } else {
            let encoded = lz4_flex::compress_prepend_size(&bitcode::encode(msg));
            let len = encoded.len();
            if let Err(err) = self.peer.send(peer, encoded.clone(), reliability) {
                warn!("Error while sending message of len {}: {}", len, err)
            }
        }
    }

    pub(crate) fn broadcast(&self, msg: &NetMsg, reliability: Reliability) {
        let encoded = lz4_flex::compress_prepend_size(&bitcode::encode(msg));
        let len = encoded.len();
        if let Err(err) = self.peer.broadcast(encoded, reliability) {
            warn!("Error while broadcasting message of len {}: {}", len, err)
        }
    }

    fn clean_dir(path: PathBuf) {
        let tmp = path.parent().unwrap().join("tmp");
        if tmp.exists() {
            remove_dir_all(tmp.clone()).ok();
        }
        create_dir(tmp).ok();
    }

    pub fn new_desc(&self, desc: PlayerPngDesc, player_image: RgbaImage) {
        create_player_png(
            self.peer.my_id(),
            &self.init_settings.mod_path,
            &self.init_settings.player_path,
            &desc,
            self.is_host(),
        );
        self.minas
            .lock()
            .unwrap()
            .insert(self.peer.my_id(), get_player_skin(player_image, desc));
        self.broadcast(
            &NetMsg::PlayerColor(
                desc,
                self.is_host(),
                Some(self.peer.my_id()),
                self.init_settings.my_nickname.clone(),
            ),
            Reliability::Reliable,
        );
    }

    pub(crate) fn start_inner(
        self: Arc<NetManager>,
        player_path: PathBuf,
        mut cli: bool,
    ) -> io::Result<()> {
        Self::clean_dir(player_path.clone());
        if !self.init_settings.cosmetics.0 {
            File::create(player_path.parent().unwrap().join("tmp/no_crown"))?;
        }
        if !self.init_settings.cosmetics.1 {
            File::create(player_path.parent().unwrap().join("tmp/no_amulet"))?;
        }
        if !self.init_settings.cosmetics.2 {
            File::create(player_path.parent().unwrap().join("tmp/no_amulet_gem"))?;
        }

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
            .unwrap_or_else(|| {
                SocketAddr::new("127.0.0.1".parse().unwrap(), self.init_settings.noita_port)
            });
        info!("Listening for noita connection on {}", address);
        let address = address.into();
        socket.bind(&address)?;
        socket.listen(1)?;
        socket.set_nonblocking(true)?;

        let actual_port = socket.local_addr()?.as_socket().unwrap().port();
        self.actual_noita_port.store(actual_port, Ordering::Relaxed);
        info!("Actual Noita port: {actual_port}");

        if self.is_host() {
            self.accept_local.store(true, Ordering::Relaxed);
        }

        let local_server: TcpListener = socket.into();

        let is_host = self.is_host();
        info!("Is host: {is_host}");
        let mut state = NetInnerState {
            ms: None,
            world: WorldManager::new(
                is_host,
                self.peer.my_id(),
                self.init_settings.save_state.clone(),
            ),
            explosion_data: Vec::new(),
            des: DesManager::new(is_host, self.init_settings.save_state.clone()),
            had_a_disconnect: false,
        };
        let mut last_iter = Instant::now();
        let path = crate::player_path(self.init_settings.modmanager_settings.mod_path());
        let player_image = if path.exists() {
            image::open(path)
                .unwrap_or(ImageRgba8(RgbaImage::new(20, 20)))
                .crop(1, 1, 8, 18)
                .into_rgba8()
        } else {
            RgbaImage::new(1, 1)
        };
        // Create appearance files for local player.
        create_player_png(
            self.peer.my_id(),
            &self.init_settings.mod_path,
            &self.init_settings.player_path,
            &self.init_settings.player_png_desc,
            self.is_host(),
        );
        self.nicknames
            .lock()
            .unwrap()
            .insert(self.peer.my_id(), self.init_settings.my_nickname.clone());
        self.minas.lock().unwrap().insert(
            self.peer.my_id(),
            get_player_skin(player_image.clone(), self.init_settings.player_png_desc),
        );
        while self.continue_running.load(Ordering::Relaxed) {
            if cli {
                if let Some(n) = self.peer.lobby_id() {
                    println!("Lobby ID: {}", n.raw());
                    cli = false
                }
            }
            if self.end_run.load(Ordering::Relaxed) {
                for id in self.peer.iter_peer_ids() {
                    self.send(id, &NetMsg::EndRun, Reliability::Reliable);
                }
                state.try_ms_write(&ws_encode_proxy("end_run", self.peer.my_id().to_string()));
                self.end_run(&mut state);
                self.end_run.store(false, Ordering::Relaxed);
            }
            self.local_connected
                .store(state.ms.is_some(), Ordering::Relaxed);
            if state.ms.is_none() && self.accept_local.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(10));
                if let Ok((stream, addr)) = local_server.accept() {
                    info!("New stream incoming from {}", addr);
                    stream.set_nodelay(true).ok();
                    stream.set_nonblocking(false).ok();
                    state.ms = MessageSocket::new(stream)
                        .inspect_err(|e| error!("Could not init websocket: {:?}", e))
                        .ok();
                    if state.ms.is_some() {
                        self.on_ms_connection(&mut state);
                    }
                }
            }
            if let Some(ws) = &mut state.ms {
                if let Err(err) = ws.flush() {
                    warn!("Websocket flush not ok: {err}");
                }
            }
            let mut to_kick = self.kick_list.lock().unwrap();
            let mut dont_kick = self.dont_kick.lock().unwrap();
            if self.no_more_players.load(Ordering::Relaxed) {
                if dont_kick.is_empty() {
                    dont_kick.extend(self.peer.iter_peer_ids())
                } else {
                    for peer in self.peer.iter_peer_ids() {
                        if !dont_kick.contains(&peer) {
                            to_kick.push(peer);
                        }
                    }
                }
            } else {
                dont_kick.clear()
            }
            {
                let list = self.ban_list.lock().unwrap();
                for peer in list.iter() {
                    if self.peer.iter_peer_ids().contains(peer) {
                        to_kick.push(*peer)
                    }
                }
            }
            for peer in to_kick.iter() {
                info!("player kicked: {}", peer);
                state.try_ms_write(&ws_encode_proxy("leave", peer.as_hex()));
                state.world.handle_peer_left(*peer);
                self.send(*peer, &NetMsg::Kick, Reliability::Reliable);
                self.broadcast(
                    &NetMsg::PeerDisconnected { id: *peer },
                    Reliability::Reliable,
                );
            }
            to_kick.clear();
            for net_event in self.peer.recv() {
                self.clone()
                    .handle_network_event(&mut state, &player_image, net_event);
            }
            for net_msg in self.loopback_channel.1.try_iter() {
                self.clone()
                    .handle_net_msg(&mut state, &player_image, self.peer.my_id(), net_msg);
            }
            // Handle all available messages from Noita.
            while let Some(ws) = &mut state.ms {
                let msg = ws.try_read();
                match msg {
                    Ok(Some(msg)) => {
                        self.handle_mod_message_2(msg, &mut state);
                    }
                    Ok(None) => break,
                    Err(err) => {
                        warn!("Game closed (Lost connection to noita instance: {})", err);
                        state.had_a_disconnect = true;
                        state.ms = None;
                    }
                }
            }
            for msg in state.world.get_emitted_msgs() {
                self.do_message_request(msg)
            }
            state.world.update();

            let updates = state.world.get_noita_updates();
            for update in updates {
                state.try_ms_write(&ws_encode_proxy_bin(0, &update));
            }

            if state.had_a_disconnect {
                self.broadcast(&NetMsg::NoitaDisconnected, Reliability::Reliable);
                if self.is_host() {
                    state.des.noita_disconnected(self.peer.my_id());
                }
                state.had_a_disconnect = false;
            }

            let des_pending = state.des.pending_messages();
            for (dest, msg) in des_pending {
                self.send(dest, &NetMsg::ForwardProxyToDes(msg), Reliability::Reliable);
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

    fn handle_network_event(
        self: Arc<NetManager>,
        state: &mut NetInnerState,
        player_image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
        net_event: omni::OmniNetworkEvent,
    ) {
        match net_event {
            omni::OmniNetworkEvent::PeerConnected(id) => {
                self.broadcast(&NetMsg::Welcome, Reliability::Reliable);
                info!("Peer connected {id}");
                if self.peer.my_id() == self.peer.host_id() {
                    info!("Sending start game message");
                    self.send(
                        id,
                        &NetMsg::StartGame {
                            settings: self.settings.lock().unwrap().clone(),
                        },
                        Reliability::Reliable,
                    );
                }
                if id != self.peer.my_id() {
                    // Create temporary appearance files for new player.
                    info!("Created temporary appearance for {id}");
                    create_player_png(
                        id,
                        &self.init_settings.mod_path,
                        &self.init_settings.player_path,
                        &PlayerPngDesc::default(),
                        id == self.peer.host_id(),
                    );
                    info!("Sending PlayerColor to {id}");
                    self.send(
                        id,
                        &NetMsg::PlayerColor(
                            self.init_settings.player_png_desc,
                            self.is_host(),
                            Some(self.peer.my_id()),
                            self.init_settings.my_nickname.clone(),
                        ),
                        Reliability::Reliable,
                    );
                }
                state.try_ms_write(&ws_encode_proxy("join", id.as_hex()));
            }
            omni::OmniNetworkEvent::PeerDisconnected(id) => {
                state.try_ms_write(&ws_encode_proxy("leave", id.as_hex()));
                state.world.handle_peer_left(id);
            }
            omni::OmniNetworkEvent::Message { src, data } => {
                let Some(net_msg) = lz4_flex::decompress_size_prepended(&data)
                    .ok()
                    .and_then(|decomp| bitcode::decode::<NetMsg>(&decomp).ok())
                else {
                    return;
                };
                self.handle_net_msg(state, player_image, src, net_msg);
            }
        }
    }

    fn handle_net_msg(
        self: Arc<NetManager>,
        state: &mut NetInnerState,
        player_image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
        src: OmniPeerId,
        net_msg: NetMsg,
    ) {
        match net_msg {
            NetMsg::RequestMods => {
                if let Some(n) = &self.init_settings.modmanager_settings.game_save_path {
                    let res = get_mods(n);
                    if let Ok(mods) = res {
                        self.send(src, &NetMsg::Mods { mods }, Reliability::Reliable)
                    }
                }
            }
            NetMsg::Mods { mods } => *self.active_mods.lock().unwrap() = mods,
            NetMsg::Welcome => {}
            NetMsg::PeerDisconnected { id } => {
                info!("player kicked: {}", id);
                state.try_ms_write(&ws_encode_proxy("leave", id.as_hex()));
                state.world.handle_peer_left(id);
            }
            NetMsg::EndRun => {
                state.try_ms_write(&ws_encode_proxy("end_run", self.peer.my_id().to_string()))
            }
            NetMsg::StartGame { settings } => {
                *self.settings.lock().unwrap() = settings;
                info!("Settings updated");
                self.accept_local.store(true, Ordering::SeqCst);
                state.world.reset();
            }
            NetMsg::ModRaw { data } => {
                state.try_ms_write(&ws_encode_mod(src, &data));
            }
            NetMsg::ModCompressed { data } => {
                if let Ok(decompressed) = lz4_flex::decompress_size_prepended(&data) {
                    state.try_ms_write(&ws_encode_mod(src, &decompressed));
                }
            }
            NetMsg::WorldMessage(msg) => state.world.handle_msg(src, msg),
            NetMsg::PlayerColor(rgb, host, pong, name) => {
                info!("Player appearance created for {}", src);
                // Create proper appearance files for new player.
                create_player_png(
                    src,
                    &self.init_settings.mod_path,
                    &self.init_settings.player_path,
                    &rgb,
                    host,
                );
                self.nicknames.lock().unwrap().insert(src, name);
                self.minas
                    .lock()
                    .unwrap()
                    .insert(src, get_player_skin(player_image.clone(), rgb));
                if let Some(id) = pong {
                    if id != self.peer.my_id() {
                        self.send(
                            id,
                            &NetMsg::PlayerColor(
                                self.init_settings.player_png_desc,
                                self.is_host(),
                                None,
                                self.init_settings.my_nickname.clone(),
                            ),
                            Reliability::Reliable,
                        );
                    }
                }
            }
            NetMsg::Kick => std::process::exit(0),
            NetMsg::RemoteMsg(remote_message) => self.handle_remote_msg(state, src, remote_message),
            NetMsg::ForwardDesToProxy(des_to_proxy) => {
                state.des.handle_noita_msg(src, des_to_proxy)
            }
            NetMsg::ForwardProxyToDes(proxy_to_des) => {
                state.try_ms_write(&NoitaInbound::ProxyToDes(proxy_to_des));
            }
            NetMsg::NoitaDisconnected => {
                state.des.noita_disconnected(src);
                state.try_ms_write(&NoitaInbound::ProxyToDes(ProxyToDes::RemoveEntities(
                    src.into(),
                )));
                state.try_ms_write(&ws_encode_proxy("dc", src.as_hex()));
            }
        }
    }

    fn handle_remote_msg(
        &self,
        state: &mut NetInnerState,
        src: OmniPeerId,
        message: RemoteMessage,
    ) {
        state.try_ms_write(&NoitaInbound::RemoteMessage {
            source: src.into(),
            message,
        });
    }

    fn do_message_request(&self, request: impl Into<MessageRequest<NetMsg>>) {
        let request: MessageRequest<NetMsg> = request.into();
        match request.dst {
            Destination::Peer(peer) => {
                self.send(peer, &request.msg, request.reliability);
            }
            Destination::Host => {
                self.send(self.peer.host_id(), &request.msg, request.reliability);
            }
            Destination::Broadcast => self.broadcast(&request.msg, request.reliability),
        }
    }

    fn on_ms_connection(self: &Arc<NetManager>, state: &mut NetInnerState) {
        self.init_settings.save_state.mark_game_started();
        info!("New stream connected");

        let settings = self.settings.lock().unwrap();
        let def = DefaultSettings::default();
        state.try_ms_write(&ws_encode_proxy("seed", settings.seed));
        let my_id = self.peer.my_id();
        state.try_ms_write(&ws_encode_proxy("peer_id", format!("{:016x}", my_id.0)));
        state.try_ms_write(&ws_encode_proxy(
            "host_id",
            format!("{:016x}", self.peer.host_id().0),
        ));
        info!("Chosen nickname: {}", self.init_settings.my_nickname);
        state.try_ws_write_option("name", self.init_settings.my_nickname.as_str());
        state.try_ws_write_option("world_num", settings.world_num as u32);
        state.try_ws_write_option(
            "friendly_fire",
            settings.friendly_fire.unwrap_or(def.friendly_fire),
        );
        state.try_ws_write_option("share_gold", settings.share_gold.unwrap_or(def.share_gold));
        state.try_ws_write_option(
            "same_loadout",
            settings.same_loadout.unwrap_or(def.same_loadout),
        );
        state.try_ws_write_option("debug", settings.debug_mode.unwrap_or(def.debug_mode));
        // state.try_ws_write_option("item_dedup", settings.item_dedup.unwrap_or(def.item_dedup)); TODO
        state.try_ws_write_option("duplicate", settings.duplicate.unwrap_or(def.duplicate));
        state.try_ws_write_option(
            "randomize_perks",
            settings.randomize_perks.unwrap_or(def.randomize_perks),
        );
        state.try_ws_write_option(
            "enemy_hp_scale",
            settings.enemy_hp_mult.unwrap_or(def.enemy_hp_mult),
        );
        let mode = settings.game_mode.unwrap_or(def.game_mode);
        state.try_ws_write_option("game_mode", mode);
        if let GameMode::LocalHealth(mode) = mode {
            match mode {
                LocalHealthMode::Normal => {}
                LocalHealthMode::PermaDeath => state.try_ws_write_option("perma_death", true),
                LocalHealthMode::Alternate => state.try_ws_write_option("no_notplayer", true),
            }
        }
        state.try_ws_write_option(
            "health_per_player",
            settings.health_per_player.unwrap_or(def.health_per_player),
        );
        state.try_ws_write_option(
            "global_hp_loss",
            settings.global_hp_loss.unwrap_or(def.global_hp_loss),
        );
        state.try_ws_write_option(
            "physics_damage",
            settings.physics_damage.unwrap_or(def.physics_damage),
        );
        let lst = settings.clone();
        state.try_ws_write_option(
            "perk_ban_list",
            lst.perk_ban_list.unwrap_or(def.perk_ban_list).as_str(),
        );
        state.try_ws_write_option(
            "no_material_damage",
            settings
                .no_material_damage
                .unwrap_or(def.no_material_damage),
        );
        state.try_ws_write_option(
            "health_lost_on_revive",
            settings
                .health_lost_on_revive
                .unwrap_or(def.health_lost_on_revive),
        );
        state.world.nice_terraforming = settings.nice_terraforming.unwrap_or(def.nice_terraforming);
        let rgb = self
            .new_desc
            .lock()
            .unwrap()
            .unwrap_or(self.init_settings.player_png_desc)
            .colors
            .player_main;
        state.try_ws_write_option(
            "mina_color",
            rgb[0] as u32 + ((rgb[1] as u32) << 8) + ((rgb[2] as u32) << 16),
        );

        let rgb = self
            .new_desc
            .lock()
            .unwrap()
            .unwrap_or(self.init_settings.player_png_desc)
            .colors
            .player_alt;
        state.try_ws_write_option(
            "mina_color_alt",
            rgb[0] as u32 + ((rgb[1] as u32) << 8) + ((rgb[2] as u32) << 16),
        );

        let progress = settings.progress.join(",");
        state.try_ws_write_option("progress", progress.as_str());

        state.try_ms_write(&NoitaInbound::Ready {
            my_peer_id: self.peer.my_id().into(),
        });
        info!("Settings sent")
    }

    fn handle_mod_message_2(&self, msg: NoitaOutbound, state: &mut NetInnerState) {
        match msg {
            NoitaOutbound::Raw(raw_msg) => {
                match raw_msg[0] & 0b11 {
                    // Message to proxy
                    1 => {
                        self.handle_message_to_proxy(&raw_msg[1..], state);
                    }
                    // Broadcast
                    2 => {
                        let msg_to_send = NetMsg::ModRaw {
                            data: raw_msg[1..].to_owned(),
                        };
                        let reliable = raw_msg[0] & 4 > 0;
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
                    3 => self.handle_bin_message_to_proxy(&raw_msg[1..], state),
                    msg_variant => {
                        error!("Unknown msg variant from mod: {}", msg_variant)
                    }
                }
            }
            NoitaOutbound::DesToProxy(des_to_proxy) => {
                if self.is_host() {
                    state.des.handle_noita_msg(self.peer.my_id(), des_to_proxy)
                } else {
                    self.send(
                        self.peer.host_id(),
                        &NetMsg::ForwardDesToProxy(des_to_proxy),
                        Reliability::Reliable,
                    );
                }
            }
            NoitaOutbound::RemoteMessage {
                reliable,
                destination,
                message,
            } => {
                let destination = destination.convert::<OmniPeerId>();
                let reliability = Reliability::from_reliability_bool(reliable);
                match destination {
                    Destination::Peer(peer) => {
                        self.send(peer, &NetMsg::RemoteMsg(message), reliability)
                    }
                    Destination::Host => self.send(
                        self.peer.host_id(),
                        &NetMsg::RemoteMsg(message),
                        reliability,
                    ),
                    Destination::Broadcast => {
                        self.broadcast(&NetMsg::RemoteMsg(message), reliability)
                    }
                }
            }
        }
    }

    pub fn start(self: Arc<NetManager>, player_path: PathBuf) -> JoinHandle<()> {
        info!("Starting netmanager");
        thread::spawn(move || {
            let result = self.clone().start_inner(player_path, false);
            if let Err(err) = result {
                error!("Error in netmanager: {}", err);
                *self.error.lock().unwrap() = Some(err);
            }
            self.stopped.store(true, Ordering::Relaxed);
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
                    self.end_run(state)
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
            Some("reset_world") => {
                state.world.reset();
                state.des.reset();
            }
            Some("material_list") => {
                state.world.materials.clear();
                while let (
                    Some(i),
                    Some(d),
                    Some(h),
                    Some(cell_type),
                    Some(liquid_sand),
                    Some(liquid_static),
                ) = (
                    msg.next().and_then(|s| s.parse().ok()),
                    msg.next().and_then(|s| s.parse().ok()),
                    msg.next().and_then(|s| s.parse().ok()),
                    msg.next(),
                    msg.next().map(|s| s == "1"),
                    msg.next().map(|s| s == "1"),
                ) {
                    state.world.materials.insert(
                        i,
                        (d, h, CellType::new(cell_type, liquid_static, liquid_sand)),
                    );
                }
                let c = msg.count();
                if c != 0 {
                    error!("bad materials data {}", c);
                }
            }
            Some("cut_through_world") => {
                let x: Option<i32> = msg.next().and_then(|s| s.parse().ok());
                let y_min: Option<i32> = msg.next().and_then(|s| s.parse().ok());
                let y_max: Option<i32> = msg.next().and_then(|s| s.parse().ok());
                let radius: Option<i32> = msg.next().and_then(|s| s.parse().ok());
                let (Some(x), Some(y_min), Some(y_max), Some(radius)) = (x, y_min, y_max, radius)
                else {
                    error!("Missing arguments in cut_through_world message");
                    return;
                };

                state.world.cut_through_world(x, y_min, y_max, radius);
            }
            Some("cut_through_world_line") => {
                let x: Option<i32> = msg.next().and_then(|s| s.parse().ok());
                let lx: Option<i32> = msg.next().and_then(|s| s.parse().ok());
                let y: Option<i32> = msg.next().and_then(|s| s.parse().ok());
                let ly: Option<i32> = msg.next().and_then(|s| s.parse().ok());
                let (Some(x), Some(y), Some(lx), Some(ly)) = (x, y, lx, ly) else {
                    error!("Missing arguments in cut_through_world_line message");
                    return;
                };
                let r: Option<i32> = msg.next().and_then(|s| s.parse().ok());
                let chance: Option<u64> = msg.next().and_then(|s| s.parse().ok());
                state.world.cut_through_world_line(
                    x,
                    y,
                    lx,
                    ly,
                    r.unwrap_or(12),
                    chance.unwrap_or(100).min(100) as u8,
                );
            }
            Some("cut_through_world_circle") => {
                let x: Option<i32> = msg.next().and_then(|s| s.parse().ok());
                let y: Option<i32> = msg.next().and_then(|s| s.parse().ok());
                let r: Option<i32> = msg.next().and_then(|s| s.parse().ok());
                let mat: Option<u16> = msg.next().and_then(|s| s.parse().ok());
                let chance: u64 = msg
                    .next()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(100);
                let (Some(x), Some(y), Some(r)) = (x, y, r) else {
                    error!("Missing arguments in cut_through_world_circle message");
                    return;
                };
                state
                    .world
                    .cut_through_world_circle(x, y, r, mat, chance.min(100) as u8);
            }
            Some("cut_through_world_explosion") => {
                let x: Option<i32> = msg.next().and_then(|s| s.parse().ok());
                let y: Option<i32> = msg.next().and_then(|s| s.parse().ok());
                let r: Option<u64> = msg.next().and_then(|s| s.parse().ok());
                let d: Option<u32> = msg.next().and_then(|s| s.parse().ok());
                let ray: Option<u64> = msg.next().and_then(|s| s.parse().ok());
                let hole: Option<bool> = msg.next().and_then(|s| s.parse().ok());
                let liquid: Option<bool> = msg.next().and_then(|s| s.parse().ok());
                let mat: Option<u16> = msg.next().and_then(|s| s.parse().ok());
                let prob: Option<u64> = msg.next().and_then(|s| s.parse().ok());
                let (
                    Some(x),
                    Some(y),
                    Some(r),
                    Some(d),
                    Some(ray),
                    Some(hole),
                    Some(liquid),
                    Some(mat),
                    Some(prob),
                ) = (x, y, r, d, ray, hole, liquid, mat, prob)
                else {
                    error!("Missing arguments in cut_through_world_expl message");
                    return;
                };
                state.explosion_data.push(ExplosionData::new(
                    x,
                    y,
                    r,
                    d,
                    ray,
                    hole,
                    liquid,
                    mat,
                    prob.min(100) as u8,
                ));
            }
            Some("flush_exp") => {
                state
                    .world
                    .cut_through_world_explosion(std::mem::take(&mut state.explosion_data));
            }
            Some("flush") => self.peer.flush(),
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
                let pos = data[1..]
                    .split(|b| *b == b':')
                    .map(|s| String::from_utf8_lossy(s).parse::<i32>().unwrap_or(0))
                    .collect::<Vec<i32>>();
                state.world.add_end(data[0], &pos);
            }
            key => {
                error!("Unknown bin msg from mod: {:?}", key)
            }
        }
    }

    fn end_run(&self, state: &mut NetInnerState) {
        self.init_settings.save_state.reset();
        {
            let mut settings = self.pending_settings.lock().unwrap();
            if !settings.use_constant_seed {
                settings.seed = rand::random();
            }
            info!("New seed: {}", settings.seed);
            settings.progress = self
                .init_settings
                .modmanager_settings
                .get_progress()
                .unwrap_or_default();
            if settings.world_num == u16::MAX {
                settings.world_num = 0
            } else {
                settings.world_num += 1
            }
            *self.settings.lock().unwrap() = settings.clone();
            state.world.reset();
            state.des.reset();
            self.dirty.store(false, Ordering::Relaxed);
        }
        self.resend_game_settings();
    }
}

#[derive(Clone, Copy)]
pub struct ExplosionData {
    x: i32,
    y: i32,
    r: u64,
    d: u32,
    ray: u64,
    hole: bool,
    liquid: bool,
    mat: Pixel,
    prob: u8,
}
impl ExplosionData {
    #[allow(clippy::too_many_arguments)]
    fn new(
        x: i32,
        y: i32,
        r: u64,
        d: u32,
        ray: u64,
        hole: bool,
        liquid: bool,
        mat: u16,
        prob: u8,
    ) -> ExplosionData {
        ExplosionData {
            x,
            y,
            r,
            d,
            ray,
            hole,
            liquid,
            mat: Pixel {
                flags: PixelFlags::Normal,
                material: mat,
            },
            prob,
        }
    }
}

pub enum CellType {
    Solid,
    Liquid(LiquidType),
    Gas,
    Fire,
    Invalid,
}
impl CellType {
    fn new(s: &str, stat: bool, sand: bool) -> Self {
        match s {
            "solid" => Self::Solid,
            "liquid" if stat => Self::Liquid(LiquidType::Static),
            "liquid" if sand => Self::Liquid(LiquidType::Sand),
            "liquid" => Self::Liquid(LiquidType::Liquid),
            "gas" => Self::Gas,
            "fire" => Self::Fire,
            _ => Self::Invalid,
        }
    }
    fn can_remove(&self, hole: bool, liquid: bool) -> bool {
        match self {
            Self::Liquid(LiquidType::Sand) | Self::Liquid(LiquidType::Static) | Self::Solid
                if hole =>
            {
                true
            }
            Self::Liquid(LiquidType::Liquid) if liquid => true,
            _ => false,
        }
    }
}
pub enum LiquidType {
    Static,
    Liquid,
    Sand,
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
