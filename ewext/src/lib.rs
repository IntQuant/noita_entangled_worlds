use std::{ffi::c_int, sync::LazyLock};

use lua_bindings::{lua_State, Lua51};

mod lua_bindings;

mod noita;

static LUA: LazyLock<Lua51> = LazyLock::new(|| unsafe {
    let lib = libloading::Library::new("./lua51.dll").expect("library to exist");
    Lua51::from_library(lib).expect("library to be lua")
});

// const EWEXT: [(&'static str, Function); 1] = [("testfn", None)];

extern "C" fn test_fn(_lua: *mut lua_State) -> c_int {
    println!("\nStarting trace");
    backtrace::trace(|frame| {
        // let ip = frame.ip();
        let symbol_address = frame.symbol_address();

        print!("symbol: {:#08X}", symbol_address as usize);
        if let Some(base) = frame.module_base_address() {
            print!(" base: {:#08X}", base as usize);
        }
        // Resolve this instruction pointer to a symbol name
        backtrace::resolve_frame(frame, |symbol| {
            if let Some(name) = symbol.name() {
                print!(" name: {name}");
            }
            if let Some(filename) = symbol.filename() {
                print!(" file: {}", filename.display());
            }
        });
        println!();

        for i in 0..16 {
            let b: u8 =
                unsafe { std::ptr::read_volatile((symbol_address as *const u8).wrapping_add(i)) };
            print!("{:02X} ", b);
        }
        println!();

        true // keep going to the next frame
    });
    println!("End trace\n");
    0
}

#[no_mangle]
pub extern "C" fn luaopen_ewext(lua: *mut lua_State) -> c_int {
    println!("Initializing ewext");
    unsafe {
        LUA.lua_pushcclosure(lua, Some(test_fn), 0);
        // LUA.lua_setfield(lua, LUA_GLOBALSINDEX, c"ewext".as_ptr())
    }
    // let mut luastate = unsafe { State::from_ptr(luastateptr) };
    // luastate.new_lib(&EWEXT);
    1
}
