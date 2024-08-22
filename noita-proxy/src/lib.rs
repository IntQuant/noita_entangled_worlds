use bitcode::{Decode, Encode};
use bookkeeping::{
    noita_launcher::{LaunchTokenResult, NoitaLauncher},
    save_state::SaveState,
};
use clipboard::{ClipboardContext, ClipboardProvider};
use eframe::egui::{
    self, Align2, Button, Color32, Context, DragValue, FontDefinitions, FontFamily, InnerResponse,
    Key, Margin, OpenUrl, Rect, RichText, ScrollArea, Slider, TextureOptions, Ui, Vec2, Window,
};
use egui_plot::{Plot, PlotPoint, PlotUi, Text};
use image::DynamicImage::ImageRgba8;
use image::RgbaImage;
use lang::{set_current_locale, tr, LANGS};
use mod_manager::{Modmanager, ModmanagerSettings};
use net::{
    omni::PeerVariant,
    steam_networking::{ExtraPeerState, PerPeerStatusEntry},
    NetManagerInit, RunInfo,
};
use self_update::SelfUpdateManager;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{
    fmt::Display,
    net::SocketAddr,
    ops::Deref,
    sync::{atomic::Ordering, Arc},
    thread::JoinHandle,
    time::Duration,
};
use steamworks::{LobbyId, SteamAPIInitError};
use tangled::Peer;
use tracing::info;
use unic_langid::LanguageIdentifier;

mod util;
use util::args::Args;
pub use util::{args, lang, steam_helper};

mod bookkeeping;
use crate::player_cosmetics::{
    display_player_skin, player_path, player_select_current_color_slot,
    player_skin_display_color_picker, shift_hue,
};
pub use bookkeeping::{mod_manager, releases, self_update};
mod net;
mod player_cosmetics;
pub mod recorder;
#[derive(Debug, Decode, Encode, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub(crate) enum GameMode {
    SharedHealth,
    LocalHealth,
    // MestariMina, // TODO later
}

#[derive(Debug, Decode, Encode, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    seed: u64,
    debug_mode: bool,
    world_sync_version: u32,
    player_tether: bool,
    tether_length: u32,
    use_constant_seed: bool,
    item_dedup: bool,
    enemy_hp_mult: f32,
    world_sync_interval: u32,
    game_mode: GameMode,
    friendly_fire: bool,
    enemy_sync_interval: u32,
    progress: Vec<String>,
}

impl Default for GameSettings {
    fn default() -> Self {
        GameSettings {
            seed: 0,
            debug_mode: false,
            world_sync_version: 2,
            player_tether: false,
            tether_length: 750,
            use_constant_seed: false,
            item_dedup: true,
            enemy_hp_mult: 1.0,
            world_sync_interval: 2,
            game_mode: GameMode::SharedHealth,
            friendly_fire: false,
            enemy_sync_interval: 2,
            progress: Vec::new(),
        }
    }
}

pub struct NetManStopOnDrop(pub Arc<net::NetManager>, Option<JoinHandle<()>>);

impl Deref for NetManStopOnDrop {
    type Target = Arc<net::NetManager>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for NetManStopOnDrop {
    fn drop(&mut self) {
        self.0.continue_running.store(false, Ordering::Relaxed);
        self.1.take().unwrap().join().unwrap();
    }
}

enum AppState {
    Connect,
    ModManager,
    Netman {
        netman: NetManStopOnDrop,
        noita_launcher: NoitaLauncher,
    },
    Error {
        message: String,
    },
    SelfUpdate,
    LangPick,
    AskSavestateReset,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlayerAppearance {
    player_color: PlayerColor,
    player_picker: PlayerPicker,
    hue: f32,
    cosmetics: (bool, bool, bool),
}

impl Default for PlayerAppearance {
    fn default() -> Self {
        Self {
            player_color: PlayerColor::default(),
            player_picker: PlayerPicker::None,
            hue: 0.0,
            cosmetics: (true, true, true),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct AppSavedState {
    addr: String,
    nickname: Option<String>,
    times_started: u32,
    lang_id: Option<LanguageIdentifier>,
    #[serde(default)]
    game_settings: GameSettings,
    start_game_automatically: bool,
    #[serde(default)]
    show_extra_debug_stuff: bool,
    #[serde(default)]
    record_all: bool,
}

impl Default for AppSavedState {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:5123".to_string(),
            nickname: None,
            times_started: 0,
            lang_id: None,
            game_settings: GameSettings::default(),
            start_game_automatically: false,
            show_extra_debug_stuff: false,
            record_all: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Decode, Encode, Copy, Clone)]
pub struct PlayerColor {
    player_main: [u8; 4],
    player_alt: [u8; 4],
    player_arm: [u8; 4],
    player_cape: [u8; 4],
    player_cape_edge: [u8; 4],
    player_forearm: [u8; 4],
}

impl Default for PlayerColor {
    fn default() -> Self {
        Self {
            player_main: [155, 111, 154, 255],
            player_alt: [127, 84, 118, 255],
            player_arm: [89, 67, 84, 255],
            player_cape: [118, 84, 127, 255],
            player_cape_edge: [154, 111, 155, 255],
            player_forearm: [158, 115, 154, 255],
        }
    }
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
enum PlayerPicker {
    None,
    PlayerMain,
    PlayerAlt,
    PlayerArm,
    PlayerCape,
    PlayerCapeEdge,
    PlayerForearm,
}

pub struct App {
    state: AppState,
    modmanager: Modmanager,
    steam_state: Result<steam_helper::SteamState, SteamAPIInitError>,
    app_saved_state: AppSavedState,
    run_save_state: SaveState,
    modmanager_settings: ModmanagerSettings,
    self_update: SelfUpdateManager,
    show_map_plot: bool,
    /// Show settings in netman screen?
    show_settings: bool,
    lobby_id_field: String,
    args: Args,
    /// `true` if we haven't started noita automatically yet.
    can_start_automatically: bool,
    player_image: RgbaImage,
    end_run_confirmation: bool,
    appearance: PlayerAppearance,
}

const MODMANAGER: &str = "modman";
const APPEARANCE: &str = "appearance";

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

fn square_button_icon(ui: &mut Ui, icon: egui::Image) -> egui::Response {
    let side = ui.available_width();
    ui.add_sized([side, side], egui::widgets::ImageButton::new(icon)
        .rounding(ui.style().visuals.widgets.noninteractive.rounding) // Somewhy it doesnt inherit style correctly
    ) 
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, args: Args) -> Self {
        let mut saved_state: AppSavedState = cc
            .storage
            .and_then(|storage| eframe::get_value(storage, eframe::APP_KEY))
            .unwrap_or_default();
        let modmanager_settings: ModmanagerSettings = cc
            .storage
            .and_then(|storage| eframe::get_value(storage, MODMANAGER))
            .unwrap_or_default();
        let appearance: PlayerAppearance = cc
            .storage
            .and_then(|storage| {
                eframe::get_value(storage, APPEARANCE).inspect(|x| info!("Loaded appearance {x:?}"))
            })
            .unwrap_or_else(|| {
                // Fallback to loading from the old location
                cc.storage
                    .and_then(|storage| {
                        eframe::get_value(storage, eframe::APP_KEY)
                            .inspect(|x| info!("Loaded appearance from fallback: {x:?}"))
                    })
                    .unwrap_or_default()
            });
        saved_state.times_started += 1;

        Self::set_fonts(&cc.egui_ctx);
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
        let run_save_state = if let Ok(path) = std::env::current_exe() {
            SaveState::new(path.parent().unwrap().join("save_state"))
        } else {
            SaveState::new("./save_state/".into())
        };
        let path = player_path(modmanager_settings.mod_path());
        let player_image = if path.exists() {
            image::open(path)
                .unwrap_or(ImageRgba8(RgbaImage::new(20, 20)))
                .crop(1, 1, 8, 18)
                .into_rgba8()
        } else {
            RgbaImage::new(1, 1)
        };
        Self {
            state,
            modmanager: Modmanager::default(),
            steam_state: steam_helper::SteamState::new(),
            app_saved_state: saved_state,
            modmanager_settings,
            self_update: SelfUpdateManager::new(),
            show_map_plot: false,
            show_settings: false,
            lobby_id_field: "".to_string(),
            args,
            can_start_automatically: false,
            run_save_state,
            player_image,
            end_run_confirmation: false,
            appearance,
        }
    }

    fn get_netman_init(&self) -> NetManagerInit {
        let steam_nickname = if let Ok(steam) = &self.steam_state {
            Some(steam.get_user_name(steam.get_my_id()))
        } else {
            None
        };
        let my_nickname = self.app_saved_state.nickname.clone().or(steam_nickname);
        let mod_path = self.modmanager_settings.mod_path();
        let mut cosmetics = self.appearance.cosmetics;
        if let Some(path) = &self.modmanager_settings.game_save_path {
            let flags = path.join("save00/persistent/flags");
            let hat = flags.join("secret_hat").exists();
            let amulet = flags.join("secret_amulet").exists();
            let gem = flags.join("secret_amulet_gem").exists();
            if !hat {
                cosmetics.0 = false
            }
            if !amulet {
                cosmetics.1 = false
            }
            if !gem {
                cosmetics.2 = false
            }
        }
        NetManagerInit {
            my_nickname,
            save_state: self.run_save_state.clone(),
            player_color: self.appearance.player_color,
            cosmetics,
            mod_path,
            player_path: player_path(self.modmanager_settings.mod_path()),
        }
    }

    fn change_state_to_netman(&mut self, netman: Arc<net::NetManager>, player_path: PathBuf) {
        let handle = netman.clone().start(player_path);
        self.state = AppState::Netman {
            netman: NetManStopOnDrop(netman, Some(handle)),
            noita_launcher: NoitaLauncher::new(
                &self.modmanager_settings.game_exe_path,
                self.args.launch_cmd.as_deref(),
                self.steam_state.as_mut().ok(),
            ),
        };
        self.can_start_automatically = true;
    }

    fn start_server(&mut self) {
        let bind_addr = "0.0.0.0:5123".parse().unwrap();
        let peer = Peer::host(bind_addr, None).unwrap();
        let netman = net::NetManager::new(PeerVariant::Tangled(peer), self.get_netman_init());
        self.set_netman_settings(&netman);
        self.change_state_to_netman(netman, player_path(self.modmanager_settings.mod_path()));
    }

    fn set_netman_settings(&mut self, netman: &Arc<net::NetManager>) {
        let run_info: Option<RunInfo> = self.run_save_state.load();
        let mut settings = netman.settings.lock().unwrap();
        *settings = self.app_saved_state.game_settings.clone();
        if !self.app_saved_state.game_settings.use_constant_seed {
            if let Some(info) = run_info {
                settings.seed = info.seed;
                info!("Using saved seed: {}", settings.seed);
            } else {
                settings.seed = rand::random();
                info!("Using random seed: {}", settings.seed);
            }
        } else {
            info!("Using constant seed: {}", settings.seed);
        }
        settings.progress = self.modmanager_settings.get_progress().unwrap_or_default();
        *netman.pending_settings.lock().unwrap() = settings.clone();
        netman.accept_local.store(true, Ordering::SeqCst);
    }
    fn start_connect(&mut self, addr: SocketAddr) {
        let peer = Peer::connect(addr, None).unwrap();
        let netman = net::NetManager::new(PeerVariant::Tangled(peer), self.get_netman_init());
        self.change_state_to_netman(netman, player_path(self.modmanager_settings.mod_path()));
    }

    fn start_steam_host(&mut self) {
        let peer = net::steam_networking::SteamPeer::new_host(
            steamworks::LobbyType::Private,
            self.steam_state.as_ref().unwrap().client.clone(),
        );
        let netman = net::NetManager::new(PeerVariant::Steam(peer), self.get_netman_init());
        self.set_netman_settings(&netman);
        self.change_state_to_netman(netman, player_path(self.modmanager_settings.mod_path()));
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
        self.change_state_to_netman(netman, player_path(self.modmanager_settings.mod_path()));
    }

    fn connect_screen(&mut self, ctx: &Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.app_saved_state.times_started % 20 == 0 {
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
                        .app_saved_state
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
                    if square_button_icon(ui, egui::Image::new(egui::include_image!("../assets/discord-mark-white.png")))
                        .on_hover_text(tr("button_open_discord"))
                        .clicked()
                    {
                        ctx.open_url(OpenUrl::new_tab("https://discord.gg/uAK7utvVWN"));
                    }
                    let secret_active = ui.input(|i| i.modifiers.ctrl && i.key_down(Key::D));
                    if secret_active && ui.button("reset all data").clicked() {
                        self.app_saved_state = Default::default();
                        self.modmanager_settings = Default::default();
                        self.state = AppState::LangPick;
                    }
                })
            });

            ui.allocate_ui_at_rect(settings_rect.shrink(group_shrink), |ui| {
                filled_group(ui, |ui| {
                    ui.set_min_size(ui.available_size());
                    ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                        self.show_game_settings(ui, true);
                    });
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

                    ui.text_edit_singleline(&mut self.app_saved_state.addr);
                    let addr = self.app_saved_state.addr.parse();
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

    fn show_game_settings(&mut self, ui: &mut Ui, show_local: bool) {
        heading_with_underline(ui, tr("connect_settings"));
        let game_settings = &mut self.app_saved_state.game_settings;
        ui.label("Game mode");
        ui.radio_value(
            &mut game_settings.game_mode,
            GameMode::SharedHealth,
            "Shared health",
        );
        ui.radio_value(
            &mut game_settings.game_mode,
            GameMode::LocalHealth,
            "Local health",
        );
        match game_settings.game_mode {
            GameMode::SharedHealth => {
                ui.label("Health is shared, but scales with player count.");
                ui.label("Percentage-based damage and full heals are adjusted.");
                ui.label("The original mode.");
            }
            GameMode::LocalHealth => {
                ui.label("Every player has their own health, run ends when all player are dead.");
                ui.label("There is a respawn mechanic.");
            }
        }
        ui.add_space(20.0);
        ui.label(tr("connect_settings_debug"));
        ui.checkbox(
            &mut game_settings.debug_mode,
            tr("connect_settings_debug_en"),
        );
        ui.checkbox(
            &mut game_settings.use_constant_seed,
            tr("connect_settings_debug_fixed_seed"),
        );
        ui.horizontal(|ui| {
            ui.label(tr("connect_settings_seed"));
            ui.add(DragValue::new(&mut game_settings.seed));
        });
        if game_settings.world_sync_version == 2 {
            ui.add_space(10.0);
            ui.label(tr("World-will-be-synced-every-this-many-frames"));
            ui.label(tr("Higher-values-result-in-less-performance-impact"));
            ui.add(Slider::new(&mut game_settings.world_sync_interval, 1..=10));
        }
        ui.label(tr("world-sync-is-pixel-sync-note"));
        ui.add_space(20.0);

        ui.label("Enemy sync interval");
        ui.add(Slider::new(&mut game_settings.enemy_sync_interval, 2..=8));

        ui.add_space(20.0);
        ui.label(tr("connect_settings_player_tether_desc"));
        ui.checkbox(
            &mut game_settings.player_tether,
            tr("connect_settings_player_tether"),
        );
        ui.add(
            Slider::new(&mut game_settings.tether_length, 10..=5000)
                .text(tr("connect_settings_player_tether_length")),
        );
        ui.add_space(20.0);
        ui.checkbox(
            &mut game_settings.item_dedup,
            tr("connect_settings_item_dedup"),
        );
        ui.add_space(20.0);
        ui.add(
            Slider::new(&mut game_settings.enemy_hp_mult, 1.0..=1000.0)
                .logarithmic(true)
                .text(tr("connect_settings_enemy_hp_scale")),
        );
        ui.add_space(20.0);
        ui.checkbox(&mut game_settings.friendly_fire, "Friendly fire");
        if show_local {
            heading_with_underline(ui, tr("connect_settings_local"));
            ui.checkbox(
                &mut self.app_saved_state.start_game_automatically,
                tr("connect_settings_autostart"),
            );
            ui.add_space(20.0);
            if self.player_image.width() == 1 {
                self.player_image = image::open(player_path(self.modmanager_settings.mod_path()))
                    .unwrap_or(ImageRgba8(RgbaImage::new(20, 20)))
                    .crop(1, 1, 8, 18)
                    .into_rgba8();
            }
            let old_hue = self.appearance.hue;
            ui.add(Slider::new(&mut self.appearance.hue, 0.0..=360.0).text("Shift hue"));
            if old_hue != self.appearance.hue {
                let diff = self.appearance.hue - old_hue;
                shift_hue(diff, &mut self.appearance.player_color.player_main);
                shift_hue(diff, &mut self.appearance.player_color.player_alt);
                shift_hue(diff, &mut self.appearance.player_color.player_arm);
                shift_hue(diff, &mut self.appearance.player_color.player_forearm);
                shift_hue(diff, &mut self.appearance.player_color.player_cape);
                shift_hue(diff, &mut self.appearance.player_color.player_cape_edge);
            }
            ui.horizontal(|ui| {
                display_player_skin(ui, self);
                player_select_current_color_slot(ui, self);
                player_skin_display_color_picker(
                    ui,
                    &mut self.appearance.player_color,
                    &self.appearance.player_picker,
                );
            });
            if ui.button("Reset colors to default").clicked() {
                self.appearance.player_color = PlayerColor::default();
                self.appearance.hue = 0.0
            }
        }
    }

    fn connect_to_steam_lobby(&mut self, lobby_id: String) {
        let id = lobby_id.trim().parse().map(LobbyId::from_raw);
        match id {
            Ok(id) => self.start_steam_connect(id),
            Err(_error) => self.notify_error(tr("connect_steam_connect_invalid_lobby_id")),
        }
    }

    fn set_fonts(ctx: &Context) {
        let mut font_definitions = FontDefinitions::default();

        font_definitions.font_data.insert(
            "noto_sans".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/font/NotoSans-Regular.ttf")),
        );
        font_definitions.font_data.insert(
            "noto_sans_jp".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/font/NotoSansJP-Light.ttf")),
        );

        font_definitions
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .push("noto_sans".to_owned());
        font_definitions
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .push("noto_sans_jp".to_owned());

        font_definitions
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .push("noto_sans".to_owned());
        font_definitions
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .push("noto_sans_jp".to_owned());

        ctx.set_fonts(font_definitions);
    }

    fn switch_to_connect(&mut self) {
        self.state = if self.run_save_state.has_savestate() {
            AppState::AskSavestateReset
        } else {
            AppState::Connect
        };
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
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
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
                            if cfg!(target_os = "linux") {
                                ui.label(id.raw().to_string());
                            }

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
                                let start_auto = self.can_start_automatically && self.app_saved_state.start_game_automatically;
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

                    if netman.peer.is_host() {
                        ui.add_space(15.0);
                        if !self.end_run_confirmation && ui.button("End run").clicked()
                        {
                            self.end_run_confirmation = true
                        }
                        else if self.end_run_confirmation && ui.button("Confirm").clicked()
                        {
                            self.end_run_confirmation = false;
                            netman.end_run.store(true, Ordering::Relaxed)
                        }
                        ui.add_space(15.0);
                        if ui.button(tr("netman_show_settings")).clicked() {
                            self.show_settings = true;
                        }
                    }
                    ui.add_space(15.0);

                    ui.checkbox(&mut self.app_saved_state.show_extra_debug_stuff, "Show debug stuff");

                    if self.app_saved_state.show_extra_debug_stuff {
                        Window::new("Connection status").show(ctx, |ui| {
                            match &netman.peer {
                                PeerVariant::Tangled(_) => {ui.label("No connection info available in tangled mode");}
                                PeerVariant::Steam(peer) => {
                                    let steam = self.steam_state.as_ref().unwrap();
                                    let report = peer.generate_report();
                                    egui::Grid::new("Conn status grid").striped(true).show(ui, |ui| {
                                        add_per_status_ui(&report, steam, ui);
                                    });
                                },
                            }
                        });


                        if self.show_map_plot {
                            let build_fn = |plot: &mut PlotUi| {
                                netman.world_info.with_player_infos(|peer, info| {
                                    let username = if netman.peer.is_steam() {
                                        let steam = self.steam_state.as_mut().expect(
                                            "steam should be available, as we are using steam networking",
                                        );
                                        steam.get_user_name(peer.into())
                                    } else {
                                        peer.as_hex()
                                    };
                                    plot.text(Text::new(PlotPoint::new(info.x, -info.y), username).highlight(true))
                                });
                            };
                            Plot::new("map").data_aspect(1.0).show(ui, build_fn);
                        } else if ui.button("Show debug plot").clicked() {
                            self.show_map_plot = true;
                        }
                        ui.checkbox(&mut self.app_saved_state.record_all, "Record EVERYTHING sent to noita.");
                    }
                });
                netman
                    .enable_recorder
                    .store(self.app_saved_state.record_all, Ordering::Relaxed);
                if netman.peer.is_host() {
                    let mut show = self.show_settings;
                    let netman = netman.clone();
                    Window::new(tr("connect_settings"))
                        .open(&mut show)
                        .show(ctx, |ui| {
                            self.show_game_settings(ui, false);
                            if ui.button(tr("netman_apply_settings")).clicked() {
                                *netman.pending_settings.lock().unwrap() =
                                    self.app_saved_state.game_settings.clone();
                            }
                        });
                    self.show_settings = show;
                }
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
                Window::new(tr("modman"))
                    .auto_sized()
                    .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.set_max_width(600.0);
                        self.modmanager.update(
                            ctx,
                            ui,
                            &mut self.modmanager_settings,
                            self.steam_state.as_mut().ok(),
                        )
                    });
                if self.modmanager.is_done() {
                    self.switch_to_connect();
                }
            }
            AppState::SelfUpdate => {
                egui::CentralPanel::default().show(ctx, draw_bg);
                Window::new(tr("selfupdate"))
                    .auto_sized()
                    .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.set_max_width(600.0);
                        self.self_update.self_update(ui);
                    });
            }
            AppState::LangPick => {
                egui::CentralPanel::default().show(ctx, draw_bg);
                Window::new(tr("lang_picker"))
                    .auto_sized()
                    .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        for lang in &LANGS {
                            ui.set_max_width(200.0);
                            ui.vertical_centered_justified(|ui| {
                                if ui.button(lang.name()).clicked() {
                                    self.app_saved_state.lang_id = Some(lang.id());
                                    set_current_locale(lang.id())
                                }
                            });
                        }
                        if ui.button(tr("button_confirm")).clicked() {
                            self.state = AppState::ModManager;
                        }
                    });
            }
            AppState::AskSavestateReset => {
                Window::new(tr("An-in-progress-run-has-been-detected"))
                    .auto_sized()
                    .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.label(tr("savestate_desc"));
                        ui.horizontal(|ui| {
                            if ui.button(tr("Continue")).clicked() {
                                self.state = AppState::Connect;
                            }
                            if ui.button(tr("New-game")).clicked() {
                                self.state = AppState::Connect;
                                self.run_save_state.reset();
                            }
                        });
                    });
            }
        };
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.app_saved_state);
        eframe::set_value(storage, MODMANAGER, &self.modmanager_settings);
        eframe::set_value(storage, APPEARANCE, &self.appearance);
    }
}

fn add_per_status_ui(
    report: &net::steam_networking::ConnectionStatusReport,
    steam: &steam_helper::SteamState,
    ui: &mut Ui,
) {
    ui.label("Name");
    ui.label("Status");
    ui.label("Ping");
    ui.label("LocQ❓")
        .on_hover_text("Local Connection Quality (percentage of packets we delivered).");
    ui.label("RemQ❓")
        .on_hover_text("Remote Connection Quality (percentage of packets delivered to us).");
    ui.label("In");
    ui.label("Out");
    ui.label("MaxSendRate");
    ui.label("PenUnr❓")
        .on_hover_text("Pending unreliable messages");
    ui.label("PenRel❓")
        .on_hover_text("Pending reliable messages");
    ui.label("UnAck❓").on_hover_text(
        "Amount of reliable packages that were sent but weren't confirmed as received yet.",
    );
    ui.end_row();

    for PerPeerStatusEntry { peer, status } in &report.per_peer_statuses {
        let name = steam.get_user_name((*peer).into());
        ui.label(&name);
        match status {
            net::steam_networking::PerPeerStatus::Connected { realtimeinfo } => {
                ui.label("Ok❓").on_hover_text("Connected");
                ui.label(format!("{}ms", realtimeinfo.ping()));
                ui.label(format!(
                    "{:.2}%",
                    realtimeinfo.connection_quality_local() * 100.0
                ));
                ui.label(format!(
                    "{:.2}%",
                    realtimeinfo.connection_quality_remote() * 100.0
                ));
                ui.label(format!("{}by/s", realtimeinfo.in_bytes_per_sec()));
                ui.label(format!("{}by/s", realtimeinfo.out_bytes_per_sec()));
                ui.label(format!("{}by/s", realtimeinfo.send_rate_bytes_per_sec()));
                ui.label(format!("{}", realtimeinfo.pending_unreliable()));
                ui.label(format!("{}", realtimeinfo.pending_reliable()));
                ui.label(format!("{}", realtimeinfo.sent_unacked_reliable()));
            }
            net::steam_networking::PerPeerStatus::AwaitingIncoming => {
                ui.label("Awa❓")
                    .on_hover_text("Awaiting incoming connection from this peer.");
            }
            net::steam_networking::PerPeerStatus::ConnectionPending => {
                ui.label("Pen❓").on_hover_text("Connection pending.");
            }
            net::steam_networking::PerPeerStatus::NoFurtherInfo => {
                ui.label("NoI❓")
                    .on_hover_text("Connected, but no further info available.");
            }
        }
        ui.end_row();
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
