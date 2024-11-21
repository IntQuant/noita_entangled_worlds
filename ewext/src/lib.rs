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

struct SavedWorldState {
    game_global: usize,
    world_state_entity: usize,
}

#[derive(Default)]
struct ExtState {
    particle_world_state: Option<ParticleWorldState>,
    saved_world_state: Option<SavedWorldState>,
}

// const EWEXT: [(&'static str, Function); 1] = [("testfn", None)];

unsafe extern "C" fn init_particle_world_state(lua: *mut lua_State) -> c_int {
    println!("\nInitializing particle world state");
    let world_pointer = unsafe { LUA.lua_tointeger(lua, 1) };
    let chunk_map_pointer = unsafe { LUA.lua_tointeger(lua, 2) };
    let material_list_pointer = unsafe { LUA.lua_tointeger(lua, 3) };
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

unsafe extern "C" fn encode_area(lua: *mut lua_State) -> c_int {
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

unsafe fn save_world_state() {
    let game_global = grabbed_globals().game_global.read();
    let world_state_entity = grabbed_globals().world_state_entity.read();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.saved_world_state = Some(SavedWorldState {
            game_global,
            world_state_entity,
        })
    });
}

unsafe fn load_world_state() {
    println!("Loading world state");
    STATE.with(|state| {
        let state = state.borrow_mut();
        let saved_ws = state.saved_world_state.as_ref().unwrap();
        grabbed_globals().game_global.write(saved_ws.game_global);
        grabbed_globals()
            .world_state_entity
            .write(saved_ws.world_state_entity);
    });
}

unsafe extern "C" fn save_world_state_lua(_lua: *mut lua_State) -> i32 {
    save_world_state();
    0
}

unsafe extern "C" fn load_world_state_lua(_lua: *mut lua_State) -> i32 {
    load_world_state();
    0
}

unsafe extern "C" fn make_ephemerial(lua: *mut lua_State) -> c_int {
    let lua_state = LuaState::new(lua);
    unsafe {
        let entity_id = lua_state.to_integer(1) as u32;

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
        // let entity = (state.fns.as_ref().unwrap().get_entity)(entity_manager, entity_id);
        entity.cast::<c_void>().offset(0x8).cast::<u32>().write(0);
    }
    0
}

unsafe extern "C" fn on_world_initialized(lua: *mut lua_State) -> c_int {
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

        LUA.lua_pushcclosure(lua, Some(init_particle_world_state), 0);
        LUA.lua_setfield(lua, -2, c"init_particle_world_state".as_ptr());
        LUA.lua_pushcclosure(lua, Some(encode_area), 0);
        LUA.lua_setfield(lua, -2, c"encode_area".as_ptr());
        LUA.lua_pushcclosure(lua, Some(load_world_state_lua), 0);
        LUA.lua_setfield(lua, -2, c"load_world_state".as_ptr());
        LUA.lua_pushcclosure(lua, Some(save_world_state_lua), 0);
        LUA.lua_setfield(lua, -2, c"save_world_state".as_ptr());
        LUA.lua_pushcclosure(lua, Some(make_ephemerial), 0);
        LUA.lua_setfield(lua, -2, c"make_ephemerial".as_ptr());
        LUA.lua_pushcclosure(lua, Some(on_world_initialized), 0);
        LUA.lua_setfield(lua, -2, c"on_world_initialized".as_ptr());
    }
    println!("Initializing ewext - Ok");
    1
}
