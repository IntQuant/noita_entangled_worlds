use std::{os::raw::c_void, ptr};

use crate::lua::LuaState;
use crate::noita::types::{
    ComponentTypeManager, EntityManager, GameGlobal, GlobalStats, Platform, StdString, StdVec,
    TagManager, TranslationManager,
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
}
#[allow(clippy::mut_from_ref)]
impl Globals {
    pub fn world_seed(&self) -> Option<usize> {
        unsafe { self.world_seed.as_ref().copied() }
    }
    pub fn new_game_count(&self) -> Option<usize> {
        unsafe { self.new_game_count.as_ref().copied() }
    }
    pub fn game_global(&self) -> Option<&'static GameGlobal> {
        unsafe { self.game_global.as_ref()?.as_ref() }
    }
    pub fn entity_manager(&self) -> Option<&'static EntityManager> {
        unsafe { self.entity_manager.as_ref()?.as_ref() }
    }
    pub fn entity_tag_manager(&self) -> Option<&'static TagManager<u16>> {
        unsafe { self.entity_tag_manager.as_ref()?.as_ref() }
    }
    pub fn component_type_manager(&self) -> Option<&'static ComponentTypeManager> {
        unsafe { self.component_type_manager.as_ref() }
    }
    pub fn component_tag_manager(&self) -> Option<&'static TagManager<u8>> {
        unsafe { self.component_tag_manager.as_ref()?.as_ref() }
    }
    pub fn translation_manager(&self) -> Option<&'static TranslationManager> {
        unsafe { self.translation_manager.as_ref() }
    }
    pub fn platform(&self) -> Option<&'static Platform> {
        unsafe { self.platform.as_ref() }
    }
    pub fn global_stats(&self) -> Option<&'static GlobalStats> {
        unsafe { self.global_stats.as_ref() }
    }
    pub fn filenames(&self) -> Option<&'static StdVec<StdString>> {
        unsafe { self.filenames.as_ref() }
    }
    pub fn world_seed_mut(&self) -> Option<&'static mut usize> {
        unsafe { self.world_seed.as_mut() }
    }
    pub fn new_game_count_mut(&self) -> Option<&'static mut usize> {
        unsafe { self.new_game_count.as_mut() }
    }
    pub fn game_global_mut(&self) -> Option<&'static mut GameGlobal> {
        unsafe { self.game_global.as_ref()?.as_mut() }
    }
    pub fn entity_manager_mut(&self) -> Option<&'static mut EntityManager> {
        unsafe { self.entity_manager.as_ref()?.as_mut() }
    }
    pub fn entity_tag_manager_mut(&self) -> Option<&'static mut TagManager<u16>> {
        unsafe { self.entity_tag_manager.as_ref()?.as_mut() }
    }
    pub fn component_type_manager_mut(&self) -> Option<&'static mut ComponentTypeManager> {
        unsafe { self.component_type_manager.as_mut() }
    }
    pub fn component_tag_manager_mut(&self) -> Option<&'static mut TagManager<u8>> {
        unsafe { self.component_tag_manager.as_ref()?.as_mut() }
    }
    pub fn translation_manager_mut(&self) -> Option<&'static mut TranslationManager> {
        unsafe { self.translation_manager.as_mut() }
    }
    pub fn platform_mut(&self) -> Option<&'static mut Platform> {
        unsafe { self.platform.as_mut() }
    }
    pub fn global_stats_mut(&self) -> Option<&'static mut GlobalStats> {
        unsafe { self.global_stats.as_mut() }
    }
    pub fn filenames_mut(&self) -> Option<&'static mut StdVec<StdString>> {
        unsafe { self.filenames.as_mut() }
    }
    pub fn as_ref(&self) -> Option<GlobalsRef> {
        Some(GlobalsRef {
            world_seed: self.world_seed()?,
            new_game_count: self.new_game_count()?,
            game_global: self.game_global()?,
            entity_manager: self.entity_manager()?,
            entity_tag_manager: self.entity_tag_manager()?,
            component_type_manager: self.component_type_manager()?,
            component_tag_manager: self.component_tag_manager()?,
            translation_manager: self.translation_manager()?,
            platform: self.platform()?,
            global_stats: self.global_stats()?,
            filenames: self.filenames()?,
        })
    }
    pub fn as_mut(&self) -> Option<GlobalsMut> {
        Some(GlobalsMut {
            world_seed: self.world_seed_mut()?,
            new_game_count: self.new_game_count_mut()?,
            game_global: self.game_global_mut()?,
            entity_manager: self.entity_manager_mut()?,
            entity_tag_manager: self.entity_tag_manager_mut()?,
            component_type_manager: self.component_type_manager_mut()?,
            component_tag_manager: self.component_tag_manager_mut()?,
            translation_manager: self.translation_manager_mut()?,
            platform: self.platform_mut()?,
            global_stats: self.global_stats_mut()?,
            filenames: self.filenames_mut()?,
        })
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
        }
    }
}
