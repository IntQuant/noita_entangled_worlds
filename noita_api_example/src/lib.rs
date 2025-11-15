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
    let globals = Globals::default().game_global_mut();
    let from_id = lua.to_integer(1);
    let to_id = lua.to_integer(2);
    let Some(cosmetics) = globals
        .m_cell_factory
        .cell_data
        .get(from_id as usize)
        .map(|data| data.grid_cosmetic_particle_config)
    else {
        noita_api::print!("{from_id} is not a real material >:(");
        return Ok(());
    };
    convert_material_everywhere(from_id as i32, to_id as i32)?;
    let data = globals
        .m_cell_factory
        .cell_data
        .get_mut(from_id as usize)
        .unwrap();
    data.grid_cosmetic_particle_config = cosmetics;
    Ok(())
}
