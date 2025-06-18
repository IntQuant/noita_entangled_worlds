#[cfg(feature = "pre2204")]
#[unsafe(no_mangle)]
pub extern "C" fn _unwind_resume() {}

use addr_grabber::{grab_addrs, grabbed_fns, grabbed_globals};
use bimap::BiHashMap;
use eyre::{Context, OptionExt, bail};
use modules::{Module, ModuleCtx, entity_sync::EntitySync};
use net::NetManager;
use noita::{ParticleWorldState, ntypes::Entity, pixel::NoitaPixelRun};
use noita_api::{
    DamageModelComponent, EntityID, VariableStorageComponent,
    lua::{
        LUA, LuaFnRet, LuaGetValue, LuaState, RawString, ValuesOnStack,
        lua_bindings::{LUA_REGISTRYINDEX, lua_State},
    },
};
use noita_api_macro::add_lua_fn;
use rustc_hash::{FxHashMap, FxHashSet};
use shared::des::{Gid, RemoteDes};
use shared::{Destination, NoitaInbound, NoitaOutbound, PeerId, ProxyKV, SpawnOnce, WorldPos};
use std::backtrace::Backtrace;
use std::{
    arch::asm,
    borrow::Cow,
    cell::{LazyCell, RefCell},
    ffi::{c_int, c_void},
    sync::{LazyLock, Mutex, OnceLock, TryLockError},
    time::Instant,
};
use std::{num::NonZero, sync::MutexGuard};
mod addr_grabber;
mod modules;
mod net;
pub mod noita;

thread_local! {
    static STATE: LazyCell<RefCell<ExtState>> = LazyCell::new(|| {
       #[cfg(debug_assertions)]
        println!("Initializing ExtState");
        ExtState::default().into()
    });
}

/// This has a mutex because noita could call us from different threads.
/// It's not expected that several threads try to lock this at once.
static NETMANAGER: LazyLock<Mutex<Option<NetManager>>> = LazyLock::new(Default::default);

static KEEP_SELF_LOADED: LazyLock<Result<libloading::Library, libloading::Error>> =
    LazyLock::new(|| unsafe { libloading::Library::new("ewext1.dll") });
static MY_PEER_ID: OnceLock<PeerId> = OnceLock::new();

fn try_lock_netmanager() -> eyre::Result<MutexGuard<'static, Option<NetManager>>> {
    match NETMANAGER.try_lock() {
        Ok(netman) => Ok(netman),
        Err(TryLockError::WouldBlock) => bail!("Netmanager mutex already locked"),
        Err(TryLockError::Poisoned(_)) => bail!("Netnamager mutex poisoned"),
    }
}

pub(crate) fn my_peer_id() -> PeerId {
    MY_PEER_ID
        .get()
        .copied()
        .expect("peer id to be set by this point")
}

/*pub struct TimeTracker {
    start: Instant,
    message: &'static str,
}

impl TimeTracker {
    pub fn new(message: &'static str) -> Self {
        Self {
            start: Instant::now(),
            message,
        }
    }
}

impl Drop for TimeTracker {
    fn drop(&mut self) {
        /*let elapsed = self.start.elapsed();
        if elapsed.as_millis() > 1 {
            game_print(format!(
                "ewext {} took longer than expected: {} us",
                self.message,
                elapsed.as_micros(),
            ));
        }*/
    }
}*/

#[derive(Default)]
struct Modules {
    entity_sync: Option<EntitySync>,
}

#[derive(Default)]
struct ExtState {
    particle_world_state: Option<ParticleWorldState>,
    modules: Modules,
    player_entity_map: BiHashMap<PeerId, EntityID>,
    fps_by_player: FxHashMap<PeerId, u8>,
    dont_spawn: FxHashSet<Gid>,
    cam_pos: FxHashMap<PeerId, WorldPos>,
}

impl ExtState {
    fn with_global<T>(f: impl FnOnce(&mut Self) -> T) -> eyre::Result<T> {
        STATE.with(|state| {
            let mut state = state
                .try_borrow_mut()
                .wrap_err("Failed to access ExtState")?;
            Ok(f(&mut state))
        })
    }
}

fn init_particle_world_state(lua: LuaState) {
    #[cfg(debug_assertions)]
    println!("\nInitializing particle world state");
    let world_pointer = lua.to_integer(1);
    let chunk_map_pointer = lua.to_integer(2);
    let material_list_pointer = lua.to_integer(3);
    #[cfg(debug_assertions)]
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

pub fn ephemerial(entity_id: u32) -> eyre::Result<()> {
    unsafe {
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
fn make_ephemerial(lua: LuaState) -> eyre::Result<()> {
    let entity_id = lua.to_integer(1) as u32;
    ephemerial(entity_id)?;
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
    #[cfg(debug_assertions)]
    println!("Connecting to proxy...");
    let mut netman = NetManager::new()?;

    let mut kvs = Vec::new();

    loop {
        match netman.recv()? {
            NoitaInbound::RawMessage(msg) => kvs.push(msg.into()),
            NoitaInbound::Ready { my_peer_id } => {
                let _ = MY_PEER_ID.set(my_peer_id);
                break;
            }
            _ => bail!("Received unexpected value during init"),
        }
    }

    *NETMANAGER.lock().unwrap() = Some(netman);
    #[cfg(debug_assertions)]
    println!("Ok!");
    Ok(kvs)
}

fn netmanager_recv(_lua: LuaState) -> eyre::Result<Option<RawString>> {
    let mut binding = try_lock_netmanager()?;
    let netmanager = binding.as_mut().unwrap();
    while let Some(msg) = netmanager.try_recv()? {
        match msg {
            NoitaInbound::RawMessage(vec) => return Ok(Some(vec.into())),
            NoitaInbound::Ready { .. } => bail!("Unexpected Ready message"),
            NoitaInbound::ProxyToDes(proxy_to_des) => ExtState::with_global(|state| {
                let _lock = IN_MODULE_LOCK.lock().unwrap();
                if let Some(entity_sync) = &mut state.modules.entity_sync {
                    entity_sync.handle_proxytodes(proxy_to_des);
                }
            })?,
            NoitaInbound::RemoteMessage {
                source,
                message: shared::RemoteMessage::RemoteDes(remote_des),
            } => ExtState::with_global(|state| {
                let _lock = IN_MODULE_LOCK.lock().unwrap();
                if let Some(entity_sync) = &mut state.modules.entity_sync {
                    match entity_sync.handle_remotedes(
                        source,
                        remote_des,
                        netmanager,
                        &state.player_entity_map,
                        &state.dont_spawn,
                    ) {
                        Ok((Some(gid), _)) => {
                            state.dont_spawn.insert(gid);
                        }
                        Ok((_, Some(pos))) => {
                            state.cam_pos.insert(source, pos);
                        }
                        Ok((_, _)) => {}
                        Err(s) => {
                            let _ = print_error(s);
                        }
                    }
                }
            })?,
        }
    }
    Ok(None)
}

fn netmanager_send(lua: LuaState) -> eyre::Result<()> {
    let arg = lua.to_raw_string(1)?;
    let mut binding = try_lock_netmanager()?;
    let netmanager = binding.as_mut().unwrap();
    netmanager.send(&NoitaOutbound::Raw(arg))?;

    Ok(())
}

fn netmanager_flush(_lua: LuaState) -> eyre::Result<()> {
    let mut binding = try_lock_netmanager()?;
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
    #[cfg(debug_assertions)]
    println!(
        "ewext on_world_initialized in thread {:?}",
        std::thread::current().id()
    );
    grab_addrs(lua);

    STATE.with(|state| {
        let modules = &mut state.borrow_mut().modules;
        modules.entity_sync = Some(EntitySync::default());
    })
}

static IN_MODULE_LOCK: Mutex<()> = Mutex::new(());

fn with_every_module(
    f: impl Fn(&mut ModuleCtx, &mut dyn Module) -> eyre::Result<()>,
) -> eyre::Result<()> {
    let _lock = IN_MODULE_LOCK.lock().unwrap();
    let mut temp = try_lock_netmanager()?;
    let net = temp.as_mut().ok_or_eyre("Netmanager not available")?;
    ExtState::with_global(|state| {
        let mut ctx = ModuleCtx {
            net,
            player_map: &mut state.player_entity_map,
            fps_by_player: &mut state.fps_by_player,
            dont_spawn: &state.dont_spawn,
            camera_pos: &mut state.cam_pos,
        };
        let mut errs = Vec::new();
        for module in state.modules.entity_sync.iter_mut() {
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
    })?
}

fn module_on_world_init(_lua: LuaState) -> eyre::Result<()> {
    with_every_module(|ctx, module| module.on_world_init(ctx))
}

fn module_on_world_update(_lua: LuaState) -> eyre::Result<()> {
    //let _tracker = TimeTracker::new("on_world_update");
    with_every_module(|ctx, module| module.on_world_update(ctx))
}

fn module_on_new_entity(lua: LuaState) -> eyre::Result<()> {
    let entity = EntityID::try_from(lua.to_string(1)?.parse::<isize>()?)?;
    with_every_module(|_, module| module.on_new_entity(entity, true))
}

fn module_on_projectile_fired(lua: LuaState) -> eyre::Result<()> {
    // Could be called while we do game_shoot_projectile call, leading to a deadlock.
    if IN_MODULE_LOCK.try_lock().is_err() {
        return Ok(());
    }
    let (
        (shooter_id, projectile_id, initial_rng, position_x),
        (position_y, target_x, target_y, multicast_index),
    ) = noita_api::lua::LuaGetValue::get(lua, -1)?;
    with_every_module(|ctx, module| {
        module.on_projectile_fired(
            ctx,
            shooter_id,
            projectile_id,
            initial_rng,
            (position_x, position_y),
            (target_x, target_y),
            multicast_index,
        )
    })
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
    #[cfg(debug_assertions)]
    backtrace::trace(|frame| {
        let ip = frame.ip() as usize;
        println!("Probe: 0x{ip:x}");
        false
    });
}

fn __gc(_lua: LuaState) {
    #[cfg(debug_assertions)]
    println!(
        "ewext collected in thread {:?}",
        std::thread::current().id()
    );
    NETMANAGER.lock().unwrap().take();
    // TODO this doesn't actually work because it's a thread local
    STATE.with(|state| state.take());
}

pub(crate) fn print_error(error: eyre::Report) -> eyre::Result<()> {
    let lua = LuaState::current()?;
    lua.get_global(c"EwextPrintError");
    lua.push_string(&format!("{:?}\n{}", error, Backtrace::force_capture()));
    lua.call(1, 0i32)
        .wrap_err("Failed to call EwextPrintError")?;
    Ok(())
}

pub(crate) fn print(string: &str) -> eyre::Result<()> {
    let lua = LuaState::current()?;
    lua.get_global(c"EwextPrint");
    lua.push_string(string);
    lua.call(1, 0i32)
        .wrap_err("Failed to call EwextPrintError")?;
    Ok(())
}

/// # Safety
///
/// Only gets called by lua when loading a module.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaopen_ewext1(lua: *mut lua_State) -> c_int {
    #[cfg(debug_assertions)]
    println!("Initializing ewext");

    if let Err(_e) = KEEP_SELF_LOADED.as_ref() {
        #[cfg(debug_assertions)]
        println!("Got an error while loading self: {}", _e);
    }
    #[cfg(debug_assertions)]
    println!(
        "lua_call: 0x{:x}",
        (*LUA.lua_call.as_ref().unwrap()) as usize
    );
    #[cfg(debug_assertions)]
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
        add_lua_fn!(module_on_new_entity);
        add_lua_fn!(module_on_projectile_fired);

        fn sync_projectile(lua: LuaState) -> eyre::Result<()> {
            ExtState::with_global(|state| {
                let entity = lua.to_string(1)?.parse::<isize>()?;
                let peer = PeerId::from_hex(&lua.to_string(2)?)?;
                let mut rng: u64 =
                    u32::from_le_bytes(lua.to_string(3)?.parse::<i32>()?.to_le_bytes()) as u64;
                if rng == 0 {
                    rng = 1;
                }
                let mut peer_n = peer.0;
                while peer_n == 0 {
                    peer_n = peer_n.overflowing_add(rng).0
                }
                let gid = peer_n.overflowing_mul(rng).0;
                let entity_sync = state
                    .modules
                    .entity_sync
                    .as_mut()
                    .ok_or_eyre("No entity sync module loaded")?;
                entity_sync.sync_projectile(
                    EntityID(NonZero::try_from(entity)?),
                    Gid(gid),
                    peer,
                )?;
                Ok(())
            })?
        }
        add_lua_fn!(sync_projectile);

        fn des_item_thrown(lua: LuaState) -> eyre::Result<()> {
            ExtState::with_global(|state| {
                let entity_sync = state
                    .modules
                    .entity_sync
                    .as_mut()
                    .ok_or_eyre("No entity sync module loaded")?;
                entity_sync.cross_item_thrown(LuaGetValue::get(lua, -1)?)?;
                Ok(())
            })?
        }
        add_lua_fn!(des_item_thrown);

        fn des_death_notify(lua: LuaState) -> eyre::Result<()> {
            ExtState::with_global(|state| {
                let entity_sync = state
                    .modules
                    .entity_sync
                    .as_mut()
                    .ok_or_eyre("No entity sync module loaded")?;
                let entity_killed = EntityID::try_from(lua.to_integer(1))
                    .wrap_err("Expected to have a valid entity_killed")?;
                let wait_on_kill = lua.to_bool(2);
                let x = lua.to_number(3);
                let y = lua.to_number(4);
                let file = lua
                    .to_string(5)
                    .wrap_err("Expected to have a valid filepath")?;
                let entity_responsible = EntityID::try_from(lua.to_integer(6)).ok();
                let pos = WorldPos::from_f64(x, y);
                entity_sync.cross_death_notify(
                    entity_killed,
                    wait_on_kill,
                    pos,
                    file.to_string(),
                    entity_responsible,
                )?;
                Ok(())
            })?
        }
        add_lua_fn!(des_death_notify);

        fn notrack(lua: LuaState) -> eyre::Result<()> {
            ExtState::with_global(|state| {
                let entity_sync = state
                    .modules
                    .entity_sync
                    .as_mut()
                    .ok_or_eyre("No entity sync module loaded")?;
                let entity_killed: Option<EntityID> = LuaGetValue::get(lua, -1)?;
                let entity_killed =
                    entity_killed.ok_or_eyre("Expected to have a valid entity_killed")?;
                entity_sync.notrack_entity(entity_killed);
                Ok(())
            })?
        }
        add_lua_fn!(notrack);

        fn track(lua: LuaState) -> eyre::Result<()> {
            ExtState::with_global(|state| {
                let entity_sync = state
                    .modules
                    .entity_sync
                    .as_mut()
                    .ok_or_eyre("No entity sync module loaded")?;
                let entity_killed: Option<EntityID> = LuaGetValue::get(lua, -1)?;
                let entity_killed =
                    entity_killed.ok_or_eyre("Expected to have a valid entity_killed")?;
                entity_sync.track_entity(entity_killed);
                Ok(())
            })?
        }
        add_lua_fn!(track);

        fn register_player_entity(lua: LuaState) -> eyre::Result<()> {
            let (peer_id, entity): (Cow<'_, str>, Option<EntityID>) = LuaGetValue::get(lua, -1)?;
            let peer_id = PeerId::from_hex(&peer_id)?;
            let entity = entity.ok_or_eyre("Expected a valid entity")?;
            if entity
                .iter_all_components_of_type_including_disabled::<VariableStorageComponent>(None)?
                .all(|var| var.name().unwrap_or("".into()) != "ew_peer_id")
            {
                let var = entity.add_component::<VariableStorageComponent>()?;
                var.set_name("ew_peer_id".into())?;
                var.set_value_string(peer_id.0.to_string().into())?;
            }
            ExtState::with_global(|state| {
                state.player_entity_map.insert(peer_id, entity);
                Ok(())
            })?
        }
        add_lua_fn!(register_player_entity);

        fn set_player_fps(lua: LuaState) -> eyre::Result<()> {
            let peer = PeerId::from_hex(&lua.to_string(1)?)?;
            let fps = lua.to_string(2)?.parse::<u8>()?;
            ExtState::with_global(|state| {
                state.fps_by_player.insert(peer, fps);
                Ok(())
            })?
        }
        add_lua_fn!(set_player_fps);

        fn find_by_gid(lua: LuaState) -> eyre::Result<Option<EntityID>> {
            ExtState::with_global(|state| {
                let gid = lua.to_string(1)?.parse::<u64>()?;
                let entity_sync = state
                    .modules
                    .entity_sync
                    .as_mut()
                    .ok_or_eyre("No entity sync module loaded")?;
                Ok(entity_sync.find_by_gid(Gid(gid)))
            })?
        }
        add_lua_fn!(find_by_gid);

        fn des_chest_opened(lua: LuaState) -> eyre::Result<()> {
            ExtState::with_global(|state| {
                let x = lua.to_string(1)?.parse::<f64>()?;
                let y = lua.to_string(2)?.parse::<f64>()?;
                let rx = lua.to_string(3)?.parse::<f32>()?;
                let ry = lua.to_string(4)?.parse::<f32>()?;
                let file = lua.to_string(5)?.to_string();
                let gid = Gid(lua.to_string(6)?.parse::<u64>()?);
                let is_mine = lua.to_string(7)?.parse::<u8>()? == 1;
                let entity_sync = state
                    .modules
                    .entity_sync
                    .as_mut()
                    .ok_or_eyre("No entity sync module loaded")?;
                let mut temp = try_lock_netmanager()?;
                let net = temp.as_mut().ok_or_eyre("Netmanager not available")?;
                if is_mine {
                    net.send(&NoitaOutbound::RemoteMessage {
                        reliable: true,
                        destination: Destination::Peer(my_peer_id()),
                        message: shared::RemoteMessage::RemoteDes(RemoteDes::ChestOpen(
                            gid,
                            x as i32,
                            y as i32,
                            file.clone(),
                            rx,
                            ry,
                        )),
                    })?;
                    for (has_interest, peer) in entity_sync.iter_peers(&state.player_entity_map) {
                        if has_interest {
                            net.send(&NoitaOutbound::RemoteMessage {
                                reliable: true,
                                destination: Destination::Peer(peer),
                                message: shared::RemoteMessage::RemoteDes(RemoteDes::ChestOpen(
                                    gid,
                                    x as i32,
                                    y as i32,
                                    file.clone(),
                                    rx,
                                    ry,
                                )),
                            })?;
                        } else {
                            net.send(&NoitaOutbound::RemoteMessage {
                                reliable: true,
                                destination: Destination::Peer(peer),
                                message: shared::RemoteMessage::RemoteDes(RemoteDes::SpawnOnce(
                                    WorldPos::from_f64(x, y),
                                    SpawnOnce::Chest(file.clone(), rx, ry),
                                )),
                            })?;
                        }
                    }
                } else if let Some(peer) = entity_sync.find_peer_by_gid(gid) {
                    net.send(&NoitaOutbound::RemoteMessage {
                        reliable: true,
                        destination: Destination::Peer(*peer),
                        message: shared::RemoteMessage::RemoteDes(RemoteDes::ChestOpenRequest(
                            gid, x as i32, y as i32, file, rx, ry,
                        )),
                    })?;
                }
                Ok(())
            })?
        }
        add_lua_fn!(des_chest_opened);

        fn des_broken_wand(lua: LuaState) -> eyre::Result<()> {
            ExtState::with_global(|state| {
                let x = lua.to_string(1)?.parse::<f64>()?;
                let y = lua.to_string(2)?.parse::<f64>()?;
                let mut temp = try_lock_netmanager()?;
                let net = temp.as_mut().ok_or_eyre("Netmanager not available")?;
                for peer in state.player_entity_map.left_values() {
                    if *peer != my_peer_id() {
                        net.send(&NoitaOutbound::RemoteMessage {
                            reliable: true,
                            destination: Destination::Peer(*peer),
                            message: shared::RemoteMessage::RemoteDes(RemoteDes::SpawnOnce(
                                WorldPos::from_f64(x, y),
                                SpawnOnce::BrokenWand,
                            )),
                        })?;
                    }
                }
                Ok(())
            })?
        }
        add_lua_fn!(des_broken_wand);

        fn set_log(lua: LuaState) -> eyre::Result<()> {
            ExtState::with_global(|state| {
                state
                    .modules
                    .entity_sync
                    .as_mut()
                    .unwrap()
                    .set_perf(lua.to_bool(1));
                Ok(())
            })?
        }
        add_lua_fn!(set_log);
    }
    #[cfg(debug_assertions)]
    println!("Initializing ewext - Ok");
    1
}
