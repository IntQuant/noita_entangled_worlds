use std::{
    fmt::Display,
    io::{self, Write},
    net::TcpListener,
    sync::{atomic::AtomicBool, Arc, Mutex},
    thread,
    time::Duration,
};

use bitcode::{Decode, Encode};
use eframe::egui::{self, Color32};
use tangled::{Peer, PeerId, Reliability};
use tracing::{error, info};
use tungstenite::accept;

use crate::messages::NetMsg;

pub mod messages;

static HOST: PeerId = PeerId(0);

#[derive(Decode, Encode, Clone)]
pub struct GameSettings {
    seed: u64,
}

fn ws_encode_proxy(key: &'static str, value: impl Display) -> tungstenite::Message {
    let mut buf = Vec::new();
    buf.push(2);
    write!(buf, "{} {}", key, value).unwrap();
    tungstenite::Message::Binary(buf)
}

fn ws_encode_mod(peer: PeerId, data: &[u8]) -> tungstenite::Message {
    let mut buf = Vec::new();
    buf.push(1u8);
    buf.extend_from_slice(&peer.0.to_le_bytes());
    buf.extend_from_slice(data);
    tungstenite::Message::Binary(buf)
}

struct NetManager {
    peer: tangled::Peer,
    settings: Mutex<GameSettings>,
    continue_running: AtomicBool, // TODO stop on drop
    accept_local: AtomicBool,
    local_connected: AtomicBool,
}

impl NetManager {
    pub fn new(peer: tangled::Peer) -> Arc<Self> {
        Self {
            peer,
            settings: Mutex::new(GameSettings { seed: 1663107061 }),
            continue_running: AtomicBool::new(true),
            accept_local: AtomicBool::new(false),
            local_connected: AtomicBool::new(false),
        }
        .into()
    }

    fn send(&self, peer: tangled::PeerId, msg: &NetMsg, reliability: Reliability) {
        let encoded = bitcode::encode(msg);
        self.peer.send(peer, encoded.clone(), reliability).ok(); // TODO log
    }

    fn broadcast(&self, msg: &NetMsg, reliability: Reliability) {
        let encoded = bitcode::encode(msg);
        self.peer.broadcast(encoded, reliability).ok(); // TODO log
    }

    pub fn start(self: Arc<NetManager>) {
        info!("Starting netmanager");
        thread::spawn(move || {
            let local_server = TcpListener::bind("127.0.0.1:41251").unwrap();
            // let local_server = TcpListener::bind("127.0.0.1:0").unwrap();
            local_server
                .set_nonblocking(true)
                .expect("can set nonblocking");

            let mut websocket = None;

            while self
                .continue_running
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                self.local_connected
                    .store(websocket.is_some(), std::sync::atomic::Ordering::Relaxed);
                if websocket.is_none()
                    && self.accept_local.load(std::sync::atomic::Ordering::SeqCst)
                {
                    thread::sleep(Duration::from_millis(10));
                    if let Ok((stream, addr)) = local_server.accept() {
                        info!("New stream incoming from {}", addr);
                        stream.set_nodelay(true).ok();
                        stream
                            .set_read_timeout(Some(Duration::from_millis(1)))
                            .expect("can set read timeout");

                        websocket = accept(stream).ok();
                        if websocket.is_some() {
                            info!("New stream connected");
                            if let Some(ws) = &mut websocket {
                                let settings = self.settings.lock().unwrap();
                                ws.write(ws_encode_proxy("seed", settings.seed)).ok();
                                ws.write(ws_encode_proxy("name", "test_name")).ok();
                                // TODO?
                                for id in self.peer.iter_peer_ids() {
                                    ws.write(ws_encode_proxy("join", id)).ok();
                                }
                            }
                        }
                    }
                }
                if let Some(ws) = &mut websocket {
                    ws.flush().ok();
                }
                for net_event in self.peer.recv() {
                    match net_event {
                        tangled::NetworkEvent::PeerConnected(id) => {
                            info!("Peer connected");
                            if self.peer.my_id() == Some(HOST) {
                                info!("Sending start game message");
                                self.send(
                                    id,
                                    &NetMsg::StartGame {
                                        settings: self.settings.lock().unwrap().clone(),
                                    },
                                    tangled::Reliability::Reliable,
                                );
                            }
                            if let Some(ws) = &mut websocket {
                                ws.write(ws_encode_proxy("join", id)).ok();
                            }
                        }
                        tangled::NetworkEvent::PeerDisconnected(id) => {
                            if let Some(ws) = &mut websocket {
                                ws.write(ws_encode_proxy("leave", id)).ok();
                            }
                        }
                        tangled::NetworkEvent::Message(msg) => {
                            let Ok(net_msg) = bitcode::decode::<NetMsg>(&msg.data) else {
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
                                    if let Some(ws) = &mut websocket {
                                        if let Err(err) = ws.write(ws_encode_mod(msg.src, &data)) {
                                            error!(
                                                "Error occured while sending to websocket: {}",
                                                err
                                            );
                                            websocket = None;
                                        };
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(ws) = &mut websocket {
                    let msg = ws.read();
                    match msg {
                        Ok(msg) => match msg {
                            tungstenite::Message::Binary(msg) => self
                                .broadcast(&NetMsg::ModRaw { data: msg }, Reliability::Unreliable),
                            _ => {}
                        },
                        Err(tungstenite::Error::Io(io_err))
                            if io_err.kind() == io::ErrorKind::WouldBlock => {}
                        Err(err) => {
                            error!("Error occured while reading from websocket: {}", err);
                            websocket = None;
                        }
                    }
                }
            }
        });
    }
}

enum AppState {
    Init,
    Netman { netman: Arc<NetManager> },
}

pub struct App {
    state: AppState,
}

impl App {
    fn start_server(&mut self) {
        let peer = Peer::host("0.0.0.0:5123".parse().unwrap(), None).unwrap();
        let netman = NetManager::new(peer);
        netman
            .accept_local
            .store(true, std::sync::atomic::Ordering::SeqCst);
        netman.clone().start();
        self.state = AppState::Netman { netman };
    }
    fn start_connect(&mut self) {
        let peer = Peer::connect("127.0.0.1:5123".parse().unwrap(), None).unwrap();
        let netman = NetManager::new(peer);
        netman.clone().start();
        self.state = AppState::Netman { netman };
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: AppState::Init,
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
                    if ui.button("Host").clicked() {
                        self.start_server();
                    }
                    if ui.button("Connect").clicked() {
                        self.start_connect();
                    }
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
                }});
            }
        };
    }
}
