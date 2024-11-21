use std::{
    arch::asm,
    cell::{LazyCell, RefCell},
    ffi::{c_int, c_void},
    sync::LazyLock,
};

use addr_grabber::{grab_addrs, grabbed_fns, grabbed_globals};
use lua_bindings::{lua_State, Lua51};
use lua_state::LuaState;
use noita::{ntypes::Entity, NoitaPixelRun, ParticleWorldState};
use noita_api_macro::add_lua_fn;

mod lua_bindings;
mod lua_state;

mod noita;

mod addr_grabber;

static LUA: LazyLock<Lua51> = LazyLock::new(|| unsafe {
    let lib = libloading::Library::new("./lua51.dll").expect("library to exist");
    Lua51::from_library(lib).expect("library to be lua")
});

thread_local! {
    static STATE: LazyCell<RefCell<ExtState>> = LazyCell::new(|| {
        println!("Initializing ExtState");
        ExtState::default().into()
    });
}

#[derive(Default)]
struct ExtState {
    particle_world_state: Option<ParticleWorldState>,
}

fn init_particle_world_state(lua: LuaState) -> c_int {
    println!("\nInitializing particle world state");
    let world_pointer = lua.to_integer(1);
    let chunk_map_pointer = lua.to_integer(2);
    let material_list_pointer = lua.to_integer(3);
    println!("pws stuff: {world_pointer:?} {chunk_map_pointer:?}");

    STATE.with(|state| {
        state.borrow_mut().particle_world_state = Some(ParticleWorldState {
            _world_ptr: world_pointer as *mut c_void,
            chunk_map_ptr: chunk_map_pointer as *mut c_void,
            material_list_ptr: material_list_pointer as _,
            runner: Default::default(),
        });
    });
    0
}

fn encode_area(lua: LuaState) -> c_int {
    let lua = lua.raw();
    let start_x = unsafe { LUA.lua_tointeger(lua, 1) } as i32;
    let start_y = unsafe { LUA.lua_tointeger(lua, 2) } as i32;
    let end_x = unsafe { LUA.lua_tointeger(lua, 3) } as i32;
    let end_y = unsafe { LUA.lua_tointeger(lua, 4) } as i32;
    let encoded_buffer = unsafe { LUA.lua_tointeger(lua, 5) } as *mut NoitaPixelRun;

    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let pws = state.particle_world_state.as_mut().unwrap();
        let runs = unsafe { pws.encode_area(start_x, start_y, end_x, end_y, encoded_buffer) };
        unsafe { LUA.lua_pushinteger(lua, runs as isize) };
    });
    1
}

fn make_ephemerial(lua: LuaState) -> c_int {
    unsafe {
        let entity_id = lua.to_integer(1) as u32;

        let entity_manager = grabbed_globals().entity_manager.read();
        let mut entity: *mut Entity;
        asm!(
            "mov ecx, {entity_manager}",
            "push {entity_id:e}",
            "call {get_entity}",
            entity_manager = in(reg) entity_manager,
            get_entity = in(reg) grabbed_fns().get_entity,
            entity_id = in(reg) entity_id,
            clobber_abi("C"),
            out("ecx") _,
            out("eax") entity,
        );
        if !entity.is_null() {
            entity.cast::<c_void>().offset(0x8).cast::<u32>().write(0);
        }
    }
    0
}

fn on_world_initialized(lua: LuaState) -> i32 {
    grab_addrs(lua);
    0
}

/// # Safety
///
/// Only gets called by lua when loading a module.
#[no_mangle]
pub unsafe extern "C" fn luaopen_ewext0(lua: *mut lua_State) -> c_int {
    println!("Initializing ewext");
    unsafe {
        LUA.lua_createtable(lua, 0, 0);

        add_lua_fn!(init_particle_world_state);
        add_lua_fn!(encode_area);
        add_lua_fn!(make_ephemerial);
        add_lua_fn!(on_world_initialized);
    }
    println!("Initializing ewext - Ok");
    1
}
