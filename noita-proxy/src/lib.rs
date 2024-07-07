use std::{
    fmt::Display,
    net::SocketAddr,
    sync::{atomic::Ordering, Arc},
    time::Duration,
};

use bitcode::{Decode, Encode};
use bookkeeping::noita_launcher::{LaunchTokenResult, NoitaLauncher};
use clipboard::{ClipboardContext, ClipboardProvider};
use eframe::egui::{
    self, Align2, Button, Color32, DragValue, InnerResponse, Key, Margin, OpenUrl, Rect, RichText,
    ScrollArea, Slider, TextureOptions, Ui, Vec2,
};
use egui_plot::{Plot, PlotPoint, PlotUi, Text};
use lang::{set_current_locale, tr, LANGS};
use mod_manager::{Modmanager, ModmanagerSettings};
use net::{omni::PeerVariant, steam_networking::ExtraPeerState, NetManagerInit};
use self_update::SelfUpdateManager;
use serde::{Deserialize, Serialize};
use steamworks::{LobbyId, SteamAPIInitError};
use tangled::Peer;
use tracing::info;
use unic_langid::LanguageIdentifier;

mod util;
use util::args::Args;
pub use util::{args, lang, steam_helper};

mod bookkeeping;
pub use bookkeeping::{mod_manager, releases, self_update};

pub mod net;

#[derive(Debug, Decode, Encode, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    seed: u64,
    debug_mode: bool,
    world_sync_version: u32,
    player_tether: bool,
    tether_length: u32,
}

impl Default for GameSettings {
    fn default() -> Self {
        GameSettings {
            seed: 0,
            debug_mode: false,
            world_sync_version: 2,
            player_tether: false,
            tether_length: 750,
        }
    }
}

enum AppState {
    Connect,
    ModManager,
    Netman {
        netman: Arc<net::NetManager>,
        noita_launcher: NoitaLauncher,
    },
    Error {
        message: String,
    },
    SelfUpdate,
    LangPick,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppSavedState {
    addr: String,
    use_constant_seed: bool,
    nickname: Option<String>,
    times_started: u32,
    lang_id: Option<LanguageIdentifier>,
    constant_seed: u64,
    game_settings: GameSettings,
    start_game_automatically: bool,
}

impl Default for AppSavedState {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:5123".to_string(),
            use_constant_seed: false,
            nickname: None,
            times_started: 0,
            lang_id: None,
            constant_seed: 0,
            game_settings: GameSettings::default(),
            start_game_automatically: false,
        }
    }
}

pub struct App {
    state: AppState,
    modmanager: Modmanager,
    steam_state: Result<steam_helper::SteamState, SteamAPIInitError>,
    saved_state: AppSavedState,
    modmanager_settings: ModmanagerSettings,
    self_update: SelfUpdateManager,
    show_map_plot: bool,
    lobby_id_field: String,
    args: Args,
    can_start_automatically: bool,
}

const MODMANAGER: &str = "modman";

fn filled_group<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    let style = ui.style();
    let frame = egui::Frame {
        inner_margin: Margin::same(6.0), // same and symmetric looks best in corners when nesting groups
        rounding: style.visuals.widgets.noninteractive.rounding,
        stroke: style.visuals.widgets.noninteractive.bg_stroke,
        fill: Color32::from_rgba_premultiplied(20, 20, 20, 180),
        ..Default::default()
    };
    frame.show(ui, add_contents)
}

fn heading_with_underline(ui: &mut Ui, text: impl Into<RichText>) {
    ui.vertical_centered_justified(|ui| {
        ui.heading(text);
    });
    ui.separator();
}

fn square_button_text(ui: &mut Ui, text: &str, size: f32) -> egui::Response {
    let side = ui.available_width();
    ui.add_sized([side, side], Button::new(RichText::new(text).size(size)))
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, args: Args) -> Self {
        let mut saved_state: AppSavedState = cc
            .storage
            .and_then(|storage| eframe::get_value(storage, eframe::APP_KEY))
            .unwrap_or_default();
        let modmanager_settings = cc
            .storage
            .and_then(|storage| eframe::get_value(storage, MODMANAGER))
            .unwrap_or_default();
        saved_state.times_started += 1;

        let state = if let Some(lang_id) = &saved_state.lang_id {
            set_current_locale(lang_id.clone());
            AppState::ModManager
        } else {
            AppState::LangPick
        };

        egui_extras::install_image_loaders(&cc.egui_ctx);

        cc.egui_ctx
            .set_zoom_factor(args.ui_zoom_factor.unwrap_or(1.0));
        info!("Creating the app...");
        Self {
            state,
            modmanager: Modmanager::default(),
            steam_state: steam_helper::SteamState::new(),
            saved_state,
            modmanager_settings,
            self_update: SelfUpdateManager::new(),
            show_map_plot: false,
            lobby_id_field: "".to_string(),
            args,
            can_start_automatically: false,
        }
    }

    fn get_netman_init(&self) -> NetManagerInit {
        let steam_nickname = if let Ok(steam) = &self.steam_state {
            Some(steam.get_user_name(steam.get_my_id()))
        } else {
            None
        };
        let my_nickname = self.saved_state.nickname.clone().or(steam_nickname);
        NetManagerInit { my_nickname }
    }

    fn change_state_to_netman(&mut self, netman: Arc<net::NetManager>) {
        self.state = AppState::Netman {
            netman,
            noita_launcher: NoitaLauncher::new(
                &self.modmanager_settings.game_exe_path,
                self.args.launch_cmd.as_ref().map(|x| x.as_str()),
            ),
        };
        self.can_start_automatically = true;
    }

    fn start_server(&mut self) {
        let bind_addr = "0.0.0.0:5123".parse().unwrap();
        let peer = Peer::host(bind_addr, None).unwrap();
        let netman = net::NetManager::new(PeerVariant::Tangled(peer), self.get_netman_init());
        self.set_netman_settings(&netman);
        netman.clone().start();
        self.change_state_to_netman(netman);
    }

    fn set_netman_settings(&mut self, netman: &Arc<net::NetManager>) {
        let mut settings = netman.settings.lock().unwrap();
        *settings = self.saved_state.game_settings.clone();
        if !self.saved_state.use_constant_seed {
            settings.seed = rand::random();
        } else {
            settings.seed = self.saved_state.constant_seed;
            info!("Using constant seed: {}", settings.seed);
        }
        netman.accept_local.store(true, Ordering::SeqCst);
    }
    fn start_connect(&mut self, addr: SocketAddr) {
        let peer = Peer::connect(addr, None).unwrap();
        let netman = net::NetManager::new(PeerVariant::Tangled(peer), self.get_netman_init());
        netman.clone().start();
        self.change_state_to_netman(netman);
    }

    fn start_steam_host(&mut self) {
        let peer = net::steam_networking::SteamPeer::new_host(
            steamworks::LobbyType::Private,
            self.steam_state.as_ref().unwrap().client.clone(),
        );
        let netman = net::NetManager::new(PeerVariant::Steam(peer), self.get_netman_init());
        self.set_netman_settings(&netman);
        netman.clone().start();
        self.change_state_to_netman(netman);
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

        let netman = net::NetManager::new(PeerVariant::Steam(peer), self.get_netman_init());
        netman.clone().start();
        self.change_state_to_netman(netman);
    }

    fn connect_screen(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.saved_state.times_started % 20 == 0 {
                let image = egui::Image::new(egui::include_image!("../assets/longleg.png"))
                    .texture_options(TextureOptions::NEAREST);
                image.paint_at(ui, ui.ctx().screen_rect());
            } else {
                draw_bg(ui);
            }

            let group_shrink = ui.spacing().item_spacing.x * 0.5;
            let rect = ui.max_rect();
            let (rect, bottom_panel) =
                rect.split_top_bottom_at_y(rect.height() - (25.0 + group_shrink * 2.0));

            let (rect, right_b_panel) =
                rect.split_left_right_at_x(rect.width() - (50.0 + group_shrink * 2.0));
            let (settings_rect, right) = rect.split_left_right_at_fraction(0.5);
            let (steam_connect_rect, ip_connect_rect) = right.split_top_bottom_at_fraction(0.5);

            ui.allocate_ui_at_rect(bottom_panel.shrink(group_shrink), |ui| {
                filled_group(ui, |ui| {
                    ui.set_min_size(ui.available_size());
                    self.self_update.display_version(ui);

                    if self.self_update.request_update {
                        self.state = AppState::SelfUpdate;
                    }
                });
            });

            ui.allocate_ui_at_rect(right_b_panel.shrink(group_shrink), |ui| {
                filled_group(ui, |ui| {
                    ui.set_min_size(ui.available_size());

                    let lang_label = self
                        .saved_state
                        .lang_id
                        .clone()
                        .unwrap_or_default()
                        .language;
                    if square_button_text(ui, &lang_label.to_string().to_uppercase(), 21.0)
                        .on_hover_text(tr("button_set_lang"))
                        .clicked()
                    {
                        self.state = AppState::LangPick;
                    }
                    if square_button_text(ui, "Discord server", 10.0).clicked() {
                        ctx.open_url(OpenUrl::new_tab("https://discord.gg/uAK7utvVWN"));
                    }
                    let secret_active = ui.input(|i| i.modifiers.ctrl && i.key_down(Key::D));
                    if secret_active && ui.button("reset all data").clicked() {
                        self.saved_state = Default::default();
                        self.modmanager_settings = Default::default();
                        self.state = AppState::LangPick;
                    }
                })
            });

            ui.allocate_ui_at_rect(settings_rect.shrink(group_shrink), |ui| {
                filled_group(ui, |ui| {
                    ui.set_min_size(ui.available_size());
                    heading_with_underline(ui, tr("connect_settings"));

                    ui.label(tr("connect_settings_debug"));
                    ui.checkbox(
                        &mut self.saved_state.game_settings.debug_mode,
                        tr("connect_settings_debug_en"),
                    );
                    ui.checkbox(
                        &mut self.saved_state.use_constant_seed,
                        tr("connect_settings_debug_fixed_seed"),
                    );

                    ui.horizontal(|ui| {
                        ui.label(tr("connect_settings_seed"));
                        ui.add(DragValue::new(&mut self.saved_state.constant_seed));
                    });

                    ui.add_space(20.0);

                    ui.label(tr("connect_settings_wsv"));
                    ui.horizontal(|ui| {
                        ui.radio_value(
                            &mut self.saved_state.game_settings.world_sync_version,
                            1,
                            "v1",
                        );
                        ui.radio_value(
                            &mut self.saved_state.game_settings.world_sync_version,
                            2,
                            "v2",
                        );
                    });

                    ui.add_space(20.0);

                    ui.label(tr("connect_settings_player_tether_desc"));
                    ui.checkbox(
                        &mut self.saved_state.game_settings.player_tether,
                        tr("connect_settings_player_tether"),
                    );
                    ui.add(
                        Slider::new(&mut self.saved_state.game_settings.tether_length, 10..=5000)
                            .text(tr("connect_settings_player_tether_length")),
                    );

                    heading_with_underline(ui, tr("connect_settings_local"));
                    ui.checkbox(
                        &mut self.saved_state.start_game_automatically,
                        tr("connect_settings_autostart"),
                    );
                });
            });
            ui.allocate_ui_at_rect(steam_connect_rect.shrink(group_shrink), |ui| {
                filled_group(ui, |ui| {
                    ui.set_min_size(ui.available_size());

                    heading_with_underline(ui, tr("connect_steam"));

                    match &self.steam_state {
                        Ok(_) => {
                            if ui.button(tr("connect_steam_create")).clicked() {
                                self.start_steam_host();
                            }
                            if ui.button(tr("connect_steam_connect")).clicked() {
                                let id = ClipboardProvider::new()
                                    .and_then(|mut ctx: ClipboardContext| ctx.get_contents());
                                match id {
                                    Ok(id) => {
                                        self.connect_to_steam_lobby(id);
                                    }
                                    Err(error) => self.notify_error(error),
                                }
                            }

                            if cfg!(target_os = "linux") {
                                ui.add_space(30.0);
                                ui.label(tr("connect_steam_workaround_label"));
                                ui.text_edit_singleline(&mut self.lobby_id_field);
                                if ui.button(tr("connect_steam_connect_2")).clicked() {
                                    self.connect_to_steam_lobby(self.lobby_id_field.clone());
                                }
                            }
                        }
                        Err(err) => {
                            ui.label(format!("Could not init steam networking: {}", err));
                        }
                    }
                });
            });
            ui.allocate_ui_at_rect(ip_connect_rect.shrink(group_shrink), |ui| {
                filled_group(ui, |ui| {
                    ui.set_min_size(ui.available_size());

                    heading_with_underline(ui, tr("connect_ip"));

                    ui.label(tr("ip_note"));
                    if ui.button(tr("ip_host")).clicked() {
                        self.start_server();
                    }

                    ui.text_edit_singleline(&mut self.saved_state.addr);
                    let addr = self.saved_state.addr.parse();
                    ui.add_enabled_ui(addr.is_ok(), |ui| {
                        if ui.button(tr("ip_connect")).clicked() {
                            if let Ok(addr) = addr {
                                self.start_connect(addr);
                            }
                        }
                    });
                });
            });
        });
    }

    fn connect_to_steam_lobby(&mut self, lobby_id: String) {
        let id = lobby_id.trim().parse().map(LobbyId::from_raw);
        match id {
            Ok(id) => self.start_steam_connect(id),
            Err(_error) => self.notify_error(tr("connect_steam_connect_invalid_lobby_id")),
        }
    }
}

fn draw_bg(ui: &mut Ui) {
    let image = egui::Image::new(egui::include_image!("../assets/noita_ew_logo_sq.webp"))
        .texture_options(TextureOptions::NEAREST);

    let rect = ui.ctx().screen_rect();
    let aspect_ratio = 1.0;
    let new_height = f32::max(rect.width() * aspect_ratio, rect.height());
    let new_width = new_height / aspect_ratio;
    let rect = Rect::from_center_size(rect.center(), Vec2::new(new_width, new_height));

    image.paint_at(ui, rect);
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(500));
        match &mut self.state {
            AppState::Connect => {
                self.connect_screen(ctx);
            }
            AppState::Netman {
                netman,
                noita_launcher,
            } => {
                if let ExtraPeerState::CouldNotConnect(err) = netman.peer.state() {
                    self.notify_error(err);
                    return;
                }
                let stopped = netman.stopped.load(Ordering::Relaxed);
                let accept_local = netman.accept_local.load(Ordering::Relaxed);
                let local_connected = netman.local_connected.load(Ordering::Relaxed);
                egui::TopBottomPanel::top("noita_status").show(ctx, |ui| {
                    ui.add_space(3.0);
                    if accept_local {
                        if local_connected {
                            ui.colored_label(Color32::GREEN, tr("noita_connected"));
                        } else {
                            ui.colored_label(Color32::YELLOW, tr("noita_can_connect"));
                        }
                    } else {
                        ui.label(tr("noita_not_yet"));
                    }
                });
                egui::SidePanel::left("players")
                    .resizable(false)
                    .exact_width(200.0)
                    .show(ctx, |ui| {
                        ui.add_space(3.0);
                        if netman.peer.is_steam() {
                            let steam = self.steam_state.as_mut().expect(
                                "steam should be available, as we are using steam networking",
                            );
                            ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                                for peer in netman.peer.iter_peer_ids() {
                                    let role = peer_role(peer, netman);

                                    let username = steam.get_user_name(peer.into());
                                    let avatar = steam.get_avatar(ctx, peer.into());
                                    if let Some(avatar) = avatar {
                                        avatar.display_with_labels(ui, &username, &role);
                                        ui.add_space(5.0);
                                    } else {
                                        ui.label(&username);
                                    }
                                }
                            });
                        } else {
                            for peer in netman.peer.iter_peer_ids() {
                                ui.label(peer.to_string());
                            }
                        }
                    });
                egui::CentralPanel::default().show(ctx, |ui| {
                    if stopped {
                        ui.colored_label(Color32::LIGHT_RED, "Netmanager thread has stopped");
                        if let Some(err) = netman.error.lock().unwrap().as_ref() {
                            ui.label("With the following error:");
                            ui.label(err.to_string());
                        }
                        ui.separator();
                    }

                    if netman.peer.is_steam() {
                        if let Some(id) = netman.peer.lobby_id() {
                            if ui.button(tr("netman_save_lobby")).clicked() {
                                let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                                let _ = ctx.set_contents(id.raw().to_string());
                            }
                        }
                    } else {
                        ui.label(format!("Peer state: {:?}", netman.peer.state()));
                    }
                    ui.add_space(15.0);
                    if accept_local && !local_connected {
                        match noita_launcher.launch_token() {
                            LaunchTokenResult::Ok(mut token) => {
                                let start_auto = self.can_start_automatically && self.saved_state.start_game_automatically;
                                if start_auto || ui.button(tr("launcher_start_game")).clicked() {
                                    info!("Starting the game now");
                                    token.start_game();
                                    self.can_start_automatically = false;
                                }
                            },
                            LaunchTokenResult::AlreadyStarted => {
                                ui.label(tr("launcher_already_started"));
                            },
                            LaunchTokenResult::CantStart => {
                                ui.label(tr("launcher_no_command"));
                                ui.label(tr("launcher_no_command_2"));
                                ui.label(tr("launcher_no_command_3"));
                            },
                        }
                    } else {
                        ui.label(tr("launcher_only_when_awaiting"));
                    }
                    ui.add_space(15.0);

                    if self.show_map_plot {
                        let build_fn = |plot: &mut PlotUi| {
                            netman.world_info.with_player_infos(|peer, info| {
                                let username = if netman.peer.is_steam() {
                                    let steam = self.steam_state.as_mut().expect(
                                        "steam should be available, as we are using steam networking",
                                    );
                                    steam.get_user_name(peer.into())
                                } else {
                                    peer.to_hex()
                                };
                                plot.text(Text::new(PlotPoint::new(info.x, -info.y), username).highlight(true))
                            });
                        };
                        Plot::new("map").data_aspect(1.0).show(ui, build_fn);
                    } else if ui.button("Show debug plot").clicked() {
                        self.show_map_plot = true;
                    }
                });
            }
            AppState::Error { message } => {
                let add_contents = |ui: &mut Ui| {
                    ui.heading(tr("error_occured"));
                    ui.label(&*message);
                    ui.button(tr("button_back")).clicked()
                };
                if egui::CentralPanel::default().show(ctx, add_contents).inner {
                    self.state = AppState::Connect;
                }
            }
            AppState::ModManager => {
                egui::CentralPanel::default().show(ctx, draw_bg);
                egui::Window::new(tr("modman"))
                    .auto_sized()
                    .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        self.modmanager.update(
                            ctx,
                            ui,
                            &mut self.modmanager_settings,
                            self.steam_state.as_mut().ok(),
                        )
                    });
                if self.modmanager.is_done() {
                    self.state = AppState::Connect;
                }
            }
            AppState::SelfUpdate => {
                egui::CentralPanel::default().show(ctx, draw_bg);
                egui::Window::new(tr("selfupdate"))
                    .auto_sized()
                    .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        self.self_update.self_update(ui);
                    });
            }
            AppState::LangPick => {
                egui::CentralPanel::default().show(ctx, draw_bg);
                egui::Window::new(tr("lang_picker"))
                    .auto_sized()
                    .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        for lang in &LANGS {
                            ui.set_max_width(200.0);
                            ui.vertical_centered_justified(|ui| {
                                if ui.button(lang.name()).clicked() {
                                    self.saved_state.lang_id = Some(lang.id());
                                    set_current_locale(lang.id())
                                }
                            });
                        }
                        if ui.button(tr("button_confirm")).clicked() {
                            self.state = AppState::ModManager;
                        }
                    });
            }
        };
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.saved_state);
        eframe::set_value(storage, MODMANAGER, &self.modmanager_settings);
    }
}

fn peer_role(peer: net::omni::OmniPeerId, netman: &Arc<net::NetManager>) -> String {
    if peer == netman.peer.host_id() {
        tr("player_host")
    } else if Some(peer) == netman.peer.my_id() {
        tr("player_me")
    } else {
        tr("player_player")
    }
}
