use std::{os::raw::c_void, ptr, sync::OnceLock};

use crate::lua::LuaState;
use crate::noita::types::EntityManager;
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

struct Grabbed {
    globals: GrabbedGlobals,
}

// This only stores pointers that are constant, so should be safe to share between threads.
unsafe impl Sync for Grabbed {}
unsafe impl Send for Grabbed {}

pub struct GrabbedGlobals {
    // These 3 actually point to a pointer.
    pub entity_manager: *const *mut EntityManager,
}

pub fn grab_addrs(lua: LuaState) {
    lua.get_global(c"EntityGetFilename");
    let base = lua.to_cfunction(-1).unwrap() as *const c_void;
    let entity_manager: *const *mut EntityManager =
        unsafe { grab_addr_from_instruction(base, 0x00797821 - 0x00797570, Mnemonic::Mov).cast() };
    lua.pop_last();

    GRABBED
        .set(Grabbed {
            globals: GrabbedGlobals { entity_manager },
        })
        .ok();
}

pub fn grabbed_globals() -> &'static GrabbedGlobals {
    &GRABBED.get().expect("to be initialized early").globals
}
