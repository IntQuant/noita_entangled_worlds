use std::{
    cell::Cell,
    ffi::{c_char, c_int, CStr},
    mem, slice,
};

use eyre::{bail, Context, OptionExt};

use crate::{
    lua_bindings::{lua_CFunction, lua_State, LUA_GLOBALSINDEX},
    LUA,
};

thread_local! {
    static CURRENT_LUA_STATE: Cell<Option<LuaState>> = Cell::default();
}

#[derive(Clone, Copy)]
pub struct LuaState {
    lua: *mut lua_State,
}

impl LuaState {
    pub fn new(lua: *mut lua_State) -> Self {
        Self { lua }
    }

    /// Returns a lua state that is considered "current". Usually set when we get called from noita.
    pub fn current() -> eyre::Result<Self> {
        CURRENT_LUA_STATE
            .get()
            .ok_or_eyre("No current lua state available")
    }

    pub fn make_current(self) {
        CURRENT_LUA_STATE.set(Some(self));
    }

    pub(crate) fn raw(&self) -> *mut lua_State {
        self.lua
    }

    pub fn to_integer(&self, index: i32) -> isize {
        unsafe { LUA.lua_tointeger(self.lua, index) }
    }

    pub fn to_number(&self, index: i32) -> f64 {
        unsafe { LUA.lua_tonumber(self.lua, index) }
    }

    pub fn to_bool(&self, index: i32) -> bool {
        unsafe { LUA.lua_toboolean(self.lua, index) > 0 }
    }

    pub fn to_string(&self, index: i32) -> eyre::Result<String> {
        let mut size = 0;
        let buf = unsafe { LUA.lua_tolstring(self.lua, index, &mut size) };
        if buf.is_null() {
            bail!("Expected a string, but got a null pointer");
        }
        let slice = unsafe { slice::from_raw_parts(buf as *const u8, size) };

        Ok(String::from_utf8(slice.to_owned())
            .context("Attempting to get lua string, expecting it to be utf-8")?)
    }

    pub fn to_cfunction(&self, index: i32) -> lua_CFunction {
        unsafe { LUA.lua_tocfunction(self.lua, index) }
    }

    pub fn push_number(&self, val: f64) {
        unsafe { LUA.lua_pushnumber(self.lua, val) };
    }

    pub fn push_integer(&self, val: isize) {
        unsafe { LUA.lua_pushinteger(self.lua, val) };
    }

    pub fn push_bool(&self, val: bool) {
        unsafe { LUA.lua_pushboolean(self.lua, val as i32) };
    }

    pub fn push_string(&self, s: &str) {
        unsafe {
            LUA.lua_pushlstring(self.lua, s.as_bytes().as_ptr() as *const c_char, s.len());
        }
    }

    pub fn push_nil(&self) {
        unsafe { LUA.lua_pushnil(self.lua) }
    }

    pub fn call(&self, nargs: i32, nresults: i32) {
        unsafe { LUA.lua_call(self.lua, nargs, nresults) };
    }

    pub fn get_global(&self, name: &CStr) {
        unsafe { LUA.lua_getfield(self.lua, LUA_GLOBALSINDEX, name.as_ptr()) };
    }

    pub fn pop_last(&self) {
        unsafe { LUA.lua_settop(self.lua, -2) };
    }
    pub fn pop_last_n(&self, n: i32) {
        unsafe { LUA.lua_settop(self.lua, -1 - (n)) };
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
