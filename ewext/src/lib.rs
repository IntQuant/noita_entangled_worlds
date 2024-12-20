#[cfg(feature = "pre2204")]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() {}

use std::{
    arch::asm,
    cell::{LazyCell, RefCell},
    ffi::{c_int, c_void},
    sync::{LazyLock, Mutex},
    thread,
    time::Instant,
};

use addr_grabber::{grab_addrs, grabbed_fns, grabbed_globals};
use eyre::{bail, OptionExt};

use modules::{entity_sync::EntitySync, Module, ModuleCtx};
use net::NetManager;
use noita::{ntypes::Entity, pixel::NoitaPixelRun, ParticleWorldState};
use noita_api::{
    lua::{
        lua_bindings::{lua_State, LUA_REGISTRYINDEX},
        LuaFnRet, LuaState, RawString, ValuesOnStack, LUA,
    },
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

static NETMANAGER: LazyLock<Mutex<Option<NetManager>>> = LazyLock::new(Default::default);
static KEEP_SELF_LOADED: LazyLock<Result<libloading::Library, libloading::Error>> =
    LazyLock::new(|| unsafe { libloading::Library::new("ewext0.dll") });

#[derive(Default)]
struct Modules {
    entity_sync: Option<EntitySync>,
}

#[derive(Default)]
struct ExtState {
    particle_world_state: Option<ParticleWorldState>,
    modules: Modules,
}

impl ExtState {
    fn with_global<T>(f: impl FnOnce(&mut Self) -> T) -> T {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            f(&mut state)
        })
    }
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

    #[expect(clippy::while_let_loop)] // Will probably get more variants in the future
    loop {
        match netman.recv()? {
            NoitaInbound::RawMessage(msg) => kvs.push(msg.into()),
            NoitaInbound::Ready => break,
            _ => bail!("Received unexpected value during init"),
        }
    }

    *NETMANAGER.lock().unwrap() = Some(netman);
    println!("Ok!");
    Ok(kvs)
}

fn netmanager_recv(_lua: LuaState) -> eyre::Result<Option<RawString>> {
    let mut binding = NETMANAGER.lock().unwrap();
    let netmanager = binding.as_mut().unwrap();
    while let Some(msg) = netmanager.try_recv()? {
        match msg {
            NoitaInbound::RawMessage(vec) => return Ok(Some(vec.into())),
            NoitaInbound::Ready => bail!("Unexpected Ready message"),
            NoitaInbound::ProxyToDes(proxy_to_des) => ExtState::with_global(|state| {
                if let Some(entity_sync) = &mut state.modules.entity_sync {
                    entity_sync.handle_proxytodes(proxy_to_des);
                }
            }),
            NoitaInbound::RemoteMessage {
                source,
                message: shared::RemoteMessage::RemoteDes(remote_des),
            } => ExtState::with_global(|state| {
                if let Some(entity_sync) = &mut state.modules.entity_sync {
                    entity_sync.handle_remotedes(source, remote_des);
                }
            }),
        }
    }
    Ok(None)
}

fn netmanager_send(lua: LuaState) -> eyre::Result<()> {
    let arg = lua.to_raw_string(1)?;
    let mut binding = NETMANAGER.lock().unwrap();
    let netmanager = binding.as_mut().unwrap();
    netmanager.send(&shared::NoitaOutbound::Raw(arg))?;

    Ok(())
}

fn netmanager_flush(_lua: LuaState) -> eyre::Result<()> {
    let mut binding = NETMANAGER.lock().unwrap();
    let netmanager = binding.as_mut().unwrap();
    netmanager.flush()
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
    println!(
        "ewext on_world_initialized in thread {:?}",
        thread::current().id()
    );
    grab_addrs(lua);

    STATE.with(|state| {
        let modules = &mut state.borrow_mut().modules;
        modules.entity_sync = Some(EntitySync::default());
    })
}

fn with_every_module(
    f: impl Fn(&mut ModuleCtx, &mut dyn Module) -> eyre::Result<()>,
) -> eyre::Result<()> {
    let mut temp = NETMANAGER.lock().unwrap();
    let mut net = temp.as_mut().ok_or_eyre("Netmanager not available")?;
    STATE.with(|state| {
        let modules = &mut state.borrow_mut().modules;
        let mut ctx = ModuleCtx { net: &mut net };
        let mut errs = Vec::new();
        for module in &mut modules.entity_sync.iter_mut() {
            if let Err(e) = f(&mut ctx, module as &mut dyn Module) {
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

fn module_on_world_init(_lua: LuaState) -> eyre::Result<()> {
    with_every_module(|ctx, module| module.on_world_init(ctx))
}

fn module_on_world_update(_lua: LuaState) -> eyre::Result<()> {
    with_every_module(|ctx, module| module.on_world_update(ctx))
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

fn probe(_lua: LuaState) {
    backtrace::trace(|frame| {
        let ip = frame.ip() as usize;
        println!("Probe: 0x{ip:x}");
        false
    });
}

fn __gc(_lua: LuaState) {
    println!("ewext collected in thread {:?}", thread::current().id());
    NETMANAGER.lock().unwrap().take();
    // TODO this doesn't actually work because it's a thread local
    STATE.with(|state| state.take());
}

/// # Safety
///
/// Only gets called by lua when loading a module.
#[no_mangle]
pub unsafe extern "C" fn luaopen_ewext0(lua: *mut lua_State) -> c_int {
    println!("Initializing ewext");

    if let Err(e) = KEEP_SELF_LOADED.as_ref() {
        println!("Got an error while loading self: {}", e);
    }

    println!(
        "lua_call: 0x{:x}",
        (*LUA.lua_call.as_ref().unwrap()) as usize
    );
    println!(
        "lua_pcall: 0x{:x}",
        (*LUA.lua_pcall.as_ref().unwrap()) as usize
    );

    unsafe {
        LUA.lua_createtable(lua, 0, 0);

        LUA.lua_createtable(lua, 0, 0);
        LUA.lua_setmetatable(lua, -2);

        // Detect module unload. Adapted from NoitaPatcher.
        LUA.lua_newuserdata(lua, 0);
        LUA.lua_createtable(lua, 0, 0);
        add_lua_fn!(__gc);
        LUA.lua_setmetatable(lua, -2);
        LUA.lua_setfield(lua, LUA_REGISTRYINDEX, c"luaclose_ewext".as_ptr());

        add_lua_fn!(init_particle_world_state);
        add_lua_fn!(encode_area);
        add_lua_fn!(make_ephemerial);
        add_lua_fn!(on_world_initialized);
        add_lua_fn!(test_fn);
        add_lua_fn!(bench_fn);
        add_lua_fn!(probe);

        add_lua_fn!(netmanager_connect);
        add_lua_fn!(netmanager_recv);
        add_lua_fn!(netmanager_send);
        add_lua_fn!(netmanager_flush);

        add_lua_fn!(module_on_world_init);
        add_lua_fn!(module_on_world_update);
    }
    println!("Initializing ewext - Ok");
    1
}
