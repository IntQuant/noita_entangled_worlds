use std::ffi::CStr;

use crate::{
    lua_bindings::{lua_CFunction, lua_State, LUA_GLOBALSINDEX},
    LUA,
};

#[derive(Clone, Copy)]
pub(crate) struct LuaState {
    lua: *mut lua_State,
}

impl LuaState {
    pub(crate) fn new(lua: *mut lua_State) -> Self {
        Self { lua }
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

    pub(crate) fn get_global(&self, name: &CStr) {
        unsafe { LUA.lua_getfield(self.lua, LUA_GLOBALSINDEX, name.as_ptr()) };
    }

    pub(crate) fn pop_last(&self) {
        unsafe { LUA.lua_settop(self.lua, -2) };
    }
}
