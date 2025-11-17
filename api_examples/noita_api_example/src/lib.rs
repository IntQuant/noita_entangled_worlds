use eyre::eyre;
use noita_api::add_lua_fn;
use noita_api::addr_grabber::Globals;
use noita_api::lua::LUA;
use noita_api::lua::LuaState;
use noita_api::lua::lua_bindings::{LUA_REGISTRYINDEX, lua_State};
use noita_api::raw::convert_material_everywhere;
use std::ffi::c_int;
/// # Safety
///
/// Only gets called by lua when loading a module.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaopen_material_converter(lua: *mut lua_State) -> c_int {
    unsafe {
        LUA.lua_createtable(lua, 0, 0);
        LUA.lua_createtable(lua, 0, 0);
        LUA.lua_setmetatable(lua, -2);
        LUA.lua_newuserdata(lua, 0);
        LUA.lua_createtable(lua, 0, 0);
        LUA.lua_setmetatable(lua, -2);
        LUA.lua_setfield(
            lua,
            LUA_REGISTRYINDEX,
            c"luaclose_material_converter".as_ptr(),
        );
        add_lua_fn!(convert);
    }
    1
}
fn convert(lua: LuaState) -> eyre::Result<()> {
    let game_global = Globals::default().game_global_mut();
    let from_id = lua.to_integer(1);
    let to_id = lua.to_integer(2);
    let Some(cosmetics) = game_global
        .m_cell_factory
        .cell_data
        .get(from_id as usize)
        .map(|data| data.grid_cosmetic_particle_config)
    else {
        return Err(eyre!("{from_id} is not a real material >:("));
    };
    //using a lua function in the same call as rust api is UB but idc
    convert_material_everywhere(from_id as i32, to_id as i32)?;
    let data = game_global
        .m_cell_factory
        .cell_data
        .get_mut(from_id as usize)
        .expect("from_id to exist as it did before");
    data.grid_cosmetic_particle_config = cosmetics;
    Ok(())
}
