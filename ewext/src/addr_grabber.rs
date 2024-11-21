use std::{mem, os::raw::c_void, ptr, sync::OnceLock};

use iced_x86::{Decoder, DecoderOptions, Mnemonic};

use crate::{
    lua_bindings::{lua_State, LUA_GLOBALSINDEX},
    noita::ntypes::{EntityManager, ThiscallFn},
    LUA,
};

static GRABBED: OnceLock<Grabbed> = OnceLock::new();

pub(crate) unsafe fn grab_addr_from_instruction(
    func: *const c_void,
    offset: isize,
    expected_mnemonic: Mnemonic,
) -> *mut c_void {
    let instruction_addr = func.wrapping_offset(offset);
    // We don't really have an idea of how many bytes the instruction takes, so just take *enough* bytes for most cases.
    let instruction_bytes = ptr::read_unaligned(instruction_addr.cast::<[u8; 16]>());
    let mut decoder = Decoder::with_ip(
        32,
        &instruction_bytes,
        instruction_addr as u64,
        DecoderOptions::NONE,
    );
    let instruction = decoder.decode();

    if instruction.mnemonic() != expected_mnemonic {
        println!("Encountered unexpected mnemonic: {}", instruction);
    }
    assert_eq!(instruction.mnemonic(), expected_mnemonic);

    instruction.memory_displacement32() as *mut c_void
}

struct Grabbed {
    globals: GrabbedGlobals,
    fns: GrabbedFns,
}

// This only stores pointers that are constant, so should be safe to share between threads.
unsafe impl Sync for Grabbed {}
unsafe impl Send for Grabbed {}

pub(crate) struct GrabbedGlobals {
    // These 3 actually point to a pointer.
    pub(crate) game_global: *mut usize,
    pub(crate) world_state_entity: *mut usize,
    pub(crate) entity_manager: *const *mut EntityManager,
}

pub(crate) struct GrabbedFns {
    pub(crate) get_entity: *const ThiscallFn, //unsafe extern "C" fn(*const EntityManager, u32) -> *mut Entity,
}

pub(crate) unsafe fn grab_addrs(lua: *mut lua_State) {
    LUA.lua_getfield(lua, LUA_GLOBALSINDEX, c"GameGetWorldStateEntity".as_ptr());
    let base = LUA.lua_tocfunction(lua, -1).unwrap() as *const c_void;
    let world_state_entity =
        grab_addr_from_instruction(base, 0x007aa7ce - 0x007aa540, Mnemonic::Mov).cast();
    println!(
        "World state entity addr: 0x{:x}",
        world_state_entity as usize
    );
    // Pop the last element.
    LUA.lua_settop(lua, -2);

    LUA.lua_getfield(lua, LUA_GLOBALSINDEX, c"GameGetFrameNum".as_ptr());
    let base = LUA.lua_tocfunction(lua, -1).unwrap() as *const c_void;
    let load_game_global =
        grab_addr_from_instruction(base, 0x007bf3c9 - 0x007bf140, Mnemonic::Call); // CALL load_game_global
    println!("Load game global addr: 0x{:x}", load_game_global as usize);
    let game_global =
        grab_addr_from_instruction(load_game_global, 0x00439c17 - 0x00439bb0, Mnemonic::Mov).cast();
    println!("Game global addr: 0x{:x}", game_global as usize);
    // Pop the last element.
    LUA.lua_settop(lua, -2);

    LUA.lua_getfield(lua, LUA_GLOBALSINDEX, c"EntityGetFilename".as_ptr());
    let base = LUA.lua_tocfunction(lua, -1).unwrap() as *const c_void;
    let get_entity = mem::transmute_copy(&grab_addr_from_instruction(
        base,
        0x0079782b - 0x00797570,
        Mnemonic::Call,
    ));
    println!("get_entity addr: 0x{:x}", get_entity as usize);
    let entity_manager =
        grab_addr_from_instruction(base, 0x00797821 - 0x00797570, Mnemonic::Mov).cast();
    println!("entity_manager addr: 0x{:x}", entity_manager as usize);
    // Pop the last element.
    LUA.lua_settop(lua, -2);

    GRABBED
        .set(Grabbed {
            globals: GrabbedGlobals {
                game_global,
                world_state_entity,
                entity_manager,
            },
            fns: GrabbedFns { get_entity },
        })
        .ok();
}

pub(crate) fn grabbed_fns() -> &'static GrabbedFns {
    &GRABBED.get().expect("to be initialized early").fns
}

pub(crate) fn grabbed_globals() -> &'static GrabbedGlobals {
    &GRABBED.get().expect("to be initialized early").globals
}
