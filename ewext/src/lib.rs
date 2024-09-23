use std::{
    cell::{LazyCell, RefCell},
    ffi::{c_int, c_void},
    sync::LazyLock,
};

use lua_bindings::{lua_State, Lua51};
use noita::ParticleWorldState;

mod lua_bindings;

mod noita;

static LUA: LazyLock<Lua51> = LazyLock::new(|| unsafe {
    let lib = libloading::Library::new("./lua51.dll").expect("library to exist");
    Lua51::from_library(lib).expect("library to be lua")
});

thread_local! {
    static STATE: LazyCell<RefCell<ExtState>> = LazyCell::new(|| ExtState::default().into());
}

#[derive(Default)]
struct ExtState {
    particle_world_state: Option<ParticleWorldState>,
}

// const EWEXT: [(&'static str, Function); 1] = [("testfn", None)];

extern "C" fn init_particle_world_state(lua: *mut lua_State) -> c_int {
    println!("\nInitializing particle world state");
    let world_pointer = unsafe { LUA.lua_tointeger(lua, 1) };
    let chunk_map_pointer = unsafe { LUA.lua_tointeger(lua, 2) };
    println!("pws stuff: {world_pointer:?} {chunk_map_pointer:?}");

    STATE.with(|state| {
        state.borrow_mut().particle_world_state = Some(ParticleWorldState::new(
            world_pointer as *mut c_void,
            chunk_map_pointer as *mut c_void,
        ));
    });
    0
}

extern "C" fn get_pixel_pointer(lua: *mut lua_State) -> c_int {
    let x = unsafe { LUA.lua_tointeger(lua, 1) } as i32;
    let y = unsafe { LUA.lua_tointeger(lua, 2) } as i32;

    STATE.with(|state| {
        let state = state.borrow_mut();
        let pws = state.particle_world_state.as_ref().unwrap();
        let pixel_pointer = unsafe { pws.get_cell(x, y) };
        unsafe { LUA.lua_pushinteger(lua, pixel_pointer as isize) };
    });
    1
}

#[no_mangle]
pub extern "C" fn luaopen_ewext(lua: *mut lua_State) -> c_int {
    println!("Initializing ewext");
    unsafe {
        LUA.lua_createtable(lua, 0, 0);

        LUA.lua_pushcclosure(lua, Some(init_particle_world_state), 0);
        LUA.lua_setfield(lua, -2, c"init_particle_world_state".as_ptr());
        LUA.lua_pushcclosure(lua, Some(get_pixel_pointer), 0);
        LUA.lua_setfield(lua, -2, c"get_pixel_pointer".as_ptr());
    }
    println!("Initializing ewext - Ok");
    1
}
