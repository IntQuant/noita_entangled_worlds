use std::{ffi::c_int, sync::LazyLock};

use lua_bindings::{lua_State, Lua51, LUA_GLOBALSINDEX};

mod lua_bindings;

static LUA: LazyLock<Lua51> = LazyLock::new(|| unsafe {
    let lib = libloading::Library::new("./lua51.dll").expect("library to exist");
    Lua51::from_library(lib).expect("library to be lua")
});

// const EWEXT: [(&'static str, Function); 1] = [("testfn", None)];

extern "C" fn test_fn(_lua: *mut lua_State) -> c_int {
    println!("test fn called");
    0
}

#[no_mangle]
pub extern "C" fn luaopen_ewext(lua: *mut lua_State) -> c_int {
    println!("Initializing ewext");
    unsafe {
        LUA.lua_pushcclosure(lua, Some(test_fn), 0);
        LUA.lua_setfield(lua, LUA_GLOBALSINDEX, c"ewext".as_ptr())
    }
    // let mut luastate = unsafe { State::from_ptr(luastateptr) };
    // luastate.new_lib(&EWEXT);
    1
}
