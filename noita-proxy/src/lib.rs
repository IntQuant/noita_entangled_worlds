use std::{
    env, fmt::Display, net::SocketAddr, sync::{atomic::Ordering, Arc}, thread, time::Duration
};

use bitcode::{Decode, Encode};
use clipboard::{ClipboardContext, ClipboardProvider};
use eframe::egui::{self, Color32, Layout};
use steamworks::{LobbyId, SteamAPIInitError};
use tangled::Peer;
use tracing::info;

pub mod messages;

#[derive(Debug, Decode, Encode, Clone)]
pub struct GameSettings {
    seed: u64,
    debug_mode: bool,
}

pub mod net;

enum AppState {
    Init,
    Netman { netman: Arc<net::NetManager> },
    Error { message: String },
}

struct SteamState {
    pub client: steamworks::Client,
}

impl SteamState {
    fn new() -> Result<Self, SteamAPIInitError> {
        if env::var_os("NP_DISABLE_STEAM").is_some() {
            return Err(SteamAPIInitError::FailedGeneric("Disabled by env variable".to_string()))
        }
        let (client, single) = steamworks::Client::init_app(881100)?;
        thread::spawn(move || {
            info!("Spawned steam callback thread");
            loop {
                single.run_callbacks();
                thread::sleep(Duration::from_millis(3));
            }
        });
        Ok(SteamState { client })
    }
}

pub struct App {
    state: AppState,
    steam_state: Result<SteamState, SteamAPIInitError>,
    addr: String,
    debug_mode: bool,
    use_constant_seed: bool,
}

impl App {
    fn start_server(&mut self) {
        let bind_addr = "0.0.0.0:5123".parse().unwrap();
        let peer = Peer::host(bind_addr, None).unwrap();
        let netman = net::NetManager::new(net::PeerVariant::Tangled(peer));
        self.set_netman_settings(&netman);
        netman.clone().start();
        self.state = AppState::Netman { netman };
    }

    fn set_netman_settings(&mut self, netman: &Arc<net::NetManager>) {
        let mut settings = netman.settings.lock().unwrap();
        settings.debug_mode = self.debug_mode;
        if !self.use_constant_seed {
            settings.seed = rand::random();
        }
        netman.accept_local.store(true, Ordering::SeqCst);
    }
    fn start_connect(&mut self, addr: SocketAddr) {
        let peer = Peer::connect(addr, None).unwrap();
        let netman = net::NetManager::new(net::PeerVariant::Tangled(peer));
        netman.clone().start();
        self.state = AppState::Netman { netman };
    }

    fn start_steam_host(&mut self) {
        let peer = net::steam_networking::SteamPeer::new_host(
            steamworks::LobbyType::Private,
            self.steam_state.as_ref().unwrap().client.clone(),
        );
        let netman = net::NetManager::new(net::PeerVariant::Steam(peer));
        self.set_netman_settings(&netman);
        netman.clone().start();
        self.state = AppState::Netman { netman };
    }

    fn notify_error(&mut self, error: impl Display) {
        self.state = AppState::Error {
            message: error.to_string(),
        }
    }

    fn start_steam_connect(&mut self, id: LobbyId) {
        let peer = net::steam_networking::SteamPeer::new_connect(
            id,
            self.steam_state.as_ref().unwrap().client.clone(),
        );
        let netman = net::NetManager::new(net::PeerVariant::Steam(peer));
        netman.clone().start();
        self.state = AppState::Netman { netman };
    }
}

impl Default for App {
    fn default() -> Self {
        info!("Creating the app...");
        Self {
            state: AppState::Init,
            addr: "127.0.0.1:5123".to_string(),
            debug_mode: false,
            use_constant_seed: false,
            steam_state: SteamState::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_secs(1));
        match &self.state {
            AppState::Init => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("Noita Entangled Worlds proxy");
                    ui.checkbox(&mut self.debug_mode, "Debug mode");
                    ui.checkbox(&mut self.use_constant_seed, "Use specified seed");
                    if ui.button("Host").clicked() {
                        self.start_server();
                    }
                    ui.separator();
                    ui.text_edit_singleline(&mut self.addr);
                    let addr = self.addr.parse();
                    ui.add_enabled_ui(addr.is_ok(), |ui| {
                        if ui.button("Connect").clicked() {
                            if let Ok(addr) = addr {
                                self.start_connect(addr);
                            }
                        }
                    });
                    ui.separator();
                    ui.heading("Steam networking");
                    match &self.steam_state {
                        Ok(_) => {
                            if ui.button("Create lobby").clicked() {
                                self.start_steam_host();
                            }
                            if ui.button("Connect to lobby in clipboard").clicked() {
                                let id = ClipboardProvider::new()
                                    .and_then(|mut ctx: ClipboardContext| ctx.get_contents());
                                match id {
                                    Ok(id) => {
                                        let id = id.parse().map(LobbyId::from_raw);
                                        match id {
                                            Ok(id) => self.start_steam_connect(id),
                                            Err(error) => self.notify_error(error),
                                        }
                                    }
                                    Err(error) => self.notify_error(error),
                                }
                            }
                        }
                        Err(err) => {
                            ui.label(format!("Could not init steam networking: {}", err));
                        }
                    }
                    ui.with_layout(Layout::right_to_left(egui::Align::Max), |ui| {
                        ui.label(concat!("Noita Proxy version ", env!("CARGO_PKG_VERSION")))
                    })
                });
            }
            AppState::Netman { netman } => {
                let stopped = netman.stopped.load(Ordering::Relaxed);
                let accept_local = netman.accept_local.load(Ordering::Relaxed);
                let local_connected = netman.local_connected.load(Ordering::Relaxed);
                egui::CentralPanel::default().show(ctx, |ui| {
                    if stopped {
                        ui.colored_label(Color32::LIGHT_RED, "Netmanager thread has stopped");
                        if let Some(err) = netman.error.lock().unwrap().as_ref() {
                            ui.label("With the following error:");
                            ui.label(err.to_string());
                        }
                        ui.separator();
                    }
                    
                    if accept_local {
                        if local_connected {
                            ui.colored_label(Color32::GREEN, "Local Noita instance connected");
                        } else {
                            ui.colored_label(Color32::YELLOW, "Awaiting Noita connection. It's time to start new game in Noita now!");
                        }
                    } else {
                        ui.label("Not yet ready");
                    }
                    ui.separator();
                    if let Some(id) = netman.peer.lobby_id() {
                        if ui.button("Save lobby id to clipboard").clicked() {
                            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                            let _ = ctx.set_contents(id.raw().to_string());
                        }
                    } else {
                        ui.label("Lobby id not available");
                    }
                    ui.heading("Current users");
                    for peer in netman.peer.iter_peer_ids() {
                        ui.label(peer.to_string());
                    }
                    ui.label(format!("Peer state: {}", netman.peer.state()));
                });
            }
            AppState::Error { message } => {
                if egui::CentralPanel::default()
                    .show(ctx, |ui| {
                        ui.heading("An error occured:");
                        ui.label(message);
                        ui.button("Back").clicked()
                    })
                    .inner
                {
                    self.state = AppState::Init;
                }
            }
        };
    }
}
