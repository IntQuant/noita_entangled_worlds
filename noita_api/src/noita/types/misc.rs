use crate::lua::LuaState;
use crate::noita::types::{StdMap, StdString, StdVec, Vec2};
#[derive(Debug)]
pub struct GlobalStatsVTable {}
#[derive(Debug)]
#[repr(C)]
pub struct GlobalStats {
    pub vftable: &'static GlobalStatsVTable,
    pub stats_version: usize,
    pub debug_tracker: usize,
    pub debug: bool,
    padding1: [u8; 3],
    pub debug_reset_counter: usize,
    pub fix_stats_flag: bool,
    pub session_dead: bool,
    padding2: [u8; 2],
    pub key_value_stats: StdMap<StdString, usize>,
    pub session: GameStats,
    pub highest: GameStats,
    pub global: GameStats,
    pub prev_best: GameStats,
}
#[derive(Debug)]
pub struct GameStatsVTable {}
#[derive(Debug)]
#[repr(C)]
pub struct GameStats {
    pub vftable: &'static GameStatsVTable,
    pub dead: bool,
    padding1: [u8; 3],
    pub death_count: usize,
    pub streaks: usize,
    pub world_seed: usize,
    pub killed_by: StdString,
    pub killed_by_extra: StdString,
    pub death_pos: Vec2,
    field_0x4c: usize,
    pub playtime: f64,
    pub playtime_str: StdString,
    pub places_visited: usize,
    pub enemies_killed: usize,
    pub heart_containers: usize,
    field_0x7c: usize,
    pub hp: i64,
    pub gold: i64,
    pub gold_all: i64,
    pub gold_infinite: bool,
    padding2: [u8; 3],
    pub items: usize,
    pub projectiles_shot: usize,
    pub kicks: usize,
    pub damage_taken: f64,
    pub healed: f64,
    pub teleports: usize,
    pub wands_edited: usize,
    pub biomes_visited_with_wands: usize,
    field_0xc4: usize,
}
#[derive(Debug)]
pub struct TranslationManagerVTable {}
#[derive(Debug)]
#[repr(C)]
pub struct TranslationManager {
    pub vftable: &'static TranslationManagerVTable,
    pub unknown_strings: StdVec<StdString>,
    pub languages: StdVec<Language>,
    pub key_to_index: StdMap<StdString, usize>,
    pub extra_lang_files: StdVec<StdString>,
    pub current_lang_idx: usize,
    pub unknown: usize,
    pub unknown_float: f32,
    pub unknown_primitive_vec: StdVec<usize>,
    pub unknown_map: StdMap<StdString, StdString>,
}

#[derive(Debug)]
#[repr(C)]
pub struct Language {
    pub id: StdString,
    pub name: StdString,
    pub font_default: StdString,
    pub font_inventory_title: StdString,
    pub font_important_message_title: StdString,
    pub font_world_space_message: StdString,
    pub fonts_utf8: bool,
    pub fonts_pixel_font: bool,
    padding1: [u8; 2],
    pub fonts_dpi: f32,
    pub ui_wand_info_offset1: f32,
    pub ui_wand_info_offset2: f32,
    pub ui_action_info_offset2: f32,
    pub ui_configurecontrols_offset2: f32,
    pub strings: StdVec<StdString>,
}
#[derive(Debug)]
#[repr(C)]
pub struct ModListEntry {
    pub name: StdString,
    pub steam_id: usize,
    unk1: [u8; 4],
    pub enabled: bool,
    unk1_bool: bool,
    unk2_bool: bool,
    unk2: u8,
    unk3: [u8; 4],
}
#[derive(Debug)]
#[repr(C)]
pub struct Mods {
    pub names: StdVec<ModListEntry>,
    pub list: StdVec<Mod>,
}
#[derive(Debug)]
#[repr(C)]
pub struct ModVTable {}
#[derive(Debug)]
#[repr(C)]
pub struct Mod {
    unk: [usize; 14],
    pub lua_data: &'static ModLua,
    pub vtable: &'static ModVTable,
    unk2: [usize; 8],
}
#[derive(Debug)]
#[repr(C)]
pub struct ModLua {
    unk: [usize; 14],
    pub lua_state: *const LuaState,
}
