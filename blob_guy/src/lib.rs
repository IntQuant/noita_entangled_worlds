mod blob_guy;
mod chunk;
mod noita;
use crate::blob_guy::{Blob, Pos};
use crate::chunk::{Chunk, ChunkPos};
use crate::noita::ParticleWorldState;
use noita_api::lua::LUA;
use noita_api::lua::LuaState;
use noita_api::lua::lua_bindings::{LUA_REGISTRYINDEX, lua_State};
use noita_api::{add_lua_fn, game_print};
use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use std::cell::{LazyCell, RefCell};
use std::ffi::{c_int, c_void};
use std::hint::black_box;
use std::sync::LazyLock;
const CHUNK_SIZE: usize = 128;
#[derive(Default)]
struct State {
    particle_world_state: Option<ParticleWorldState>,
    blobs: SmallVec<[Blob; 64]>,
    world: FxHashMap<ChunkPos, Chunk>,
}
thread_local! {
    static STATE: LazyCell<RefCell<State>> = LazyCell::new(|| {
        State::default().into()
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
        let world_pointer = lua.to_integer(1);
        let chunk_map_pointer = lua.to_integer(2);
        let material_list_pointer = lua.to_integer(3);
        state.particle_world_state = Some(ParticleWorldState {
            _world_ptr: world_pointer as *mut c_void,
            chunk_map_ptr: chunk_map_pointer as *mut c_void,
            material_list_ptr: material_list_pointer as _,
            blob_guy: noita_api::raw::cell_factory_get_type("blob_guy".into())? as u16,
        });
        Ok(())
    })
}
fn update(_: LuaState) -> eyre::Result<()> {
    STATE.with(|state| {
        let mut state = state.try_borrow_mut()?;
        if state.blobs.is_empty() {
            state.blobs.push(Blob {
                pos: Pos::new(1.0, 2.0),
            })
        }
        for blob in state.blobs.iter_mut() {
            blob.update()
        }
        if let Some(mut pws) = std::mem::take(&mut state.particle_world_state) {
            for i in 0..8 {
                let x = (i * CHUNK_SIZE) as i32 - 512;
                for j in 0..8 {
                    let y = (j * CHUNK_SIZE) as i32 - 512;
                    let chunk = unsafe { pws.encode_area(x, y) };
                    state.world.insert(ChunkPos::new(x, y), chunk);
                }
            }
            game_print(
                state
                    .world
                    .values()
                    .map(|c| c.is_blob.iter().filter(|b| **b).count())
                    .sum::<usize>()
                    .to_string(),
            );
            state.particle_world_state = Some(pws)
        }
        Ok(())
    })
}
