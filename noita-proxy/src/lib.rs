use std::{net::SocketAddr, sync::Arc, time::Duration};

use bitcode::{Decode, Encode};
use eframe::egui::{self, Color32};
use tangled::Peer;

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
}

pub struct App {
    state: AppState,
    addr: String,
    debug_mode: bool,
    use_constant_seed: bool,
}

impl App {
    fn start_server(&mut self) {
        let bind_addr = "0.0.0.0:5123".parse().unwrap();
        let peer = Peer::host(bind_addr, None).unwrap();
        let netman = net::NetManager::new(peer);
        {
            let mut settings = netman.settings.lock().unwrap();
            settings.debug_mode = self.debug_mode;
            if !self.use_constant_seed {
                settings.seed = rand::random();
            }
        }
        netman
            .accept_local
            .store(true, std::sync::atomic::Ordering::SeqCst);
        netman.clone().start();
        self.state = AppState::Netman { netman };
    }
    fn start_connect(&mut self, addr: SocketAddr) {
        let peer = Peer::connect(addr, None).unwrap();
        let netman = net::NetManager::new(peer);
        netman.clone().start();
        self.state = AppState::Netman { netman };
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: AppState::Init,
            addr: "192.168.1.168:5123".to_string(),
            debug_mode: false,
            use_constant_seed: false,
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
                });
            }
            AppState::Netman { netman } => {
                let accept_local = netman
                    .accept_local
                    .load(std::sync::atomic::Ordering::Relaxed);
                let local_connected = netman
                    .local_connected
                    .load(std::sync::atomic::Ordering::Relaxed);
                egui::CentralPanel::default().show(ctx, |ui| {
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
                    ui.heading("Current users");
                    for peer in netman.peer.iter_peer_ids() {
                        ui.label(peer.0.to_string());
                    }
                    ui.label(format!("Peer state: {}", netman.peer.state()));
                });
            }
        };
    }
}
