pub mod blob_guy;
pub mod chunk;
pub mod noita;
use crate::blob_guy::Blob;
use crate::chunk::Chunk;
use crate::noita::ParticleWorldState;
use noita_api::add_lua_fn;
use noita_api::lua::LUA;
use noita_api::lua::LuaState;
use noita_api::lua::lua_bindings::{LUA_REGISTRYINDEX, lua_State};
use smallvec::SmallVec;
use std::cell::{LazyCell, RefCell};
use std::ffi::{c_int, c_void};
use std::hint::black_box;
use std::sync::LazyLock;
pub const CHUNK_SIZE: usize = 128;
pub const CHUNK_AMOUNT: usize = 3;
#[derive(Default)]
struct State {
    particle_world_state: ParticleWorldState,
    blobs: SmallVec<[Blob; 8]>,
    world: [Chunk; CHUNK_AMOUNT * CHUNK_AMOUNT],
    blob_guy: u16,
}
thread_local! {
    static STATE: LazyCell<RefCell<State>> = LazyCell::new(|| {
        Default::default()
    });
}
static KEEP_SELF_LOADED: LazyLock<Result<libloading::Library, libloading::Error>> =
    LazyLock::new(|| unsafe { libloading::Library::new("blob_guy.dll") });
/// # Safety
///
/// Only gets called by lua when loading a module.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaopen_blob_guy(lua: *mut lua_State) -> c_int {
    let _ = black_box(KEEP_SELF_LOADED.as_ref());
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
fn init_particle_world_state(lua: LuaState) -> eyre::Result<()> {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let world_ptr = lua.to_integer(1) as *const noita::ntypes::GridWorld;
        let chunk_map_ptr = unsafe { world_ptr.read() }.chunk_map.cell_array;
        let chunk_map = unsafe { std::slice::from_raw_parts(chunk_map_ptr, 512 * 512) };
        let material_list_ptr = lua.to_integer(2) as *const noita::ntypes::CellData;
        let construct_ptr = lua.to_integer(3) as *const c_void;
        let remove_ptr = lua.to_integer(4) as *const c_void;
        let blob_guy = noita_api::raw::cell_factory_get_type("blob_guy".into())? as u16;
        state.blob_guy = blob_guy;
        let pws = ParticleWorldState {
            world_ptr,
            chunk_map,
            material_list_ptr,
            blob_guy,
            blob_ptr: unsafe { material_list_ptr.offset(blob_guy as isize) },
            construct_ptr,
            remove_ptr,
        };
        state.particle_world_state = pws;
        Ok(())
    })
}
fn update(_: LuaState) -> eyre::Result<()> {
    STATE.with(|state| {
        let mut state = state.try_borrow_mut()?;
        state.update()
    })
}
