use crate::{lua_bindings::lua_State, LUA};

#[derive(Clone, Copy)]
pub(crate) struct LuaState(*mut lua_State);

impl LuaState {
    pub(crate) fn new(lua: *mut lua_State) -> Self {
        Self(lua)
    }

    pub(crate) fn to_integer(&self, index: i32) -> isize {
        unsafe { LUA.lua_tointeger(self.0, index) }
    }
}
