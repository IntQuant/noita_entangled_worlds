use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use eframe::egui::{Color32, DragValue, Slider, TextEdit, Ui};

use std::fmt::Display;
use std::str::FromStr;

use super::tr;

#[derive(Debug, Decode, Encode, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct GameSettings {
    pub seed: u64,
    pub world_num: u8,
    pub debug_mode: Option<bool>,
    pub use_constant_seed: bool,
    pub duplicate: Option<bool>,
    pub enemy_hp_mult: Option<f32>,
    pub game_mode: Option<GameMode>,
    pub friendly_fire: Option<bool>,
    pub randomize_perks: Option<bool>,
    pub progress: Vec<String>,
    pub max_players: Option<u32>,
    pub health_per_player: Option<u32>,
    pub health_lost_on_revive: Option<u32>,
    pub no_material_damage: Option<bool>,
    pub global_hp_loss: Option<bool>,
    pub perk_ban_list: Option<String>,
    pub disabled_globals: Option<String>,
    pub spell_ban_list: Option<String>,
    pub physics_damage: Option<bool>,
    pub share_gold: Option<bool>,
    pub nice_terraforming: Option<bool>,
    pub same_loadout: Option<bool>,
    pub disable_kummitus: Option<bool>,
    pub give_host_sampo: Option<bool>,
    pub home_on_players: Option<bool>,
    pub pvp_kill_steal: Option<u32>,
    pub dont_steal: Option<bool>,
    pub wait_on_players: Option<bool>,
    pub time_in_hm: Option<u32>,
    pub time_out_hm: Option<u32>,
    pub chest_on_win: Option<bool>,
    pub wait_for_time: Option<bool>,
    pub timed: Option<bool>,
    pub lha_dont_run: Option<bool>,
}

pub struct DefaultSettings {
    pub debug_mode: bool,
    //item_dedup: bool,
    pub enemy_hp_mult: f32,
    pub game_mode: GameMode,
    pub friendly_fire: bool,
    pub randomize_perks: bool,
    pub max_players: u32,
    pub health_per_player: u32,
    pub health_lost_on_revive: u32,
    pub no_material_damage: bool,
    pub global_hp_loss: bool,
    pub perk_ban_list: String,
    pub disabled_globals: String,
    pub spell_ban_list: String,
    pub physics_damage: bool,
    pub share_gold: bool,
    pub nice_terraforming: bool,
    pub same_loadout: bool,
    pub duplicate: bool,
    pub disable_kummitus: bool,
    pub give_host_sampo: bool,
    pub home_on_players: bool,
    pub pvp_kill_steal: u32,
    pub dont_steal: bool,
    pub wait_on_players: bool,
    pub time_in_hm: u32,
    pub time_out_hm: u32,
    pub chest_on_win: bool,
    pub wait_for_time: bool,
    pub timed: bool,
    pub lha_dont_run: bool,
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
            lha_dont_run: false,
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

#[derive(Debug, Decode, Encode, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum GameMode {
    SharedHealth,
    LocalHealth(LocalHealthMode),
    // MestariMina, // TODO later
}
impl GameMode {
    pub fn color(&self) -> Color32 {
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
        write!(f, "{desc}")
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

impl GameSettings {
    pub fn show_editor(&mut self, ui: &mut Ui, enabled: bool) {
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
                                        game_settings.global_hp_loss.unwrap_or(def.global_hp_loss);
                                    if ui.checkbox(&mut temp, tr("global_hp_loss")).changed() {
                                        game_settings.global_hp_loss = Some(temp)
                                    }
                                }
                                {
                                    let mut temp =
                                        game_settings.lha_dont_run.unwrap_or(def.lha_dont_run);
                                    if ui.checkbox(&mut temp, tr("lha_dont_run")).changed() {
                                        game_settings.lha_dont_run = Some(temp)
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
                        TextEdit::singleline(&mut temp),
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
                        TextEdit::singleline(&mut temp),
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
                        TextEdit::singleline(&mut temp),
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
