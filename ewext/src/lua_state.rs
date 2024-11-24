use std::{
    cell::Cell,
    ffi::{c_char, c_int, CStr},
    mem,
};

use eyre::OptionExt;

use crate::{
    lua_bindings::{lua_CFunction, lua_State, LUA_GLOBALSINDEX},
    LUA,
};

thread_local! {
    static CURRENT_LUA_STATE: Cell<Option<LuaState>> = Cell::default();
}

#[derive(Clone, Copy)]
pub(crate) struct LuaState {
    lua: *mut lua_State,
}

impl LuaState {
    pub(crate) fn new(lua: *mut lua_State) -> Self {
        Self { lua }
    }

    /// Returns a lua state that is considered "current". Usually set when we get called from noita.
    pub(crate) fn current() -> eyre::Result<Self> {
        CURRENT_LUA_STATE
            .get()
            .ok_or_eyre("No current lua state available")
    }

    pub(crate) fn make_current(self) {
        CURRENT_LUA_STATE.set(Some(self));
    }

    pub(crate) fn raw(&self) -> *mut lua_State {
        self.lua
    }

    pub(crate) fn to_integer(&self, index: i32) -> isize {
        unsafe { LUA.lua_tointeger(self.lua, index) }
    }

    pub(crate) fn to_cfunction(&self, index: i32) -> lua_CFunction {
        unsafe { LUA.lua_tocfunction(self.lua, index) }
    }

    pub(crate) fn push_string(&self, s: &str) {
        unsafe {
            LUA.lua_pushstring(self.lua, s.as_bytes().as_ptr() as *const c_char);
        }
    }

    pub(crate) fn get_global(&self, name: &CStr) {
        unsafe { LUA.lua_getfield(self.lua, LUA_GLOBALSINDEX, name.as_ptr()) };
    }

    pub(crate) fn pop_last(&self) {
        unsafe { LUA.lua_settop(self.lua, -2) };
    }

    /// Raise an error with message `s`
    ///
    /// This takes String so that it gets deallocated properly, as this functions doesn't return.
    unsafe fn raise_error(&self, s: String) -> ! {
        self.push_string(&s);
        mem::drop(s);
        unsafe { LUA.lua_error(self.lua) };
        // lua_error does not return.
        unreachable!()
    }
}

pub(crate) trait LuaFnRet {
    fn do_return(self, lua: LuaState) -> c_int;
}

/// Function intends to return several values that it has on stack.
pub(crate) struct ValuesOnStack(pub(crate) c_int);

impl LuaFnRet for ValuesOnStack {
    fn do_return(self, _lua: LuaState) -> c_int {
        self.0
    }
}

impl LuaFnRet for () {
    fn do_return(self, _lua: LuaState) -> c_int {
        0
    }
}

impl<R: LuaFnRet> LuaFnRet for eyre::Result<R> {
    fn do_return(self, lua: LuaState) -> c_int {
        match self {
            Ok(ok) => ok.do_return(lua),
            Err(err) => unsafe {
                lua.raise_error(format!("Error in ewext call: {:?}", err));
            },
        }
    }
}
