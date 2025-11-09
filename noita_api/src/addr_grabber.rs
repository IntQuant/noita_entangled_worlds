use std::{os::raw::c_void, ptr};

use crate::lua::LuaState;
use crate::noita::types::{
    ComponentSystemManager, ComponentTypeManager, Entity, EntityManager, EventManager, GameGlobal,
    GlobalStats, Inventory, Mods, Platform, StdString, StdVec, TagManager, TranslationManager,
    WorldStateComponent,
};
use iced_x86::{Decoder, DecoderOptions, Mnemonic};

pub(crate) unsafe fn grab_addr_from_instruction(
    func: *const c_void,
    offset: isize,
    expected_mnemonic: Mnemonic,
) -> *mut c_void {
    let instruction_addr = func.wrapping_offset(offset);
    // We don't really have an idea of how many bytes the instruction takes, so just take *enough* bytes for most cases.
    let instruction_bytes = unsafe { ptr::read_unaligned(instruction_addr.cast::<[u8; 16]>()) };
    let mut decoder = Decoder::with_ip(
        32,
        &instruction_bytes,
        instruction_addr as u64,
        DecoderOptions::NONE,
    );
    let instruction = decoder.decode();

    #[cfg(debug_assertions)]
    if instruction.mnemonic() != expected_mnemonic {
        println!("Encountered unexpected mnemonic: {instruction}");
    }
    assert_eq!(instruction.mnemonic(), expected_mnemonic);

    instruction.memory_displacement32() as *mut c_void
}

// This only stores pointers that are constant, so should be safe to share between threads.
unsafe impl Sync for Globals {}
unsafe impl Send for Globals {}

#[derive(Debug)]
pub struct GlobalsRef {
    pub world_seed: usize,
    pub new_game_count: usize,
    pub game_global: &'static GameGlobal,
    pub entity_manager: &'static EntityManager,
    pub entity_tag_manager: &'static TagManager<u16>,
    pub component_type_manager: &'static ComponentTypeManager,
    pub component_tag_manager: &'static TagManager<u8>,
    pub translation_manager: &'static TranslationManager,
    pub platform: &'static Platform,
    pub global_stats: &'static GlobalStats,
    pub filenames: &'static StdVec<StdString>,
    pub inventory: &'static Inventory,
    pub mods: &'static Mods,
    pub max_component: &'static usize,
    pub component_manager: &'static ComponentSystemManager,
    pub world_state: &'static Entity,
    pub world_state_component: &'static WorldStateComponent,
    pub event_manager: &'static EventManager,
}
#[derive(Debug)]
pub struct GlobalsMut {
    pub world_seed: &'static mut usize,
    pub new_game_count: &'static mut usize,
    pub game_global: &'static mut GameGlobal,
    pub entity_manager: &'static mut EntityManager,
    pub entity_tag_manager: &'static mut TagManager<u16>,
    pub component_type_manager: &'static mut ComponentTypeManager,
    pub component_tag_manager: &'static mut TagManager<u8>,
    pub translation_manager: &'static mut TranslationManager,
    pub platform: &'static mut Platform,
    pub global_stats: &'static mut GlobalStats,
    pub filenames: &'static mut StdVec<StdString>,
    pub inventory: &'static mut Inventory,
    pub mods: &'static mut Mods,
    pub max_component: &'static mut usize,
    pub component_manager: &'static mut ComponentSystemManager,
    pub world_state: &'static mut Entity,
    pub world_state_component: &'static mut WorldStateComponent,
    pub event_manager: &'static mut EventManager,
}

#[derive(Debug, Default)]
pub struct Globals {
    pub world_seed: *mut usize,
    pub new_game_count: *mut usize,
    pub game_global: *const *mut GameGlobal,
    pub entity_manager: *const *mut EntityManager,
    pub entity_tag_manager: *const *mut TagManager<u16>,
    pub component_type_manager: *mut ComponentTypeManager,
    pub component_tag_manager: *const *mut TagManager<u8>,
    pub translation_manager: *mut TranslationManager,
    pub platform: *mut Platform,
    pub global_stats: *mut GlobalStats,
    pub filenames: *mut StdVec<StdString>,
    pub inventory: *mut Inventory,
    pub mods: *mut Mods,
    pub max_component: *mut usize,
    pub component_manager: *mut ComponentSystemManager,
    pub world_state: *const *mut Entity,
    pub world_state_component: *const *mut WorldStateComponent,
    pub event_manager: *const *mut EventManager,
}
#[allow(clippy::mut_from_ref)]
impl Globals {
    pub fn world_seed(&self) -> usize {
        unsafe { self.world_seed.as_ref().copied().unwrap() }
    }
    pub fn new_game_count(&self) -> usize {
        unsafe { self.new_game_count.as_ref().copied().unwrap() }
    }
    pub fn game_global(&self) -> &'static GameGlobal {
        unsafe { self.game_global.as_ref().unwrap().as_ref().unwrap() }
    }
    pub fn entity_manager(&self) -> &'static EntityManager {
        unsafe { self.entity_manager.as_ref().unwrap().as_ref().unwrap() }
    }
    pub fn entity_tag_manager(&self) -> &'static TagManager<u16> {
        unsafe { self.entity_tag_manager.as_ref().unwrap().as_ref().unwrap() }
    }
    pub fn component_type_manager(&self) -> &'static ComponentTypeManager {
        unsafe { self.component_type_manager.as_ref().unwrap() }
    }
    pub fn component_tag_manager(&self) -> &'static TagManager<u8> {
        unsafe {
            self.component_tag_manager
                .as_ref()
                .unwrap()
                .as_ref()
                .unwrap()
        }
    }
    pub fn component_manager(&self) -> &'static ComponentSystemManager {
        unsafe { self.component_manager.as_ref().unwrap() }
    }
    pub fn translation_manager(&self) -> &'static TranslationManager {
        unsafe { self.translation_manager.as_ref().unwrap() }
    }
    pub fn platform(&self) -> &'static Platform {
        unsafe { self.platform.as_ref().unwrap() }
    }
    pub fn global_stats(&self) -> &'static GlobalStats {
        unsafe { self.global_stats.as_ref().unwrap() }
    }
    pub fn filenames(&self) -> &'static StdVec<StdString> {
        unsafe { self.filenames.as_ref().unwrap() }
    }
    pub fn inventory(&self) -> &'static Inventory {
        unsafe { self.inventory.as_ref().unwrap() }
    }
    pub fn mods(&self) -> &'static Mods {
        unsafe { self.mods.as_ref().unwrap() }
    }
    pub fn max_component(&self) -> &'static usize {
        unsafe { self.max_component.as_ref().unwrap() }
    }
    pub fn world_seed_mut(&self) -> &'static mut usize {
        unsafe { self.world_seed.as_mut().unwrap() }
    }
    pub fn new_game_count_mut(&self) -> &'static mut usize {
        unsafe { self.new_game_count.as_mut().unwrap() }
    }
    pub fn game_global_mut(&self) -> &'static mut GameGlobal {
        unsafe { self.game_global.as_ref().unwrap().as_mut().unwrap() }
    }
    pub fn entity_manager_mut(&self) -> &'static mut EntityManager {
        unsafe { self.entity_manager.as_ref().unwrap().as_mut().unwrap() }
    }
    pub fn entity_tag_manager_mut(&self) -> &'static mut TagManager<u16> {
        unsafe { self.entity_tag_manager.as_ref().unwrap().as_mut().unwrap() }
    }
    pub fn world_state_mut(&self) -> &'static mut Entity {
        unsafe { self.world_state.as_ref().unwrap().as_mut().unwrap() }
    }
    pub fn world_state(&self) -> &'static Entity {
        unsafe { self.world_state.as_ref().unwrap().as_ref().unwrap() }
    }
    pub fn event_manager_mut(&self) -> &'static mut EventManager {
        unsafe { self.event_manager.as_ref().unwrap().as_mut().unwrap() }
    }
    pub fn event_manager(&self) -> &'static EventManager {
        unsafe { self.event_manager.as_ref().unwrap().as_ref().unwrap() }
    }
    pub fn world_state_component_mut(&self) -> &'static mut WorldStateComponent {
        unsafe {
            self.world_state_component
                .as_ref()
                .unwrap()
                .as_mut()
                .unwrap()
        }
    }
    pub fn world_state_component(&self) -> &'static WorldStateComponent {
        unsafe {
            self.world_state_component
                .as_ref()
                .unwrap()
                .as_ref()
                .unwrap()
        }
    }
    pub fn component_type_manager_mut(&self) -> &'static mut ComponentTypeManager {
        unsafe { self.component_type_manager.as_mut().unwrap() }
    }
    pub fn component_tag_manager_mut(&self) -> &'static mut TagManager<u8> {
        unsafe {
            self.component_tag_manager
                .as_ref()
                .unwrap()
                .as_mut()
                .unwrap()
        }
    }
    pub fn translation_manager_mut(&self) -> &'static mut TranslationManager {
        unsafe { self.translation_manager.as_mut().unwrap() }
    }
    pub fn platform_mut(&self) -> &'static mut Platform {
        unsafe { self.platform.as_mut().unwrap() }
    }
    pub fn global_stats_mut(&self) -> &'static mut GlobalStats {
        unsafe { self.global_stats.as_mut().unwrap() }
    }
    pub fn filenames_mut(&self) -> &'static mut StdVec<StdString> {
        unsafe { self.filenames.as_mut().unwrap() }
    }
    pub fn inventory_mut(&self) -> &'static mut Inventory {
        unsafe { self.inventory.as_mut().unwrap() }
    }
    pub fn mods_mut(&self) -> &'static mut Mods {
        unsafe { self.mods.as_mut().unwrap() }
    }
    pub fn max_component_mut(&self) -> &'static mut usize {
        unsafe { self.max_component.as_mut().unwrap() }
    }
    pub fn component_manager_mut(&self) -> &'static mut ComponentSystemManager {
        unsafe { self.component_manager.as_mut().unwrap() }
    }
    pub fn as_ref(&self) -> GlobalsRef {
        GlobalsRef {
            world_seed: self.world_seed(),
            new_game_count: self.new_game_count(),
            game_global: self.game_global(),
            entity_manager: self.entity_manager(),
            entity_tag_manager: self.entity_tag_manager(),
            component_type_manager: self.component_type_manager(),
            component_tag_manager: self.component_tag_manager(),
            translation_manager: self.translation_manager(),
            platform: self.platform(),
            global_stats: self.global_stats(),
            filenames: self.filenames(),
            inventory: self.inventory(),
            mods: self.mods(),
            max_component: self.max_component(),
            component_manager: self.component_manager(),
            world_state: self.world_state(),
            world_state_component: self.world_state_component(),
            event_manager: self.event_manager(),
        }
    }
    pub fn as_mut(&self) -> GlobalsMut {
        GlobalsMut {
            world_seed: self.world_seed_mut(),
            new_game_count: self.new_game_count_mut(),
            game_global: self.game_global_mut(),
            entity_manager: self.entity_manager_mut(),
            entity_tag_manager: self.entity_tag_manager_mut(),
            component_type_manager: self.component_type_manager_mut(),
            component_tag_manager: self.component_tag_manager_mut(),
            translation_manager: self.translation_manager_mut(),
            platform: self.platform_mut(),
            global_stats: self.global_stats_mut(),
            filenames: self.filenames_mut(),
            inventory: self.inventory_mut(),
            mods: self.mods_mut(),
            max_component: self.max_component_mut(),
            component_manager: self.component_manager_mut(),
            world_state: self.world_state_mut(),
            world_state_component: self.world_state_component_mut(),
            event_manager: self.event_manager_mut(),
        }
    }
    pub fn new(lua: LuaState) -> Self {
        lua.get_global(c"EntityGetFilename");
        let base = lua.to_cfunction(-1).unwrap() as *const c_void;
        let entity_manager: *const *mut EntityManager = unsafe {
            grab_addr_from_instruction(base, 0x00797821 - 0x00797570, Mnemonic::Mov).cast()
        };
        lua.pop_last();
        let world_seed = 0x1205004 as *mut usize;
        let new_game_count = 0x1205024 as *mut usize;
        let global_stats = 0x1208940 as *mut GlobalStats;
        let game_global = 0x122374c as *const *mut GameGlobal;
        let entity_tag_manager = 0x1206fac as *const *mut TagManager<u16>;
        let component_type_manager = 0x1223c88 as *mut ComponentTypeManager;
        let component_tag_manager = 0x1204b30 as *const *mut TagManager<u8>;
        let translation_manager = 0x1207c28 as *mut TranslationManager;
        let platform = 0x1221bc0 as *mut Platform;
        let filenames = 0x1207bd4 as *mut StdVec<StdString>;
        let inventory = 0x12224f0 as *mut Inventory;
        let mods = 0x1207e90 as *mut Mods;
        let max_component = 0x1152ff0 as *mut usize;
        let component_manager = 0x12236e8 as *mut ComponentSystemManager;
        let world_state = 0x1204bd0 as *const *mut Entity;
        let world_state_component = 0x1205010 as *const *mut WorldStateComponent;
        let event_manager = 0x1204b34 as *const *mut EventManager;
        Self {
            world_seed,
            new_game_count,
            game_global,
            entity_manager,
            entity_tag_manager,
            component_type_manager,
            component_tag_manager,
            translation_manager,
            platform,
            global_stats,
            filenames,
            inventory,
            mods,
            max_component,
            component_manager,
            world_state,
            world_state_component,
            event_manager,
        }
    }
}
