use arboard::Clipboard;
use bitcode::{Decode, Encode};
use bookkeeping::{
    noita_launcher::{LaunchTokenResult, NoitaLauncher},
    releases::Version,
    save_paths::SavePaths,
    save_state::SaveState,
    self_restart::SelfRestarter,
};
use cpal::traits::{DeviceTrait, HostTrait};
use eframe::egui::load::TexturePoll;
use eframe::egui::{
    self, Align2, Button, Color32, ComboBox, Context, DragValue, FontDefinitions, FontFamily,
    ImageButton, InnerResponse, Key, Layout, Margin, OpenUrl, Rect, RichText, ScrollArea, Sense,
    SizeHint, Slider, TextureOptions, ThemePreference, Ui, UiBuilder, Vec2, Visuals, Window, pos2,
};
use eframe::epaint::TextureHandle;
use image::DynamicImage::ImageRgba8;
use image::RgbaImage;
use lang::{LANGS, set_current_locale, tr};
use lobby_code::{LobbyCode, LobbyError, LobbyKind};
use mod_manager::{Modmanager, ModmanagerSettings};
use net::{
    NetManagerInit, RunInfo,
    omni::PeerVariant,
    steam_networking::{ExtraPeerState, PerPeerStatusEntry},
};
use player_cosmetics::PlayerPngDesc;
use rustc_hash::FxHashMap;
use self_update::SelfUpdateManager;
use serde::{Deserialize, Serialize};
use std::process::exit;
use std::thread::sleep;
use std::{collections::HashMap, fs, str::FromStr};
use std::{
    fmt::Display,
    mem,
    net::SocketAddr,
    ops::Deref,
    sync::{Arc, atomic::Ordering},
    thread::JoinHandle,
    time::Duration,
};
use std::{net::IpAddr, path::PathBuf};
use steamworks::{LobbyId, SteamAPIInitError};
use tangled::{Peer, Reliability};
use tokio::time;
use tracing::info;
use unic_langid::LanguageIdentifier;

mod util;
pub use util::{args, lang, steam_helper};
use util::{args::Args, steam_helper::LobbyExtraData};

mod bookkeeping;
use crate::net::messages::NetMsg;
use crate::net::omni::OmniPeerId;
use crate::net::world::world_model::ChunkCoord;
use crate::player_cosmetics::{
    display_player_skin, get_player_skin, player_path, player_select_current_color_slot,
    player_skin_display_color_picker, shift_hue,
};
pub use bookkeeping::{mod_manager, releases, self_update};
use shared::WorldPos;
mod lobby_code;
pub mod net;
mod player_cosmetics;

const DEFAULT_PORT: u16 = 5123;

#[derive(Debug, Decode, Encode, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum GameMode {
    SharedHealth,
    LocalHealth(LocalHealthMode),
    // MestariMina, // TODO later
}
impl GameMode {
    fn color(&self) -> Color32 {
        match self {
            GameMode::SharedHealth => Color32::LIGHT_BLUE,
            GameMode::LocalHealth(LocalHealthMode::Normal) => Color32::GOLD,
            GameMode::LocalHealth(LocalHealthMode::Alternate) => Color32::GREEN,
            GameMode::LocalHealth(LocalHealthMode::PermaDeath) => Color32::ORANGE,
            GameMode::LocalHealth(LocalHealthMode::PvP) => Color32::RED,
        }
    }
}

impl Display for GameMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let desc = match self {
            GameMode::SharedHealth => "Shared",
            GameMode::LocalHealth(LocalHealthMode::Normal) => "LocalNormal",
            GameMode::LocalHealth(LocalHealthMode::Alternate) => "LocalAlternate",
            GameMode::LocalHealth(LocalHealthMode::PermaDeath) => "LocalPermadeath",
            GameMode::LocalHealth(LocalHealthMode::PvP) => "PvP",
        };
        write!(f, "{}", desc)
    }
}

impl FromStr for GameMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Shared" => Ok(GameMode::SharedHealth),
            "LocalNormal" => Ok(GameMode::LocalHealth(LocalHealthMode::Normal)),
            "LocalAlternate" => Ok(GameMode::LocalHealth(LocalHealthMode::Alternate)),
            "LocalPermadeath" => Ok(GameMode::LocalHealth(LocalHealthMode::PermaDeath)),
            "PvP" => Ok(GameMode::LocalHealth(LocalHealthMode::PvP)),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Decode, Encode, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum LocalHealthMode {
    Normal,
    Alternate,
    PermaDeath,
    PvP,
}

#[derive(Debug, Decode, Encode, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct GameSettings {
    seed: u64,
    world_num: u16,
    debug_mode: Option<bool>,
    use_constant_seed: bool,
    duplicate: Option<bool>,
    enemy_hp_mult: Option<f32>,
    game_mode: Option<GameMode>,
    friendly_fire: Option<bool>,
    randomize_perks: Option<bool>,
    progress: Vec<String>,
    max_players: Option<u32>,
    health_per_player: Option<u32>,
    health_lost_on_revive: Option<u32>,
    no_material_damage: Option<bool>,
    global_hp_loss: Option<bool>,
    dead_isnt_dead: Option<bool>,
    perk_ban_list: Option<String>,
    disabled_globals: Option<String>,
    spell_ban_list: Option<String>,
    physics_damage: Option<bool>,
    share_gold: Option<bool>,
    nice_terraforming: Option<bool>,
    same_loadout: Option<bool>,
    disable_kummitus: Option<bool>,
    give_host_sampo: Option<bool>,
    home_on_players: Option<bool>,
    pvp_kill_steal: Option<u32>,
    dont_steal: Option<bool>,
    wait_on_players: Option<bool>,
    time_in_hm: Option<u32>,
    time_out_hm: Option<u32>,
    chest_on_win: Option<bool>,
    wait_for_time: Option<bool>,
    timed: Option<bool>,
}
impl GameSettings {
    fn show_editor(&mut self, ui: &mut Ui, enabled: bool) {
        ui.add_enabled_ui(enabled, |ui| {
            let def = DefaultSettings::default();
            let game_settings = self;
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    let mut temp = game_settings.game_mode.unwrap_or(def.game_mode);
                    ui.label(tr("Game-mode"));
                    if ui
                        .radio_value(&mut temp, GameMode::SharedHealth, tr("Shared-health"))
                        .changed()
                        || ui
                            .radio_value(
                                &mut temp,
                                GameMode::LocalHealth(LocalHealthMode::Normal),
                                tr("Local-health"),
                            )
                            .changed()
                        || ui
                            .radio_value(
                                &mut temp,
                                GameMode::LocalHealth(LocalHealthMode::Alternate),
                                tr("Local-health-alt"),
                            )
                            .changed()
                        || ui
                            .radio_value(
                                &mut temp,
                                GameMode::LocalHealth(LocalHealthMode::PermaDeath),
                                tr("Local-health-perma"),
                            )
                            .changed()
                        || ui
                            .radio_value(
                                &mut temp,
                                GameMode::LocalHealth(LocalHealthMode::PvP),
                                "PvP",
                            )
                            .changed()
                    {
                        game_settings.game_mode = Some(temp)
                    }
                });
                ui.vertical(|ui| {
                    ui.set_height(220.0);
                    match game_settings.game_mode.unwrap_or(def.game_mode) {
                        GameMode::SharedHealth => {
                            ui.label(tr("shared_health_desc_1"));
                            ui.label(tr("shared_health_desc_2"));
                            ui.label(tr("shared_health_desc_3"));
                            ui.add_space(5.0);
                            ui.label(tr("Health-per-player"));
                            let mut temp = game_settings
                                .health_per_player
                                .unwrap_or(def.health_per_player);
                            if ui.add(Slider::new(&mut temp, 0..=100)).changed() {
                                game_settings.health_per_player = Some(temp)
                            }
                            {
                                let mut temp =
                                    game_settings.dead_isnt_dead.unwrap_or(def.dead_isnt_dead);
                                if ui.checkbox(&mut temp, tr("dead_isnt_dead")).changed() {
                                    game_settings.dead_isnt_dead = Some(temp)
                                }
                            }
                        }
                        GameMode::LocalHealth(mode) => match mode {
                            LocalHealthMode::Normal => {
                                ui.label(tr("local_health_desc_1"));
                                ui.add_space(5.0);
                                ui.label(tr("Health-percent-lost-on-reviving"));
                                {
                                    let mut temp = game_settings
                                        .health_lost_on_revive
                                        .unwrap_or(def.health_lost_on_revive);
                                    if ui.add(Slider::new(&mut temp, 0..=100)).changed() {
                                        game_settings.health_lost_on_revive = Some(temp)
                                    }
                                }
                                {
                                    let mut temp =
                                        game_settings.dead_isnt_dead.unwrap_or(def.dead_isnt_dead);
                                    if ui.checkbox(&mut temp, tr("dead_isnt_dead")).changed() {
                                        game_settings.dead_isnt_dead = Some(temp)
                                    }
                                }
                                {
                                    let mut temp =
                                        game_settings.global_hp_loss.unwrap_or(def.global_hp_loss);
                                    if ui.checkbox(&mut temp, tr("global_hp_loss")).changed() {
                                        game_settings.global_hp_loss = Some(temp)
                                    }
                                }
                                {
                                    let mut temp = game_settings
                                        .no_material_damage
                                        .unwrap_or(def.no_material_damage);
                                    if ui.checkbox(&mut temp, tr("no_material_damage")).changed() {
                                        game_settings.no_material_damage = Some(temp)
                                    }
                                }
                                ui.add_space(1.0);
                                {
                                    let mut temp =
                                        game_settings.physics_damage.unwrap_or(def.physics_damage);
                                    if ui.checkbox(&mut temp, tr("physics_damage")).changed() {
                                        game_settings.physics_damage = Some(temp)
                                    }
                                }
                            }
                            LocalHealthMode::Alternate => {
                                ui.label(tr("local_health_desc_1"));
                                ui.add_space(5.0);
                                ui.label(tr("Health-percent-lost-on-reviving"));
                                {
                                    let mut temp = game_settings
                                        .health_lost_on_revive
                                        .unwrap_or(def.health_lost_on_revive);
                                    if ui.add(Slider::new(&mut temp, 0..=100)).changed() {
                                        game_settings.health_lost_on_revive = Some(temp)
                                    }
                                }
                                {
                                    let mut temp =
                                        game_settings.dead_isnt_dead.unwrap_or(def.dead_isnt_dead);
                                    if ui.checkbox(&mut temp, tr("dead_isnt_dead")).changed() {
                                        game_settings.dead_isnt_dead = Some(temp)
                                    }
                                }
                                {
                                    let mut temp =
                                        game_settings.global_hp_loss.unwrap_or(def.global_hp_loss);
                                    if ui.checkbox(&mut temp, tr("global_hp_loss")).changed() {
                                        game_settings.global_hp_loss = Some(temp)
                                    }
                                }
                            }
                            LocalHealthMode::PermaDeath => {
                                ui.label(tr("local_health_desc_1"));
                            }

                            LocalHealthMode::PvP => {
                                ui.label("round based pvp mode");
                                ui.add_space(5.0);
                                {
                                    ui.label("% money stolen on kill");
                                    let mut temp =
                                        game_settings.pvp_kill_steal.unwrap_or(def.pvp_kill_steal);
                                    if ui.add(Slider::new(&mut temp, 0..=100)).changed() {
                                        game_settings.pvp_kill_steal = Some(temp)
                                    }
                                }
                                {
                                    let mut temp =
                                        game_settings.dont_steal.unwrap_or(def.dont_steal);
                                    if ui
                                        .checkbox(&mut temp, "just gain money instead of stealing")
                                        .changed()
                                    {
                                        game_settings.dont_steal = Some(temp)
                                    }
                                }
                                {
                                    let mut temp =
                                        game_settings.chest_on_win.unwrap_or(def.chest_on_win);
                                    if ui.checkbox(&mut temp, "spawns chest on win").changed() {
                                        game_settings.chest_on_win = Some(temp)
                                    }
                                }
                                let mut timed = game_settings.timed.unwrap_or(def.timed);
                                if ui.checkbox(&mut timed, "timed rounds/hm").changed() {
                                    game_settings.timed = Some(timed)
                                }
                                if timed {
                                    {
                                        let mut temp = game_settings
                                            .wait_for_time
                                            .unwrap_or(def.wait_for_time);
                                        if ui
                                            .checkbox(&mut temp, "wait on time to finish round")
                                            .changed()
                                        {
                                            game_settings.wait_for_time = Some(temp)
                                        }
                                    }
                                    {
                                        ui.label("time in hm");
                                        let mut temp =
                                            game_settings.time_in_hm.unwrap_or(def.time_in_hm);
                                        if ui.add(Slider::new(&mut temp, 30..=1200)).changed() {
                                            game_settings.time_in_hm = Some(temp)
                                        }
                                    }
                                    {
                                        ui.label("time in round");
                                        let mut temp =
                                            game_settings.time_out_hm.unwrap_or(def.time_out_hm);
                                        if ui.add(Slider::new(&mut temp, 30..=1200)).changed() {
                                            game_settings.time_out_hm = Some(temp)
                                        }
                                    }
                                } else {
                                    let mut temp = game_settings
                                        .wait_on_players
                                        .unwrap_or(def.wait_on_players);
                                    if ui
                                        .checkbox(
                                            &mut temp,
                                            "wait on players to finish round to start next round",
                                        )
                                        .changed()
                                    {
                                        game_settings.wait_on_players = Some(temp)
                                    }
                                }
                            }
                        },
                    }
                });
            });
            if cfg!(debug_assertions) {
                ui.add_space(10.0);
                ui.label(tr("connect_settings_debug"));
                {
                    let mut temp = game_settings.debug_mode.unwrap_or(def.debug_mode);
                    if ui
                        .checkbox(&mut temp, tr("connect_settings_debug_en"))
                        .changed()
                    {
                        game_settings.debug_mode = Some(temp)
                    }
                }
            }
            ui.add_space(10.0);
            ui.label("World generation");
            ui.horizontal(|ui| {
                ui.checkbox(
                    &mut game_settings.use_constant_seed,
                    tr("connect_settings_debug_fixed_seed"),
                );
                ui.add_space(10.0);
                if game_settings.use_constant_seed {
                    ui.label(tr("connect_settings_seed"));
                    ui.add(DragValue::new(&mut game_settings.seed));
                }
            });
            {
                let mut temp = game_settings.duplicate.unwrap_or(def.duplicate);
                if ui
                    .checkbox(&mut temp, "duplicate synced entities")
                    .changed()
                {
                    game_settings.duplicate = Some(temp)
                }
            }
            {
                let mut temp = game_settings
                    .nice_terraforming
                    .unwrap_or(def.nice_terraforming);
                if ui
                    .checkbox(
                        &mut temp,
                        "fix blackholes/explosions to work in unseen chunks",
                    )
                    .changed()
                {
                    game_settings.nice_terraforming = Some(temp)
                }
            }
            {
                let mut temp = game_settings
                    .disable_kummitus
                    .unwrap_or(def.disable_kummitus);
                if ui
                    .checkbox(&mut temp, "disable kummitus on non hosts")
                    .changed()
                {
                    game_settings.disable_kummitus = Some(temp)
                }
            }
            {
                let mut temp = game_settings.give_host_sampo.unwrap_or(def.give_host_sampo);
                if ui
                    .checkbox(&mut temp, "give host sampo on collection")
                    .changed()
                {
                    game_settings.give_host_sampo = Some(temp)
                }
            }
            {
                let mut temp = game_settings.enemy_hp_mult.unwrap_or(def.enemy_hp_mult);
                if ui
                    .add(
                        Slider::new(&mut temp, 1.0..=1000.0)
                            .logarithmic(true)
                            .text(tr("connect_settings_enemy_hp_scale")),
                    )
                    .changed()
                {
                    game_settings.enemy_hp_mult = Some(temp)
                }
            }
            {
                let mut temp = game_settings
                    .spell_ban_list
                    .clone()
                    .unwrap_or(def.spell_ban_list);
                ui.label("spell ban list, by internal names, comma seperated");
                if ui
                    .add_sized(
                        [ui.available_width() - 30.0, 20.0],
                        egui::TextEdit::singleline(&mut temp),
                    )
                    .changed()
                {
                    game_settings.spell_ban_list = Some(temp)
                }
            }
            ui.add_space(10.0);
            ui.label("Player settings");
            ui.horizontal(|ui| {
                ui.label(tr("connect_settings_max_players"));
                let mut temp = game_settings.max_players.unwrap_or(def.max_players);
                if ui.add(Slider::new(&mut temp, 2..=250)).changed() {
                    game_settings.max_players = Some(temp)
                }
            });
            {
                let mut temp = game_settings.friendly_fire.unwrap_or(def.friendly_fire);
                if ui.checkbox(&mut temp, tr("Enable-friendly-fire")).changed() {
                    game_settings.friendly_fire = Some(temp)
                }
            }
            {
                let mut temp = game_settings.share_gold.unwrap_or(def.share_gold);
                if ui.checkbox(&mut temp, "Share Gold").changed() {
                    game_settings.share_gold = Some(temp)
                }
            }
            {
                let mut temp = game_settings.same_loadout.unwrap_or(def.same_loadout);
                if ui
                    .checkbox(&mut temp, tr("Player-have-same-starting-loadout"))
                    .changed()
                {
                    game_settings.same_loadout = Some(temp)
                }
            }
            {
                let mut temp = game_settings.home_on_players.unwrap_or(def.home_on_players);
                if ui
                    .checkbox(&mut temp, "have homing home on players")
                    .changed()
                {
                    game_settings.home_on_players = Some(temp)
                }
            }
            ui.add_space(10.0);
            ui.label("Perks");
            {
                let mut temp = game_settings.randomize_perks.unwrap_or(def.randomize_perks);
                if ui
                    .checkbox(
                        &mut temp,
                        tr("Have-perk-pools-be-independent-of-each-other"),
                    )
                    .changed()
                {
                    game_settings.randomize_perks = Some(temp)
                }
            }
            {
                let mut temp = game_settings
                    .perk_ban_list
                    .clone()
                    .unwrap_or(def.perk_ban_list);
                ui.label("perk ban list, by internal names, comma seperated");
                if ui
                    .add_sized(
                        [ui.available_width() - 30.0, 20.0],
                        egui::TextEdit::singleline(&mut temp),
                    )
                    .changed()
                {
                    game_settings.perk_ban_list = Some(temp)
                }
            }
            {
                let mut temp = game_settings
                    .disabled_globals
                    .clone()
                    .unwrap_or(def.disabled_globals);
                ui.label("global perks to ignore, by internal names, comma seperated, will cause undefined behaviour do not report issues, find list in perk_fns.lua");
                if ui
                    .add_sized(
                        [ui.available_width() - 30.0, 20.0],
                        egui::TextEdit::singleline(&mut temp),
                    )
                    .changed()
                {
                    game_settings.disabled_globals = Some(temp)
                }
            }
            if ui.button(tr("apply_default_settings")).clicked() {
                *game_settings = GameSettings::default()
            }
        });
    }
}
pub struct DefaultSettings {
    debug_mode: bool,
    //item_dedup: bool,
    enemy_hp_mult: f32,
    game_mode: GameMode,
    friendly_fire: bool,
    randomize_perks: bool,
    max_players: u32,
    health_per_player: u32,
    health_lost_on_revive: u32,
    no_material_damage: bool,
    global_hp_loss: bool,
    dead_isnt_dead: bool,
    perk_ban_list: String,
    disabled_globals: String,
    spell_ban_list: String,
    physics_damage: bool,
    share_gold: bool,
    nice_terraforming: bool,
    same_loadout: bool,
    duplicate: bool,
    disable_kummitus: bool,
    give_host_sampo: bool,
    home_on_players: bool,
    pvp_kill_steal: u32,
    dont_steal: bool,
    wait_on_players: bool,
    time_in_hm: u32,
    time_out_hm: u32,
    chest_on_win: bool,
    wait_for_time: bool,
    timed: bool,
}

impl Default for DefaultSettings {
    fn default() -> Self {
        DefaultSettings {
            debug_mode: false,
            //item_dedup: true,
            randomize_perks: true,
            enemy_hp_mult: 1.0,
            game_mode: GameMode::LocalHealth(LocalHealthMode::Normal),
            friendly_fire: false,
            max_players: 250,
            health_per_player: 100,
            health_lost_on_revive: 0,
            no_material_damage: false,
            global_hp_loss: false,
            dead_isnt_dead: false,
            perk_ban_list: String::new(),
            disabled_globals: String::new(),
            spell_ban_list: String::new(),
            physics_damage: true,
            share_gold: false,
            nice_terraforming: true,
            same_loadout: false,
            duplicate: false,
            disable_kummitus: false,
            give_host_sampo: false,
            home_on_players: false,
            pvp_kill_steal: 50,
            dont_steal: true,
            wait_on_players: true,
            time_in_hm: 300,
            time_out_hm: 600,
            chest_on_win: true,
            wait_for_time: false,
            timed: true,
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

impl Drop for App {
    fn drop(&mut self) {
        self.set_settings()
    }
}

enum AppState {
    Connect,
    ModManager,
    TangledConnecting {
        peer: Peer,
    },
    ConnectedLobby {
        netman: NetManStopOnDrop,
        noita_launcher: NoitaLauncher,
    },
    Error {
        message: String,
    },
    SelfUpdate,
    LangPick,
    AskSavestateReset,
    GogModeIssue(LobbyCode),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ConnectedMenu {
    Normal,
    Settings,
    Mods,
    BanList,
    ConnectionInfo,
    VoIP,
    Map,
    NoitaLog,
    ProxyLog,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
struct PlayerAppearance {
    player_color: PlayerColor,
    player_picker: PlayerPicker,
    hue: f64,
    cosmetics: (bool, bool, bool),
    invert_border: bool,
}

impl PlayerAppearance {
    fn create_png_desc(&self, game_save_path: Option<PathBuf>) -> PlayerPngDesc {
        let mut cosmetics = self.cosmetics;
        if let Some(path) = &game_save_path {
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
        PlayerPngDesc {
            cosmetics: cosmetics.into(),
            colors: self.player_color,
            invert_border: self.invert_border,
        }
    }
    fn mina_color_picker(
        &mut self,
        ui: &mut Ui,
        game_save_path: Option<PathBuf>,
        player_image: RgbaImage,
    ) {
        let old_hue = self.hue;
        let old = ui.style_mut().spacing.slider_width;
        ui.style_mut().spacing.slider_width = 256.0;
        ui.add(
            Slider::new(&mut self.hue, 0.0..=360.0)
                .text(tr("Shift-hue"))
                .min_decimals(0)
                .max_decimals(0)
                .step_by(2.0),
        );
        ui.style_mut().spacing.slider_width = old;
        if old_hue != self.hue {
            let diff = self.hue - old_hue;
            match self.player_picker {
                PlayerPicker::PlayerAlt => {
                    shift_hue(diff, &mut self.player_color.player_alt);
                }
                PlayerPicker::PlayerArm => {
                    shift_hue(diff, &mut self.player_color.player_arm);
                }
                PlayerPicker::PlayerCape => {
                    shift_hue(diff, &mut self.player_color.player_cape);
                }
                PlayerPicker::PlayerForearm => {
                    shift_hue(diff, &mut self.player_color.player_forearm);
                }
                PlayerPicker::PlayerCapeEdge => {
                    shift_hue(diff, &mut self.player_color.player_cape_edge);
                }
                PlayerPicker::PlayerMain => {
                    shift_hue(diff, &mut self.player_color.player_main);
                }
                PlayerPicker::None => {
                    shift_hue(diff, &mut self.player_color.player_main);
                    shift_hue(diff, &mut self.player_color.player_alt);
                    shift_hue(diff, &mut self.player_color.player_arm);
                    shift_hue(diff, &mut self.player_color.player_forearm);
                    shift_hue(diff, &mut self.player_color.player_cape);
                    shift_hue(diff, &mut self.player_color.player_cape_edge);
                }
            }
        }
        ui.horizontal(|ui| {
            display_player_skin(
                ui,
                get_player_skin(
                    player_image.clone(),
                    self.create_png_desc(game_save_path.clone()),
                ),
                12.0,
            );
            player_select_current_color_slot(ui, self, game_save_path.clone());
            player_skin_display_color_picker(ui, &mut self.player_color, &self.player_picker);
        });
        if ui.button(tr("Reset-colors-to-default")).clicked() {
            self.hue = 0.0;
            self.player_color = Default::default();
        }
    }
}

impl Default for PlayerAppearance {
    fn default() -> Self {
        Self {
            player_color: PlayerColor::default(),
            player_picker: PlayerPicker::None,
            hue: 0.0,
            cosmetics: (true, true, true),
            invert_border: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
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
    spacewars: bool,
    random_ports: bool,
    public_lobby: bool,
    allow_friends: bool,
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
            spacewars: false,
            random_ports: false,
            public_lobby: false,
            allow_friends: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Decode, Encode, Copy, Clone)]
pub struct PlayerColor {
    player_main: [f64; 4],
    player_alt: [f64; 4],
    player_arm: [f64; 4],
    player_cape: [f64; 4],
    player_cape_edge: [f64; 4],
    player_forearm: [f64; 4],
}

impl Default for PlayerColor {
    fn default() -> Self {
        Self {
            player_main: [155.0, 111.0, 154.0, 255.0],
            player_alt: [127.0, 84.0, 118.0, 255.0],
            player_arm: [89.0, 67.0, 84.0, 255.0],
            player_cape: [118.0, 84.0, 127.0, 255.0],
            player_cape_edge: [154.0, 111.0, 155.0, 255.0],
            player_forearm: [158.0, 115.0, 154.0, 255.0],
        }
    }
}
/*impl PlayerColor {
    pub fn notplayer() -> Self {
        Self {
            player_main: [155.0, 111.0, 154.0, 255.0],
            player_alt: [127.0, 84.0, 118.0, 255.0],
            player_arm: [89.0, 67.0, 84.0, 255.0],
            player_cape: [118.0, 84.0, 127.0, 255.0],
            player_cape_edge: [154.0, 111.0, 155.0, 255.0],
            player_forearm: [158.0, 115.0, 154.0, 255.0],
        }
    }
}*/
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

#[derive(Default)]
struct EndRunButton {
    end_run_confirmation: bool,
}
impl EndRunButton {
    fn show(&mut self, ui: &mut Ui, netman: &mut NetManStopOnDrop) {
        ui.horizontal(|ui| {
            let dirty = netman.dirty.load(Ordering::Relaxed);
            let button = Button::new(tr("launcher_end_run"))
                .small()
                .fill(Color32::LIGHT_RED);
            if !self.end_run_confirmation
                && if dirty {
                    ui.add(button).clicked()
                } else {
                    ui.button(tr("launcher_end_run")).clicked()
                }
            {
                self.end_run_confirmation = true
            } else if self.end_run_confirmation
                && ui.button(tr("launcher_end_run_confirm")).clicked()
            {
                self.end_run_confirmation = false;
                netman.end_run.store(true, Ordering::Relaxed);
            };
            if dirty {
                ui.label("PENDING SETTINGS NOT SET UNTIL RUN ENDS");
            }
        });
    }
}

#[derive(Debug, Serialize, Deserialize, Decode, Encode, Clone)]
#[serde(default)]
pub struct AudioSettings {
    volume: HashMap<OmniPeerId, f32>,
    dropoff: f32,
    range: u64,
    //walls_strength: f32,
    //max_wall_durability: u32,
    player_position: bool,
    global: bool,
    push_to_talk: bool,
    mute_out: bool,
    mute_in: bool,
    mute_in_while_polied: bool,
    mute_in_while_dead: bool,
    disabled: bool,
    loopback: bool,
    global_output_volume: f32,
    global_input_volume: f32,
    input_device: Option<String>,
    output_device: Option<String>,
    input_devices: Vec<String>,
    output_devices: Vec<String>,
}

impl AudioSettings {
    fn show_ui(&mut self, ui: &mut Ui, main: bool) -> bool {
        let mut changed = false;
        ui.label("drop off rate of audio from others");
        changed |= ui
            .add(Slider::new(&mut self.dropoff, 0.0..=128.0))
            .changed();
        ui.label("maximal range of audio");
        changed |= ui.add(Slider::new(&mut self.range, 0..=4096)).changed();
        ui.label("global input volume");
        changed |= ui
            .add(Slider::new(&mut self.global_input_volume, 0.0..=8.0))
            .changed();
        ui.label("global output volume");
        changed |= ui
            .add(Slider::new(&mut self.global_output_volume, 0.0..=8.0))
            .changed();
        changed |= ui.checkbox(&mut self.loopback, "loopback audio").changed();
        changed |= ui
            .checkbox(&mut self.global, "have voice always be played")
            .changed();
        changed |= ui
            .checkbox(
                &mut self.push_to_talk,
                "push to talk, keybinds in noita, T by default",
            )
            .changed();
        changed |= ui
            .checkbox(
                &mut self.player_position,
                "use player position rather than camera position",
            )
            .changed();
        changed |= ui.checkbox(&mut self.mute_in, "mute input").changed();
        changed |= ui
            .checkbox(&mut self.mute_in_while_polied, "mute input while polied")
            .changed();
        changed |= ui
            .checkbox(&mut self.mute_in_while_dead, "mute input while dead")
            .changed();
        changed |= ui.checkbox(&mut self.mute_out, "mute output").changed();
        if main {
            changed |= ui.checkbox(&mut self.disabled, "disabled").changed();
            if self.input_devices.is_empty() {
                #[cfg(target_os = "linux")]
                let host = cpal::available_hosts()
                    .into_iter()
                    .find(|id| *id == cpal::HostId::Jack)
                    .and_then(|id| cpal::host_from_id(id).ok())
                    .unwrap_or(cpal::default_host());
                #[cfg(not(target_os = "linux"))]
                let host = cpal::default_host();
                self.input_devices = host
                    .input_devices()
                    .map(|devices| devices.filter_map(|d| d.name().ok()).collect())
                    .unwrap_or_default();
                self.output_devices = host
                    .output_devices()
                    .map(|devices| devices.filter_map(|d| d.name().ok()).collect())
                    .unwrap_or_default();
                if self.input_device.is_none() {
                    self.input_device = host.default_input_device().and_then(|a| a.name().ok())
                }
                if self.output_device.is_none() {
                    self.output_device = host.default_output_device().and_then(|a| a.name().ok())
                }
            }
            ComboBox::from_label("Input Device")
                .selected_text(
                    self.input_device
                        .clone()
                        .unwrap_or_else(|| "None".to_string()),
                )
                .show_ui(ui, |ui| {
                    for device in &self.input_devices {
                        if ui
                            .selectable_label(self.input_device.as_deref() == Some(device), device)
                            .clicked()
                        {
                            self.input_device = Some(device.clone());
                            changed = true;
                        }
                    }
                });
            ComboBox::from_label("Output Device")
                .selected_text(
                    self.output_device
                        .clone()
                        .unwrap_or_else(|| "None".to_string()),
                )
                .show_ui(ui, |ui| {
                    for device in &self.output_devices {
                        if ui
                            .selectable_label(self.output_device.as_deref() == Some(device), device)
                            .clicked()
                        {
                            self.output_device = Some(device.clone());
                            changed = true;
                        }
                    }
                });
        }
        if ui.button("default").clicked() {
            *self = Default::default();
            changed = true;
        }
        changed
    }
}
struct ImageMap {
    textures: FxHashMap<ChunkCoord, TextureHandle>,
    zoom: f32,
    offset: Vec2,
    players: FxHashMap<OmniPeerId, (Option<WorldPos>, bool, bool, TextureHandle)>,
    notplayer: Option<TexturePoll>,
    centered_on: Option<OmniPeerId>,
    dont_scale: bool,
}
impl Default for ImageMap {
    fn default() -> Self {
        Self {
            textures: FxHashMap::default(),
            zoom: 1.0,
            offset: Vec2::new(f32::MAX, f32::MAX),
            players: Default::default(),
            notplayer: None,
            centered_on: None,
            dont_scale: false,
        }
    }
}
impl ImageMap {
    fn update_textures(
        &mut self,
        ui: &mut Ui,
        map: &FxHashMap<ChunkCoord, RgbaImage>,
        ctx: &Context,
    ) {
        for (coord, img) in map {
            let name = format!("{}x{}", coord.0, coord.1);
            if self.textures.contains_key(coord) {
                ctx.forget_image(&name)
            }
            let size = [img.width() as usize, img.height() as usize];
            let color_image =
                egui::ColorImage::from_rgba_unmultiplied(size, img.as_flat_samples().as_slice());
            let tex = ui
                .ctx()
                .load_texture(name, color_image, TextureOptions::NEAREST);
            self.textures.insert(*coord, tex);
        }
    }
    fn update_player_textures(
        &mut self,
        ui: &mut Ui,
        map: &FxHashMap<OmniPeerId, (Option<WorldPos>, bool, bool, RgbaImage)>,
    ) {
        for (p, (coord, is_dead, does_exist, img)) in map {
            if !self.players.contains_key(p) {
                let name = format!("{}", p);
                let size = [img.width() as usize, img.height() as usize];
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    size,
                    img.as_flat_samples().as_slice(),
                );
                let tex = ui
                    .ctx()
                    .load_texture(name, color_image, TextureOptions::NEAREST);
                self.players
                    .insert(*p, (*coord, *is_dead, *does_exist, tex));
            }
            self.players.entry(*p).and_modify(|(w, b, d, _)| {
                *w = *coord;
                *b = *is_dead;
                *d = *does_exist;
            });
        }
    }
    fn ui(&mut self, ui: &mut Ui, netman: &NetManStopOnDrop, ctx: &Context) {
        if self.offset == Vec2::new(f32::MAX, f32::MAX) {
            self.offset = Vec2::new(ui.available_width() / 2.0, ui.available_height() / 2.0);
        }
        if netman.reset_map.load(Ordering::Relaxed) {
            netman.reset_map.store(false, Ordering::Relaxed);
            self.textures.clear();
        }
        {
            let map = &mut netman.chunk_map.lock().unwrap();
            if !map.is_empty() {
                self.update_textures(ui, map, ctx);
            }
            map.clear();
        }
        if self.notplayer.is_none() {
            self.notplayer = egui::include_image!("../assets/notplayer.png")
                .load(ctx, TextureOptions::NEAREST, SizeHint::Size(7, 17))
                .ok();
        }
        {
            self.update_player_textures(ui, &netman.players_sprite.lock().unwrap());
        }
        let response = ui.interact(
            ui.available_rect_before_wrap(),
            ui.id().with("map_interact"),
            Sense::drag(),
        );
        if response.dragged() {
            self.offset += response.drag_delta();
        }

        if ui.input(|i| i.raw_scroll_delta.y) != 0.0 {
            let mouse_pos = ui.input(|i| i.pointer.latest_pos().unwrap_or_default());
            let mouse_relative = mouse_pos - self.offset;
            let zoom_factor = 2.0_f32.powf(ui.input(|i| i.raw_scroll_delta.y / 256.0));
            self.zoom *= zoom_factor;
            let new_mouse_relative = mouse_relative * zoom_factor;
            self.offset = mouse_pos - new_mouse_relative;
        }
        let s = 32.0;
        if ui.input(|i| i.keys_down.contains(&Key::W) || i.keys_down.contains(&Key::ArrowUp)) {
            self.offset.y += s
        }
        if ui.input(|i| i.keys_down.contains(&Key::S) || i.keys_down.contains(&Key::ArrowDown)) {
            self.offset.y -= s
        }
        if ui.input(|i| i.keys_down.contains(&Key::A) || i.keys_down.contains(&Key::ArrowLeft)) {
            self.offset.x += s
        }
        if ui.input(|i| i.keys_down.contains(&Key::D) || i.keys_down.contains(&Key::ArrowRight)) {
            self.offset.x -= s
        }
        if ui.input(|i| i.key_released(Key::Q)) {
            self.zoom *= 2.0 / 3.0
        }
        if ui.input(|i| i.key_released(Key::E)) {
            self.zoom *= 3.0 / 2.0
        }
        if ui.input(|i| i.key_released(Key::X)) {
            self.dont_scale = !self.dont_scale
        }
        let q = ui.input(|i| i.key_released(Key::Z));
        let e = ui.input(|i| i.key_released(Key::C));
        if q || e {
            let players: Vec<OmniPeerId> = self
                .players
                .iter()
                .filter_map(|(a, (c, _, d, _))| if c.is_some() && !d { Some(a) } else { None })
                .cloned()
                .collect();
            self.centered_on = if !players.is_empty() {
                if let Some(id) = self.centered_on {
                    if let Some(i) = players.iter().position(|o| *o == id) {
                        let i = if q { i as i32 - 1 } else { i as i32 + 1 }
                            .rem_euclid(players.len() as i32 + 1)
                            as usize;
                        if i == players.len() {
                            None
                        } else {
                            Some(players[i])
                        }
                    } else {
                        Some(players[0])
                    }
                } else if q {
                    Some(players[players.len() - 1])
                } else {
                    Some(players[0])
                }
            } else {
                None
            }
        }
        let tile_size = self.zoom * 128.0;
        if let Some(peer) = self.centered_on {
            if let Some((Some(pos), _, _, _)) = self.players.get(&peer) {
                self.offset = Vec2::new(ui.available_width() / 2.0, ui.available_height() / 2.0)
                    - Vec2::new(
                        pos.x as f32 * tile_size / 128.0,
                        (pos.y - 12) as f32 * tile_size / 128.0,
                    )
            }
        }
        let painter = ui.painter();
        for (coord, tex) in &self.textures {
            let pos =
                self.offset + Vec2::new(coord.0 as f32 * tile_size, coord.1 as f32 * tile_size);
            let rect = Rect::from_min_size(pos.to_pos2(), Vec2::splat(tile_size));
            painter.image(
                tex.id(),
                rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        for (pos, is_dead, does_exist, tex) in self.players.values() {
            if *does_exist {
                continue;
            }
            if let Some(pos) = pos {
                let pos = self.offset
                    + Vec2::new(
                        pos.x as f32 * tile_size / 128.0,
                        (pos.y - 12) as f32 * tile_size / 128.0,
                    );
                let mut tile_size = tile_size;
                if self.dont_scale && self.zoom < 1.0 {
                    tile_size = 128.0
                }
                let rect = Rect::from_min_size(
                    pos.to_pos2(),
                    Vec2::new(7.0 * tile_size / 128.0, 16.0 * tile_size / 128.0),
                );
                if *is_dead {
                    if let Some(tex) = &self.notplayer {
                        if let Some(id) = tex.texture_id() {
                            painter.image(
                                id,
                                rect,
                                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                Color32::WHITE,
                            );
                        }
                    }
                } else {
                    painter.image(
                        tex.id(),
                        rect,
                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }
            }
        }
    }
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            volume: Default::default(),
            dropoff: 1.0,
            range: 1024,
            global: false,
            //walls_strength: 1.0,
            //max_wall_durability: 14,
            player_position: true,
            push_to_talk: true,
            mute_out: false,
            mute_in: false,
            mute_in_while_polied: true,
            mute_in_while_dead: false,
            disabled: false,
            input_device: None,
            output_device: None,
            input_devices: Vec::new(),
            output_devices: Vec::new(),
            global_output_volume: 1.0,
            global_input_volume: 1.0,
            loopback: false,
        }
    }
}

pub struct App {
    state: AppState,
    audio: AudioSettings,
    modmanager: Modmanager,
    steam_state: Result<steam_helper::SteamState, SteamAPIInitError>,
    app_saved_state: AppSavedState,
    run_save_state: SaveState,
    modmanager_settings: ModmanagerSettings,
    self_update: SelfUpdateManager,
    lobby_id_field: String,
    args: Args,
    /// `true` if we haven't started noita automatically yet.
    can_start_automatically: bool,
    player_image: RgbaImage,
    end_run_button: EndRunButton,
    appearance: PlayerAppearance,
    connected_menu: ConnectedMenu,
    show_host_settings: bool,
    show_audio_settings: bool,
    running_on_steamdeck: bool,
    copied_lobby: bool,
    my_lobby_kind: LobbyKind,
    show_lobby_list: bool,
    map: ImageMap,
    refresh_timer: time::Instant,
    noitalog_number: usize,
    noitalog: Vec<String>,
    proxylog: String,
    save_paths: SavePaths,
    clipboard: Option<Clipboard>,
}

fn filled_group<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    let style = ui.style();
    let frame = egui::Frame {
        inner_margin: Margin::same(6), // same and symmetric looks best in corners when nesting groups
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
    ui.add_sized(
        [side, side],
        ImageButton::new(icon), // Somewhy it doesnt inherit style correctly
    )
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Settings {
    color: PlayerAppearance,
    app: AppSavedState,
    modmanager: ModmanagerSettings,
    audio: AudioSettings,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, args: Args) -> Self {
        cc.egui_ctx.set_visuals(Visuals::dark());
        cc.egui_ctx.set_theme(ThemePreference::Dark);
        let save_paths = SavePaths::new();
        let settings = save_paths.load_settings();
        let mut saved_state: AppSavedState = settings.app;
        let modmanager_settings: ModmanagerSettings = settings.modmanager;
        let appearance: PlayerAppearance = settings.color;
        let audio: AudioSettings = settings.audio;
        saved_state.times_started += 1;

        info!("Setting fonts...");
        Self::set_fonts(&cc.egui_ctx);
        let state = if let Some(lang_id) = &saved_state.lang_id {
            set_current_locale(lang_id.clone());
            AppState::ModManager
        } else {
            AppState::LangPick
        };

        info!("Installing image loaders...");
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let my_lobby_kind = args.override_lobby_kind.unwrap_or({
            if saved_state.spacewars {
                LobbyKind::Gog
            } else {
                LobbyKind::Steam
            }
        });

        info!("Initializing steam state...");
        let steam_state = steam_helper::SteamState::new(my_lobby_kind == LobbyKind::Gog);

        info!("Checking if running on steam deck...");
        let running_on_steamdeck = steam_state
            .as_ref()
            .map(|steam| steam.client.utils().is_steam_running_on_steam_deck())
            .unwrap_or(false);
        let default_zoom_factor = if running_on_steamdeck { 0.3 } else { 1.0 };
        if running_on_steamdeck {
            cc.egui_ctx
                .send_viewport_cmd(egui::ViewportCommand::MinInnerSize(Vec2::new(10.0, 10.0)));
            cc.egui_ctx
                .send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
        }

        cc.egui_ctx
            .set_zoom_factor(args.ui_zoom_factor.unwrap_or(default_zoom_factor));
        info!("Creating the app...");
        let run_save_state = SaveState::new(&save_paths.save_state_path);
        let path = player_path(modmanager_settings.mod_path());
        let player_image = if path.exists() {
            image::open(path)
                .unwrap_or(ImageRgba8(RgbaImage::new(20, 20)))
                .crop(1, 1, 7, 16)
                .into_rgba8()
        } else {
            RgbaImage::new(7, 17)
        };

        let mut me = Self {
            state,
            audio,
            modmanager: Modmanager::default(),
            steam_state,
            app_saved_state: saved_state,
            modmanager_settings,
            self_update: SelfUpdateManager::new(),
            lobby_id_field: "".to_string(),
            args,
            can_start_automatically: false,
            run_save_state,
            player_image,
            end_run_button: EndRunButton::default(),
            appearance,
            connected_menu: ConnectedMenu::Normal,
            show_host_settings: false,
            show_audio_settings: false,
            running_on_steamdeck,
            copied_lobby: true,
            my_lobby_kind,
            show_lobby_list: false,
            map: Default::default(),
            refresh_timer: time::Instant::now(),
            noitalog_number: 0,
            noitalog: Vec::new(),
            proxylog: String::new(),
            save_paths,
            clipboard: Clipboard::new().ok(),
        };

        if let Some(connect_to) = me.args.auto_connect_to {
            me.start_steam_connect(connect_to.code);
        }

        me
    }

    fn set_settings(&self) {
        let mut audio = self.audio.clone();
        audio.input_devices.clear();
        audio.output_devices.clear();
        self.save_paths.save_settings(Settings {
            color: self.appearance.clone(),
            app: self.app_saved_state.clone(),
            modmanager: self.modmanager_settings.clone(),
            audio,
        });
    }

    fn get_netman_init(&self) -> NetManagerInit {
        let my_nickname = self.nickname();
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
        let noita_port = if self.app_saved_state.random_ports {
            0
        } else {
            21251
        };

        NetManagerInit {
            my_nickname,
            save_state: self.run_save_state.clone(),
            cosmetics,
            mod_path,
            player_path: player_path(self.modmanager_settings.mod_path()),
            modmanager_settings: self.modmanager_settings.clone(),
            player_png_desc: PlayerPngDesc {
                cosmetics: cosmetics.into(),
                colors: self.appearance.player_color,
                invert_border: self.appearance.invert_border,
            },
            noita_port,
        }
    }

    fn nickname(&self) -> String {
        let default = "no name found".to_string();
        let steam_nickname = if let Ok(steam) = &self.steam_state {
            let sid = steam.get_my_id();
            Some(steam.get_user_name(sid))
        } else {
            None
        };
        let mut my_nickname = self.app_saved_state.nickname.clone().or(steam_nickname);
        if let Some(n) = &my_nickname {
            if n.trim().is_empty() {
                my_nickname = None;
            }
        }

        my_nickname.unwrap_or(default)
    }

    fn change_state_to_netman(&mut self, netman: Arc<net::NetManager>, player_path: PathBuf) {
        self.copied_lobby = false;
        let handle = netman.clone().start(player_path);
        self.state = AppState::ConnectedLobby {
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
        let bind_addr = SocketAddr::new("::".parse().unwrap(), DEFAULT_PORT);
        let peer = Peer::host(bind_addr, None).unwrap();
        let netman = net::NetManager::new(
            PeerVariant::Tangled(peer),
            self.get_netman_init(),
            self.audio.clone(),
        );
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
    }

    fn start_connect(&mut self, addr: SocketAddr) {
        let peer = Peer::connect(addr, None).unwrap();
        self.state = AppState::TangledConnecting { peer };
    }

    fn start_connect_step_2(&mut self, peer: Peer) {
        let netman = net::NetManager::new(
            PeerVariant::Tangled(peer),
            self.get_netman_init(),
            self.audio.clone(),
        );
        self.change_state_to_netman(netman, player_path(self.modmanager_settings.mod_path()));
    }

    fn start_steam_host(&mut self) {
        let lobby_type = if self.app_saved_state.public_lobby {
            steamworks::LobbyType::Public
        } else if self.app_saved_state.allow_friends {
            steamworks::LobbyType::FriendsOnly
        } else {
            steamworks::LobbyType::Private
        };
        let peer = net::steam_networking::SteamPeer::new_host(
            lobby_type,
            self.steam_state.as_ref().unwrap().client.clone(),
            self.app_saved_state
                .game_settings
                .max_players
                .unwrap_or(DefaultSettings::default().max_players),
            self.make_lobby_extra_data(),
        );
        let netman = net::NetManager::new(
            PeerVariant::Steam(peer),
            self.get_netman_init(),
            self.audio.clone(),
        );
        self.set_netman_settings(&netman);
        self.change_state_to_netman(netman, player_path(self.modmanager_settings.mod_path()));
    }

    fn make_lobby_extra_data(&self) -> LobbyExtraData {
        LobbyExtraData {
            name: self.nickname(),
            game_mode: Some(
                self.app_saved_state
                    .game_settings
                    .game_mode
                    .unwrap_or(DefaultSettings::default().game_mode),
            ),
        }
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

        let netman = net::NetManager::new(
            PeerVariant::Steam(peer),
            self.get_netman_init(),
            self.audio.clone(),
        );
        self.change_state_to_netman(netman, player_path(self.modmanager_settings.mod_path()));
    }

    fn connect_screen(&mut self, ctx: &Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.app_saved_state.times_started % 20 == 0 {
                let image = egui::Image::new(egui::include_image!("../assets/longleg.png"))
                    .texture_options(TextureOptions::NEAREST);
                image.paint_at(ui, ctx.screen_rect());
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
            let (steam_connect_rect, other_rect) = right.split_top_bottom_at_fraction(0.33);
            let (ip_connect_rect, info_rect) = other_rect.split_top_bottom_at_fraction(0.5);

            ui.allocate_new_ui(
                UiBuilder {
                    max_rect: Some(bottom_panel.shrink(group_shrink)),
                    ..Default::default()
                },
                |ui| {
                    filled_group(ui, |ui| {
                        ui.set_min_size(ui.available_size());
                        self.self_update.display_version(ui);

                        if self.self_update.request_update {
                            self.state = AppState::SelfUpdate;
                        }
                    });
                },
            );

            ui.allocate_new_ui(
                UiBuilder {
                    max_rect: Some(info_rect.shrink(group_shrink)),
                    ..Default::default()
                },
                |ui| {
                    filled_group(ui, |ui| {
                        ui.set_min_size(ui.available_size());
                        // heading_with_underline(ui, tr("Info"));
                        // ui.label(tr("info_stress_tests"));
                    });
                },
            );

            ui.allocate_new_ui(
                UiBuilder {
                    max_rect: Some(right_b_panel.shrink(group_shrink)),
                    ..Default::default()
                },
                |ui| {
                    filled_group(ui, |ui| {
                        ui.set_min_size(ui.available_size());

                        self.panel_right_bar(ui, ctx);
                    })
                },
            );

            ui.allocate_new_ui(
                UiBuilder {
                    max_rect: Some(settings_rect.shrink(group_shrink)),
                    ..Default::default()
                },
                |ui| {
                    filled_group(ui, |ui| {
                        ui.set_min_size(ui.available_size());
                        ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                            self.show_local_settings(ui);
                            if ui.button("Show host settings").clicked() {
                                self.show_host_settings = !self.show_host_settings
                            }
                            if self.show_host_settings {
                                self.app_saved_state.game_settings.show_editor(ui, true)
                            }
                            if ui.button("Show audio settings").clicked() {
                                self.show_audio_settings = !self.show_audio_settings
                            }
                            if self.show_audio_settings {
                                self.audio.show_ui(ui, true);
                            }
                            if self.running_on_steamdeck && ui.button("Close Proxy").clicked() {
                                exit(0)
                            }
                        });
                    });
                },
            );
            ui.allocate_new_ui(
                UiBuilder {
                    max_rect: Some(steam_connect_rect.shrink(group_shrink)),
                    ..Default::default()
                },
                |ui| {
                    filled_group(ui, |ui| {
                        ui.set_min_size(ui.available_size());

                        self.panel_connect_by_steam(ui);
                    });
                },
            );
            ui.allocate_new_ui(
                UiBuilder {
                    max_rect: Some(ip_connect_rect.shrink(group_shrink)),
                    ..Default::default()
                },
                |ui| {
                    filled_group(ui, |ui| {
                        ui.set_min_size(ui.available_size());

                        self.panel_connect_by_ip(ui);
                    });
                },
            );
        });

        if self.show_lobby_list {
            let mut connect_to = None;
            Window::new(tr("Lobby-list"))
                .open(&mut self.show_lobby_list)
                .show(ctx, |ui| {
                    ui.set_min_height(100.0);
                    let steam_state = self.steam_state.as_mut().unwrap();
                    ui.horizontal(|ui| {
                        if ui.button(tr("Refresh")).clicked() {
                            steam_state.update_lobby_list(&mut self.refresh_timer);
                        }
                    });
                    ScrollArea::vertical().show(ui, |ui| {
                        match steam_state.list_lobbies(&mut self.refresh_timer) {
                            steam_helper::MaybeLobbyList::Pending => {
                                ui.label(tr("Lobby-list-pending"));
                            }
                            steam_helper::MaybeLobbyList::List(lobby_ids) => {
                                let mut shown_anything = false;
                                for id in lobby_ids.iter() {
                                    let info = steam_state.lobby_info(*id);
                                    if info.version != Some(Version::current()) {
                                        continue;
                                    }
                                    shown_anything = true;
                                    ui.group(|ui| {
                                        ui.set_max_height(50.0);
                                        ui.horizontal(|ui| {
                                            ui.colored_label(Color32::WHITE, info.data.name);
                                            ui.with_layout(
                                                Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    ui.label(format!(
                                                        "{}/{}",
                                                        info.member_count, info.member_limit
                                                    ));
                                                    if let Some(game_mode) = &info.data.game_mode {
                                                        ui.label(" - ");
                                                        let color = game_mode.color();
                                                        ui.colored_label(
                                                            color,
                                                            tr(&format!("game_mode_{}", game_mode)),
                                                        );
                                                    }
                                                },
                                            );
                                        });
                                        ui.horizontal(|ui| {
                                            let mut enabled = false;
                                            if let Some(version) = info.version {
                                                enabled = version == Version::current();
                                                let color = if enabled {
                                                    Color32::GRAY
                                                } else {
                                                    Color32::RED
                                                };
                                                ui.colored_label(color, format!("EW {}", version));
                                            } else if info.is_noita_online {
                                                ui.colored_label(
                                                    Color32::LIGHT_BLUE,
                                                    "Noita Online",
                                                );
                                            } else {
                                                ui.label(tr("Not-Entangled-Worlds-lobby"));
                                            }
                                            ui.with_layout(
                                                Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    ui.add_enabled_ui(enabled, |ui| {
                                                        if ui.small_button(tr("Join")).clicked() {
                                                            connect_to = Some(*id);
                                                        }
                                                    });
                                                },
                                            );
                                        });
                                    });
                                }
                                if !shown_anything {
                                    ui.label(tr("No-public-lobbies-at-the-moment"));
                                }
                            }
                            steam_helper::MaybeLobbyList::Errored => {
                                ui.label("Failed to request lobby list");
                            }
                        }
                    });
                });
            if let Some(lobby) = connect_to {
                self.start_steam_connect(lobby);
            }
        }
    }

    fn panel_right_bar(&mut self, ui: &mut Ui, ctx: &Context) {
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
        if square_button_icon(
            ui,
            egui::Image::new(egui::include_image!("../assets/discord-mark-white.png")),
        )
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
    }

    fn panel_connect_by_steam(&mut self, ui: &mut Ui) {
        heading_with_underline(ui, tr("connect_steam"));

        match &self.steam_state {
            Ok(_) => {
                if ui.button(tr("connect_steam_create")).clicked() {
                    self.set_settings();
                    self.start_steam_host();
                }
                ui.checkbox(
                    &mut self.app_saved_state.public_lobby,
                    tr("Make-lobby-public"),
                );
                ui.checkbox(&mut self.app_saved_state.allow_friends, "Allow friends");
                if ui.button(tr("connect_steam_connect")).clicked() {
                    let id = self.clipboard.as_mut().and_then(|c| c.get_text().ok());
                    match id {
                        Some(id) => {
                            self.set_settings();
                            self.connect_to_steam_lobby(id);
                        }
                        None => self.notify_error(if self.clipboard.is_none() {
                            "no clipboard"
                        } else {
                            "clipboard failed"
                        }),
                    }
                }
                if ui.button(tr("Open-lobby-list")).clicked() {
                    self.show_lobby_list = true;
                }

                if cfg!(target_os = "linux") {
                    ui.add_space(15.0);
                    ui.label(tr("connect_steam_workaround_label"));
                    ui.text_edit_singleline(&mut self.lobby_id_field);
                    if ui.button(tr("connect_steam_connect_2")).clicked() {
                        self.set_settings();
                        self.connect_to_steam_lobby(self.lobby_id_field.clone());
                    }
                }
            }
            Err(err) => {
                ui.label(format!("Could not init steam networking: {}", err));
            }
        }
    }

    fn panel_connect_by_ip(&mut self, ui: &mut Ui) {
        heading_with_underline(ui, tr("connect_ip"));

        ui.label(tr("ip_note"));
        if ui.button(tr("ip_host")).clicked() {
            self.set_settings();
            self.start_server();
        }

        ui.text_edit_singleline(&mut self.app_saved_state.addr);
        let addr = self.app_saved_state.addr.parse();

        let ip: Result<IpAddr, _> = self.app_saved_state.addr.parse();
        let addr2 = ip.map(|ip| SocketAddr::new(ip, DEFAULT_PORT));

        let addr = addr.or(addr2);

        ui.add_enabled_ui(addr.is_ok(), |ui| {
            if ui.button(tr("ip_connect")).clicked() {
                if let Ok(addr) = addr {
                    self.set_settings();
                    self.start_connect(addr);
                }
            }
        });
    }

    fn show_local_settings(&mut self, ui: &mut Ui) {
        heading_with_underline(ui, tr("connect_settings_local"));
        {
            let mut temp = self.app_saved_state.nickname.clone().unwrap_or(
                if let Ok(steam) = &self.steam_state {
                    steam.get_user_name(steam.get_my_id())
                } else {
                    "NO NICKNAME CHOSEN".to_string()
                },
            );
            if ui
                .horizontal(|ui| {
                    ui.label("nickname");
                    ui.text_edit_singleline(&mut temp)
                })
                .inner
                .changed()
            {
                self.app_saved_state.nickname = Some(temp)
            }
        }
        ui.checkbox(
            &mut self.app_saved_state.start_game_automatically,
            tr("connect_settings_autostart"),
        );
        ui.checkbox(
            &mut self.app_saved_state.spacewars,
            tr("connect_settings_spacewars"),
        );
        ui.checkbox(
            &mut self.app_saved_state.random_ports,
            tr("connect_settings_random_ports"),
        );
        if self.player_image.width() == 1 {
            self.player_image = image::open(player_path(self.modmanager_settings.mod_path()))
                .unwrap_or(ImageRgba8(RgbaImage::new(20, 20)))
                .crop(1, 1, 7, 16)
                .into_rgba8();
        }
        ui.add_space(20.0);
        self.appearance.mina_color_picker(
            ui,
            self.modmanager_settings.game_save_path.clone(),
            self.player_image.clone(),
        );
    }

    fn connect_to_steam_lobby(&mut self, lobby_id_raw: String) {
        let lobby_id_raw = lobby_id_raw
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>();
        let lobby = LobbyCode::parse(&lobby_id_raw);

        match lobby {
            Ok(lobby) => {
                if self.my_lobby_kind == lobby.kind {
                    self.start_steam_connect(lobby.code)
                } else {
                    self.state = AppState::GogModeIssue(lobby);
                }
            }
            Err(LobbyError::NotALobbyCode) => {
                self.notify_error(tr("connect_steam_connect_invalid_lobby_id"))
            }
            Err(LobbyError::CodeVersionMismatch) => self
                .notify_error("Lobby code was created by a newer version of proxy. Please update"),
        }
    }

    fn set_fonts(ctx: &Context) {
        let mut font_definitions = FontDefinitions::default();

        font_definitions.font_data.insert(
            "noto_sans".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/font/NotoSans-Regular.ttf"))
                .into(),
        );
        font_definitions.font_data.insert(
            "noto_sans_jp".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/font/NotoSansJP-Light.ttf"))
                .into(),
        );
        font_definitions.font_data.insert(
            "noto_sans_sc".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/font/MiSans-Light.ttf")).into(),
        );
        font_definitions.font_data.insert(
            "noto_sans_kr".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/font/NotoSansKR-Light.ttf"))
                .into(),
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

        font_definitions
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .push("noto_sans_sc".to_owned());
        font_definitions
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .push("noto_sans_sc".to_owned());

        font_definitions
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .push("noto_sans_kr".to_owned());
        font_definitions
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .push("noto_sans_kr".to_owned());

        ctx.set_fonts(font_definitions);
    }

    fn switch_to_connect(&mut self) {
        self.state = if self.run_save_state.has_savestate() {
            AppState::AskSavestateReset
        } else {
            AppState::Connect
        };
    }

    fn show_lobby(&mut self, ctx: &Context) {
        let AppState::ConnectedLobby {
            netman,
            noita_launcher,
        } = &mut self.state
        else {
            panic!("Called in incorrect state");
        };
        let mut goto_menu = false;
        let stopped = netman.stopped.load(Ordering::Relaxed);
        let accept_local = netman.accept_local.load(Ordering::Relaxed);
        let local_connected = netman.local_connected.load(Ordering::Relaxed);
        egui::TopBottomPanel::bottom("noita_status").show(ctx, |ui| {
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
            .exact_width(230.0)
            .show(ctx, |ui| {
                ui.add_space(3.0);
                show_player_list(ui, netman);
            });
        let mut update_lobby_data = false;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let last = self.connected_menu;
                ui.selectable_value(&mut self.connected_menu, ConnectedMenu::Normal, "Lobby");
                ui.selectable_value(
                    &mut self.connected_menu,
                    ConnectedMenu::Settings,
                    "Game Settings",
                );
                ui.selectable_value(
                    &mut self.connected_menu,
                    ConnectedMenu::VoIP,
                    "VoIP Settings",
                );
                ui.selectable_value(&mut self.connected_menu, ConnectedMenu::Map, "Chunk Map");
                ui.selectable_value(
                    &mut self.connected_menu,
                    ConnectedMenu::NoitaLog,
                    "Noita Log",
                );
                ui.selectable_value(
                    &mut self.connected_menu,
                    ConnectedMenu::ProxyLog,
                    "Proxy Log",
                );
                if netman.peer.is_steam() {
                    ui.selectable_value(
                        &mut self.connected_menu,
                        ConnectedMenu::ConnectionInfo,
                        "Connection Info",
                    );
                }
                if !netman.ban_list.lock().unwrap().is_empty() {
                    ui.selectable_value(
                        &mut self.connected_menu,
                        ConnectedMenu::BanList,
                        "Ban List",
                    );
                }
                if !netman.active_mods.lock().unwrap().is_empty() {
                    ui.selectable_value(&mut self.connected_menu, ConnectedMenu::Mods, "Mod List");
                }
                if last == ConnectedMenu::Settings && last != self.connected_menu {
                    let new_settings = self.app_saved_state.game_settings.clone();
                    *netman.pending_settings.lock().unwrap() = new_settings.clone();
                    update_lobby_data = true;
                    let mut old_settings = netman.settings.lock().unwrap().clone();
                    old_settings.progress.clear();
                    old_settings.seed = new_settings.seed;
                    old_settings.world_num = new_settings.world_num;
                    netman
                        .dirty
                        .store(old_settings != new_settings, Ordering::Relaxed)
                }
                ui.add_space(ui.available_width() - 56.0);
                if ui.button("Back out").clicked() {
                    goto_menu = true
                }
            });
            ui.separator();
            if stopped {
                ui.colored_label(Color32::LIGHT_RED, "Netmanager thread has stopped");
                if let Some(err) = netman.error.lock().unwrap().as_ref() {
                    ui.label("With the following error:");
                    ui.label(err.to_string());
                }
                ui.separator();
            }
            if let Some(game) = self.modmanager_settings.game_exe_path.parent() {
                if let Ok(s) = fs::read_to_string(game.join("logger.txt")) {
                    let l = self.noitalog.len();
                    if l != 0 && s.len() >= self.noitalog[l - 1].len() {
                        if s.len() != self.noitalog[l - 1].len() {
                            self.noitalog[l - 1] = s
                        }
                    } else {
                        self.noitalog_number = self.noitalog.len();
                        self.noitalog.push(s);
                    }
                }
            }
            match self.connected_menu {
                ConnectedMenu::Normal => {
                    if netman.peer.is_steam() {
                        if let Some(id) = netman.peer.lobby_id() {
                            if ui.button(tr("netman_save_lobby")).clicked() || !self.copied_lobby {
                                let lobby_code = LobbyCode {
                                    kind: self.my_lobby_kind,
                                    code: id,
                                };
                                if let Some(clipboard) = self.clipboard.as_mut() {
                                    let _ = clipboard.set_text(lobby_code.serialize());
                                }
                                self.copied_lobby = true;
                            }
                        } else {
                            ui.label("No lobby created yet");
                        }
                    }
                    self.appearance.mina_color_picker(
                        ui,
                        self.modmanager_settings.game_save_path.clone(),
                        self.player_image.clone(),
                    );
                    ui.add_space(15.0);
                    ui.horizontal(|ui| {
                        if ui.button("save colors").clicked() {
                            let desc = self
                                .appearance
                                .create_png_desc(self.modmanager_settings.game_save_path.clone());
                            netman.new_desc(desc, self.player_image.clone());
                            *netman.new_desc.lock().unwrap() = Some(desc);
                        };
                        ui.label("requires noita restart")
                    });
                    ui.add_space(15.0);
                    if accept_local && !local_connected {
                        match noita_launcher.launch_token() {
                            LaunchTokenResult::Ok(mut token) => {
                                let start_auto = self.can_start_automatically
                                    && self.app_saved_state.start_game_automatically;
                                if start_auto || ui.button(tr("launcher_start_game")).clicked() {
                                    info!("Starting the game now");
                                    token.start_game(
                                        netman.actual_noita_port.load(Ordering::Relaxed),
                                    );
                                    self.can_start_automatically = false;
                                }
                            }
                            LaunchTokenResult::AlreadyStarted => {
                                ui.label(tr("launcher_already_started"));
                            }
                            LaunchTokenResult::CantStart => {
                                ui.label(tr("launcher_no_command"));
                                ui.label(tr("launcher_no_command_2"));
                                ui.label(tr("launcher_no_command_3"));
                            }
                        }
                    } else {
                        ui.label(tr("launcher_only_when_awaiting"));
                    }
                    if netman.peer.is_host() {
                        ui.add_space(15.0);
                        self.end_run_button.show(ui, netman);
                        ui.add_space(15.0);
                        {
                            let mut temp = netman.no_more_players.load(Ordering::Relaxed);
                            if ui
                                .checkbox(&mut temp, "don't let more players join")
                                .changed()
                            {
                                netman.no_more_players.store(temp, Ordering::Relaxed);
                            }
                        }
                        {
                            let mut temp = netman.no_chunkmap_to_players.load(Ordering::Relaxed);
                            if ui
                                .checkbox(&mut temp, "don't send chunk map to players")
                                .changed()
                            {
                                netman.no_chunkmap_to_players.store(temp, Ordering::Relaxed);
                            }
                        }
                        {
                            let mut temp = netman.no_chunkmap.load(Ordering::Relaxed);
                            if ui.checkbox(&mut temp, "don't save chunk map, chunkmap is disabled by default do to current implementation ram/vram leaking on long runs").changed() {
                                netman.no_chunkmap.store(temp, Ordering::Relaxed);
                            }
                        }
                    }
                    {
                        let mut temp = netman.log_performance.load(Ordering::Relaxed);
                        if ui.checkbox(&mut temp, "log performance metrics, requires noita to be restarted").changed() {
                            netman.log_performance.store(temp, Ordering::Relaxed);
                        }
                    }
                }
                ConnectedMenu::Mods => {
                    let mods_list = netman.active_mods.lock().unwrap();
                    for mods in mods_list.iter() {
                        ui.label(mods);
                    }
                    if mods_list.is_empty() {
                        self.connected_menu = ConnectedMenu::Normal
                    }
                }
                ConnectedMenu::Map => self.map.ui(ui, netman, ctx),
                ConnectedMenu::BanList => {
                    let mut ban_list = netman.ban_list.lock().unwrap();
                    let mut i = ban_list.len();
                    while i != 0 {
                        i -= 1;
                        if ui.button(format!("unban {}", ban_list[i])).clicked() {
                            ban_list.remove(i);
                        }
                    }
                    if ban_list.is_empty() {
                        self.connected_menu = ConnectedMenu::Normal
                    }
                }
                ConnectedMenu::Settings => {
                    ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                        if netman.peer.is_host() {
                            self.app_saved_state.game_settings.show_editor(ui, true);
                        } else {
                            netman.settings.lock().unwrap().show_editor(ui, false);
                        }
                    });
                }
                ConnectedMenu::ConnectionInfo => match &netman.peer {
                    PeerVariant::Tangled(_) => {
                        ui.label("No connection info available in tangled mode");
                    }
                    PeerVariant::Steam(peer) => {
                        let steam = self.steam_state.as_ref().unwrap();
                        let report = peer.generate_report();
                        egui::Grid::new("Conn status grid")
                            .striped(true)
                            .show(ui, |ui| {
                                add_per_status_ui(&report, steam, ui);
                            });
                        ctx.request_repaint_after(Duration::from_millis(16));
                    }
                },
                ConnectedMenu::VoIP => {
                    let mut save = self.audio.show_ui(ui, false);
                    for peer in netman.peer.iter_peer_ids() {
                        if netman.peer.my_id() != peer {
                            ui.label(format!(
                                "volume for {}",
                                netman
                                    .nicknames
                                    .lock()
                                    .unwrap()
                                    .get(&peer)
                                    .unwrap_or(&peer.to_string())
                            ));
                            if ui
                                .add(Slider::new(
                                    self.audio.volume.entry(peer).or_insert(1.0),
                                    0.0..=8.0,
                                ))
                                .changed()
                            {
                                save = true;
                            }
                        }
                    }
                    if save {
                        *netman.audio.lock().unwrap() = self.audio.clone()
                    }
                }
                ConnectedMenu::NoitaLog => {
                    if !self.noitalog.is_empty() {
                        let mut s = self.noitalog[self.noitalog_number].clone() + "\n";
                        ui.horizontal(|ui| {
                            let l = self.noitalog.len();
                            if l > 1 {
                                ui.add(Slider::new(&mut self.noitalog_number, 0..=l - 1));
                            }
                            if let Some(clipboard) = self.clipboard.as_mut() {
                                if ui.button("save to clipboard").clicked() {
                                    let _ = clipboard.set_text(&s);
                                }
                            }
                        });
                        ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut s)
                                        .font(egui::TextStyle::Monospace)
                                        .desired_width(f32::INFINITY)
                                        .lock_focus(true),
                                );
                            });
                    }
                }
                ConnectedMenu::ProxyLog => {
                    if let Ok(s) = fs::read_to_string(if let Ok(path) = std::env::current_exe() {
                        path.parent().unwrap().join("ew_log.txt")
                    } else {
                        "ew_log.txt".into()
                    }) {
                        if s.len() > self.proxylog.len() {
                            self.proxylog = s
                        }
                    }
                    let mut s = self.proxylog.clone() + "\n";
                    ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut s)
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(f32::INFINITY)
                                    .lock_focus(true),
                            );
                        });
                }
            }
            if self.app_saved_state.show_extra_debug_stuff {
                ui.checkbox(
                    &mut self.app_saved_state.record_all,
                    tr("Record-everything-sent-to-noita"),
                );
            }
        });
        goto_menu |= netman.back_out.load(Ordering::Relaxed);
        netman
            .enable_recorder
            .store(self.app_saved_state.record_all, Ordering::Relaxed);
        if update_lobby_data {
            let data = self.make_lobby_extra_data();
            if let AppState::ConnectedLobby { netman, .. } = &mut self.state {
                netman.update_lobby_data(data);
            }
        }
        if goto_menu {
            self.state = AppState::Connect;
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
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(500));
        match &mut self.state {
            AppState::Connect => {
                self.connect_screen(ctx);
            }
            AppState::ConnectedLobby {
                netman,
                noita_launcher: _,
            } => {
                if let ExtraPeerState::CouldNotConnect(err) = netman.peer.state() {
                    self.notify_error(err);
                    return;
                }
                self.show_lobby(ctx);
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
                            &self.args,
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
                if let Some(lang) = &self.args.language {
                    let li: LanguageIdentifier = lang.parse().unwrap();
                    self.app_saved_state.lang_id = Some(li.clone());
                    set_current_locale(li);
                    self.state = AppState::ModManager;
                    return;
                }
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
            AppState::TangledConnecting { peer } => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label(tr("ip_wait_for_connection"));
                });
                if peer.state() == tangled::PeerState::Disconnected {
                    self.state = AppState::Error {
                        message: tr("ip_could_not_connect"),
                    };
                    return;
                }
                if peer.my_id().is_some() {
                    let AppState::TangledConnecting { peer } =
                        mem::replace(&mut self.state, AppState::Connect)
                    else {
                        unreachable!();
                    };
                    self.start_connect_step_2(peer);
                }
            }
            AppState::GogModeIssue(target_lobby) => {
                let mut button_back = false;
                let mut button_restart = false;
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label(format!(
                        "Mismathing modes: Host is in {:?} mode, you're in {:?} mode",
                        target_lobby.kind, self.my_lobby_kind
                    ));
                    button_restart = ui
                        .add(Button::new(tr("Switch-mode-and-restart")).fill(Color32::RED))
                        .clicked();
                    button_back = ui.button(tr("button_back")).clicked();
                });
                if button_restart {
                    let Err(err) = SelfRestarter::new().and_then(|mut r| {
                        r.override_lobby_kind(target_lobby.kind)
                            .connect_to(*target_lobby)
                            .restart()
                    });
                    self.notify_error(format!("Failed to self-restart: {}", err));
                }
                if button_back {
                    self.state = AppState::Connect;
                }
            }
        };
    }
    fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
        self.set_settings()
    }
}

fn show_player_list(ui: &mut Ui, netman: &mut NetManStopOnDrop) {
    ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        let nicknames = netman.nicknames.lock().unwrap().clone();
        let minas = netman.minas.lock().unwrap().clone();
        for peer in netman.peer.iter_peer_ids().clone() {
            let role = peer_role(peer, netman);
            let peer_str = peer.to_string().clone();
            let username = nicknames.get(&peer).unwrap_or(&peer_str);
            let mina = minas.get(&peer);
            ui.group(|ui| {
                if let Some(img) = mina {
                    display_with_labels(img.clone(), ui, username, &role, netman, peer)
                } else {
                    ui.label(username);
                    if peer != netman.peer.my_id() {
                        ui.horizontal(|ui| {
                            if mina.is_some() {
                                ui.add_space(5.0);
                            }
                            if netman.peer.is_host() {
                                if ui.button("Kick").clicked() {
                                    netman.kick_list.lock().unwrap().push(peer)
                                }
                                if ui.button("Ban").clicked() {
                                    netman.ban_list.lock().unwrap().push(peer)
                                }
                            }
                            if ui.button("Mods").clicked() {
                                netman.send(peer, &NetMsg::RequestMods, Reliability::Reliable);
                            }
                        });
                    }
                }
            });
        }
    });
}
fn display_with_labels(
    img: RgbaImage,
    ui: &mut Ui,
    label_top: &str,
    label_bottom: &str,
    netman: &mut NetManStopOnDrop,
    peer: OmniPeerId,
) {
    ui.scope(|ui| {
        ui.set_min_width(200.0);
        ui.horizontal(|ui| {
            display_player_skin(ui, img, 4.0);
            ui.vertical(|ui| {
                if netman.peer.my_id() != peer {
                    ui.horizontal(|ui| {
                        if netman.peer.is_host() {
                            if ui.button("Kick").clicked() {
                                netman.kick_list.lock().unwrap().push(peer)
                            }
                            if ui.button("Ban").clicked() {
                                netman.ban_list.lock().unwrap().push(peer)
                            }
                        }
                        if ui.button("Mods").clicked() {
                            netman.send(peer, &NetMsg::RequestMods, Reliability::Reliable);
                        }
                    });
                }
                ui.label(RichText::new(label_top).size(14.0));
                ui.label(RichText::new(label_bottom).size(11.0));
            });
        })
    });
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

fn peer_role(peer: OmniPeerId, netman: &Arc<net::NetManager>) -> String {
    if peer == netman.peer.host_id() {
        tr("player_host")
    } else if peer == netman.peer.my_id() {
        tr("player_me")
    } else {
        tr("player_player")
    }
}

fn cli_setup(
    args: Args,
) -> (
    Option<steam_helper::SteamState>,
    NetManagerInit,
    LobbyKind,
    AudioSettings,
    steamworks::LobbyType,
) {
    let settings = SavePaths::new().load_settings();
    let saved_state: AppSavedState = settings.app;
    let mut mod_manager: ModmanagerSettings = settings.modmanager;
    let appearance: PlayerAppearance = settings.color;
    let audio: AudioSettings = settings.audio;
    let mut state = steam_helper::SteamState::new(saved_state.spacewars).ok();
    let my_nickname = saved_state
        .nickname
        .unwrap_or("no nickname found".to_string());

    if let Some(state) = &mut state {
        mod_manager.try_find_game_path(Some(state));
    } else if let Some(p) = args.exe_path {
        mod_manager.game_exe_path = p
    } else {
        println!("needs game exe path if you want to join as host")
    }
    mod_manager.try_find_save_path();
    let run_save_state = if let Ok(path) = std::env::current_exe() {
        SaveState::new(path.parent().unwrap().join("save_state"))
    } else {
        SaveState::new("./save_state/")
    };
    let player_path = player_path(mod_manager.mod_path());
    let mut cosmetics = (false, false, false);
    if let Some(path) = &mod_manager.game_save_path {
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
    let netmaninit = NetManagerInit {
        my_nickname,
        save_state: run_save_state,
        cosmetics,
        mod_path: mod_manager.mod_path(),
        player_path,
        modmanager_settings: mod_manager,
        player_png_desc: PlayerPngDesc {
            cosmetics: cosmetics.into(),
            colors: appearance.player_color,
            invert_border: appearance.invert_border,
        },
        noita_port: 21251,
    };
    (
        state,
        netmaninit,
        if saved_state.spacewars {
            LobbyKind::Gog
        } else {
            LobbyKind::Steam
        },
        audio,
        if saved_state.public_lobby {
            steamworks::LobbyType::Public
        } else if saved_state.allow_friends {
            steamworks::LobbyType::FriendsOnly
        } else {
            steamworks::LobbyType::Private
        },
    )
}

pub fn connect_cli(lobby: String, args: Args) {
    let (state, netmaninit, kind, audio, _) = cli_setup(args);
    let variant = if lobby.contains(':') {
        let p = Peer::connect(lobby.parse().unwrap(), None).unwrap();
        while p.my_id().is_none() {
            sleep(Duration::from_millis(100))
        }
        PeerVariant::Tangled(p)
    } else if let Some(state) = state {
        let peer = net::steam_networking::SteamPeer::new_connect(
            LobbyCode::parse(lobby.trim()).unwrap().code,
            state.client,
        );
        PeerVariant::Steam(peer)
    } else {
        println!("no steam");
        exit(1)
    };
    let player_path = netmaninit.player_path.clone();
    let netman = net::NetManager::new(variant, netmaninit, audio);
    netman.start_inner(player_path, Some(kind)).unwrap();
}

/// Bind to the provided `bind_addr` with `args` with CLI output only.
///
/// The `bind_addr` is either `Some` address/port pair to bind to, or `None` to use Steam networking.
pub fn host_cli(bind_addr: Option<SocketAddr>, args: Args) {
    let (state, netmaninit, kind, audio, lobbytype) = cli_setup(args);
    let variant = if let Some(bind_addr) = bind_addr {
        let peer = Peer::host(bind_addr, None).unwrap();
        PeerVariant::Tangled(peer)
    } else if let Some(state) = state {
        let peer = net::steam_networking::SteamPeer::new_host(
            lobbytype,
            state.client,
            250,
            LobbyExtraData {
                name: "no name specified".to_string(),
                game_mode: None,
            },
        );
        PeerVariant::Steam(peer)
    } else {
        println!("no steam");
        exit(1)
    };
    let player_path = netmaninit.player_path.clone();
    let netman = net::NetManager::new(variant, netmaninit, audio);
    netman.start_inner(player_path, Some(kind)).unwrap();
}
