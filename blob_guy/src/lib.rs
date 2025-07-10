pub mod blob_guy;
pub mod chunk;
use crate::blob_guy::Blob;
use crate::chunk::Chunks;
use noita_api::add_lua_fn;
use noita_api::lua::LUA;
use noita_api::lua::LuaState;
use noita_api::lua::lua_bindings::{LUA_REGISTRYINDEX, lua_State};
use noita_api::noita::world::ParticleWorldState;
use smallvec::SmallVec;
use std::cell::RefCell;
use std::ffi::c_int;
use std::mem::MaybeUninit;
pub const CHUNK_SIZE: usize = 128;
pub const CHUNK_AMOUNT: usize = 3;
struct State {
    particle_world_state: MaybeUninit<ParticleWorldState>,
    blobs: SmallVec<[Blob; 8]>,
    world: Chunks,
    blob_guy: u16,
}
thread_local! {
    static STATE: RefCell<State> = State {
        particle_world_state: MaybeUninit::uninit(),
        blobs: Default::default(),
        world: Default::default(),
        blob_guy: 0,
    }.into();
}
/// # Safety
///
/// Only gets called by lua when loading a module.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaopen_blob_guy(lua: *mut lua_State) -> c_int {
    unsafe {
        LUA.lua_createtable(lua, 0, 0);
        LUA.lua_createtable(lua, 0, 0);
        LUA.lua_setmetatable(lua, -2);
        LUA.lua_newuserdata(lua, 0);
        LUA.lua_createtable(lua, 0, 0);
        LUA.lua_setmetatable(lua, -2);
        LUA.lua_setfield(lua, LUA_REGISTRYINDEX, c"luaclose_blob_guy".as_ptr());
        add_lua_fn!(init_particle_world_state);
        add_lua_fn!(update);
    }
    1
}
fn init_particle_world_state(_: LuaState) -> eyre::Result<()> {
    STATE.with(|state| {
        let mut state = state.try_borrow_mut()?;
        let blob_guy = noita_api::raw::cell_factory_get_type("blob_guy".into())? as u16;
        state.blob_guy = blob_guy;
        state.particle_world_state = MaybeUninit::new(ParticleWorldState::new()?);
        Ok(())
    })
}
fn update(_: LuaState) -> eyre::Result<()> {
    STATE.with(|state| {
        let mut state = state.try_borrow_mut()?;
        state.update()
    })
}
