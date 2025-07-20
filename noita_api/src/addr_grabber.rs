use std::{os::raw::c_void, ptr, sync::OnceLock};

use crate::lua::LuaState;
use crate::noita::types::{
    ComponentTypeManager, EntityManager, GameGlobal, GlobalStats, Platform, TagManager,
    TranslationManager,
};
use iced_x86::{Decoder, DecoderOptions, Mnemonic};
static GRABBED: OnceLock<Grabbed> = OnceLock::new();

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
unsafe impl Sync for Grabbed {}
unsafe impl Send for Grabbed {}

pub struct Grabbed {
    // These 3 actually point to a pointer.
    pub world_seed: *const *mut usize,
    pub new_game_count: *const *mut usize,
    pub game_global: *const *mut GameGlobal,
    pub entity_manager: *const *mut EntityManager,
    pub entity_tag_manager: *const *mut TagManager,
    pub component_type_manager: *const *mut ComponentTypeManager,
    pub component_tag_manager: *const *mut TagManager,
    pub translation_manager: *const *mut TranslationManager,
    pub platform: *const *mut Platform,
    pub global_stats: *const *mut GlobalStats,
}

pub fn grab_addrs(lua: LuaState) {
    lua.get_global(c"EntityGetFilename");
    let base = lua.to_cfunction(-1).unwrap() as *const c_void;
    let entity_manager: *const *mut EntityManager =
        unsafe { grab_addr_from_instruction(base, 0x00797821 - 0x00797570, Mnemonic::Mov).cast() };
    lua.pop_last();

    let world_seed = 0x1205004 as *const *mut usize;
    let new_game_count = 0x1205024 as *const *mut usize;
    let global_stats = 0x1208940 as *const *mut GlobalStats;
    let game_global = 0x122374c as *const *mut GameGlobal;
    let entity_tag_manager = 0x1206fac as *const *mut TagManager;
    let component_type_manager = 0x1223c88 as *const *mut ComponentTypeManager;
    let component_tag_manager = 0x1204b30 as *const *mut TagManager;
    let translation_manager = 0x1207c28 as *const *mut TranslationManager;
    let platform = 0x1221bc0 as *const *mut Platform;

    GRABBED
        .set(Grabbed {
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
        })
        .ok();
}

pub fn grabbed_globals() -> &'static Grabbed {
    GRABBED.get().expect("to be initialized early")
}
