use std::{
    arch::asm,
    cell::{LazyCell, RefCell},
    ffi::{c_int, c_void},
    sync::{LazyLock, Mutex},
    time::Instant,
};

use addr_grabber::{grab_addrs, grabbed_fns, grabbed_globals};
use eyre::{bail, OptionExt};

use modules::{entity_sync::EntitySync, Module};
use net::NetManager;
use noita::{ntypes::Entity, pixel::NoitaPixelRun, ParticleWorldState};
use noita_api::{
    lua::{lua_bindings::lua_State, LuaFnRet, LuaState, RawString, ValuesOnStack, LUA},
    DamageModelComponent,
};
use noita_api_macro::add_lua_fn;
use shared::{NoitaInbound, ProxyKV};

mod addr_grabber;
mod modules;
mod net;
pub mod noita;

thread_local! {
    static STATE: LazyCell<RefCell<ExtState>> = LazyCell::new(|| {
        println!("Initializing ExtState");
        ExtState::default().into()
    });
}

static NETMANAGER: LazyLock<Mutex<Option<NetManager>>> = LazyLock::new(|| Default::default());

#[derive(Default)]
struct ExtState {
    particle_world_state: Option<ParticleWorldState>,
    modules: Vec<Box<dyn Module>>,
}

fn init_particle_world_state(lua: LuaState) {
    println!("\nInitializing particle world state");
    let world_pointer = lua.to_integer(1);
    let chunk_map_pointer = lua.to_integer(2);
    let material_list_pointer = lua.to_integer(3);
    println!("pws stuff: {world_pointer:?} {chunk_map_pointer:?}");

    STATE.with(|state| {
        state.borrow_mut().particle_world_state = Some(ParticleWorldState {
            _world_ptr: world_pointer as *mut c_void,
            chunk_map_ptr: chunk_map_pointer as *mut c_void,
            material_list_ptr: material_list_pointer as _,
            runner: Default::default(),
        });
    });
}

fn encode_area(lua: LuaState) -> ValuesOnStack {
    let lua = lua.raw();
    let start_x = unsafe { LUA.lua_tointeger(lua, 1) } as i32;
    let start_y = unsafe { LUA.lua_tointeger(lua, 2) } as i32;
    let end_x = unsafe { LUA.lua_tointeger(lua, 3) } as i32;
    let end_y = unsafe { LUA.lua_tointeger(lua, 4) } as i32;
    let encoded_buffer = unsafe { LUA.lua_tointeger(lua, 5) } as *mut NoitaPixelRun;

    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let pws = state.particle_world_state.as_mut().unwrap();
        let runs = unsafe { pws.encode_area(start_x, start_y, end_x, end_y, encoded_buffer) };
        unsafe { LUA.lua_pushinteger(lua, runs as isize) };
    });
    ValuesOnStack(1)
}

fn make_ephemerial(lua: LuaState) -> eyre::Result<()> {
    unsafe {
        let entity_id = lua.to_integer(1) as u32;

        let entity_manager = grabbed_globals().entity_manager.read();
        let mut entity: *mut Entity;
        asm!(
            "mov ecx, {entity_manager}",
            "push {entity_id:e}",
            "call {get_entity}",
            entity_manager = in(reg) entity_manager,
            get_entity = in(reg) grabbed_fns().get_entity,
            entity_id = in(reg) entity_id,
            clobber_abi("C"),
            out("ecx") _,
            out("eax") entity,
        );
        if entity.is_null() {
            bail!("Entity {} not found", entity_id);
        }
        entity.cast::<c_void>().offset(0x8).cast::<u32>().write(0);
    }
    Ok(())
}

struct InitKV {
    key: String,
    value: String,
}

impl From<ProxyKV> for InitKV {
    fn from(value: ProxyKV) -> Self {
        InitKV {
            key: value.key,
            value: value.value,
        }
    }
}

fn netmanager_connect(_lua: LuaState) -> eyre::Result<Vec<RawString>> {
    println!("Connecting to proxy...");
    let mut netman = NetManager::new()?;

    let mut kvs = Vec::new();

    loop {
        match netman
            .recv()?
            .ok_or_eyre("Expected to be in non-blocking mode")?
        {
            // shared::NoitaInbound::ProxyKV(proxy_kv) => kvs.push(proxy_kv.into()),
            NoitaInbound::RawMessage(msg) => kvs.push(msg.into()),
            NoitaInbound::Ready => break,
            // _ => bail!("Received an unexpected message type during init"),
        }
    }

    netman.switch_to_non_blocking()?;

    *NETMANAGER.lock().unwrap() = Some(netman);
    println!("Ok!");
    Ok(kvs)
}

fn netmanager_recv(_lua: LuaState) -> eyre::Result<Option<RawString>> {
    let mut binding = NETMANAGER.lock().unwrap();
    let netmanager = binding.as_mut().unwrap();
    Ok(match netmanager.recv()? {
        Some(NoitaInbound::RawMessage(msg)) => Some(msg.into()),
        Some(NoitaInbound::Ready) => {
            bail!("Unexpected Ready message")
        }
        None => None,
    })
}

fn netmanager_send(lua: LuaState) -> eyre::Result<()> {
    let arg = lua.to_raw_string(1)?;
    let mut binding = NETMANAGER.lock().unwrap();
    let netmanager = binding.as_mut().unwrap();
    netmanager.send(&shared::NoitaOutbound::Raw(arg))?;

    Ok(())
}

impl LuaFnRet for InitKV {
    fn do_return(self, lua: LuaState) -> c_int {
        lua.create_table(2, 0);
        lua.push_string(&self.key);
        lua.rawset_table(-2, 1);
        lua.push_string(&self.value);
        lua.rawset_table(-2, 2);
        1
    }
}

fn on_world_initialized(lua: LuaState) {
    grab_addrs(lua);

    STATE.with(|state| {
        let modules = &mut state.borrow_mut().modules;
        modules.push(Box::new(EntitySync::default()));
    })
}

fn with_every_module(f: impl Fn(&mut dyn Module) -> eyre::Result<()>) -> eyre::Result<()> {
    STATE.with(|state| {
        let modules = &mut state.borrow_mut().modules;
        let mut errs = Vec::new();
        for module in modules {
            if let Err(e) = f(module.as_mut()) {
                errs.push(e);
            }
        }
        if errs.len() == 1 {
            return Err(errs.remove(0));
        }
        if errs.len() > 1 {
            bail!("Multiple errors while running ewext modules:\n{:?}", errs)
        }
        Ok(())
    })
}

fn module_on_world_update(_lua: LuaState) -> eyre::Result<()> {
    with_every_module(|module| module.on_world_update())
}

fn bench_fn(_lua: LuaState) -> eyre::Result<()> {
    let start = Instant::now();
    let iters = 10000;
    for _ in 0..iters {
        let player = noita_api::raw::entity_get_closest_with_tag(0.0, 0.0, "player_unit".into())?
            .ok_or_eyre("Entity not found")?;
        noita_api::raw::entity_set_transform(player, 0.0, Some(0.0), None, None, None)?;
    }
    let elapsed = start.elapsed();

    noita_api::raw::game_print(
        format!(
            "Took {}us to test, {}ns per call",
            elapsed.as_micros(),
            elapsed.as_nanos() / iters
        )
        .into(),
    )?;

    Ok(())
}

fn test_fn(_lua: LuaState) -> eyre::Result<()> {
    let player = noita_api::raw::entity_get_closest_with_tag(0.0, 0.0, "player_unit".into())?
        .ok_or_eyre("Entity not found")?;
    let damage_model: DamageModelComponent = player.get_first_component(None)?;
    let hp = damage_model.hp()?;
    damage_model.set_hp(hp - 1.0)?;

    let (x, y, _, _, _) = noita_api::raw::entity_get_transform(player)?;

    noita_api::raw::game_print(
        format!("Component: {:?}, Hp: {}", damage_model.0, hp * 25.0,).into(),
    )?;

    let entities = noita_api::raw::entity_get_in_radius_with_tag(x, y, 300.0, "enemy".into())?;
    noita_api::raw::game_print(format!("{:?}", entities).into())?;

    // noita::api::raw::entity_set_transform(player, 0.0, 0.0, 0.0, 1.0, 1.0)?;

    Ok(())
}

/// # Safety
///
/// Only gets called by lua when loading a module.
#[no_mangle]
pub unsafe extern "C" fn luaopen_ewext0(lua: *mut lua_State) -> c_int {
    println!("Initializing ewext");

    // Reset some stuff
    STATE.with(|state| state.take());
    NETMANAGER.lock().unwrap().take();

    unsafe {
        LUA.lua_createtable(lua, 0, 0);

        add_lua_fn!(init_particle_world_state);
        add_lua_fn!(encode_area);
        add_lua_fn!(make_ephemerial);
        add_lua_fn!(on_world_initialized);
        add_lua_fn!(test_fn);
        add_lua_fn!(bench_fn);

        add_lua_fn!(netmanager_connect);
        add_lua_fn!(netmanager_recv);
        add_lua_fn!(netmanager_send);

        add_lua_fn!(module_on_world_update);
    }
    println!("Initializing ewext - Ok");
    1
}
