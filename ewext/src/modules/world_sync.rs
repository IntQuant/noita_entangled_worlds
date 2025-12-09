use crate::modules::world_sync::change_tracker::ChangeTracker;
use crate::modules::{Module, ModuleCtx};
use crate::my_peer_id;
use eyre::{ContextCompat, eyre};
use noita_api::addr_grabber::Globals;
use noita_api::heap::Ptr;
use noita_api::noita::types::*;
use noita_api::noita::world::ParticleWorldState;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use shared::NoitaOutbound;
use shared::world_sync::{
    CHUNK_SIZE, ChunkCoord, NoitaWorldUpdate, Pixel, ProxyToWorldSync, WorldSyncToProxy,
};
use std::mem::MaybeUninit;

mod change_tracker;

pub struct WorldSync {
    pub particle_world_state: MaybeUninit<ParticleWorldState>,
    pub world_num: u8,
    change_tracker: ChangeTracker,
    initialized: bool,
}

unsafe impl Sync for WorldSync {}
unsafe impl Send for WorldSync {}

impl Default for WorldSync {
    fn default() -> Self {
        Self {
            particle_world_state: MaybeUninit::uninit(),
            world_num: 0,
            change_tracker: ChangeTracker::new(),
            initialized: false,
        }
    }
}

impl Module for WorldSync {
    fn on_world_init(&mut self, _ctx: &mut ModuleCtx) -> eyre::Result<()> {
        self.particle_world_state = MaybeUninit::new(ParticleWorldState::new()?);
        self.initialized = true;
        Ok(())
    }
    fn on_world_update(&mut self, ctx: &mut ModuleCtx) -> eyre::Result<()> {
        if !self.initialized {
            return Ok(());
        }

        let should_update = self
            .change_tracker
            .update(Globals::default().game_global().m_grid_world);

        let Some(ent) = ctx.player_map.get_by_left(&my_peer_id()) else {
            return Ok(());
        };
        let Some(ent) = ctx.globals.entity_manager.get_entity(ent.0.get() as usize) else {
            return Ok(());
        };
        let (x, y) = (ent.transform.pos.x, ent.transform.pos.y);
        let tracked_radius = 2;
        let tracked_square = tracked_radius * 2 + 1;
        let tracked_chunks = (0..tracked_square * tracked_square)
            .into_par_iter()
            .map(|i| {
                let dx = i % tracked_square;
                let dy = i / tracked_square;
                let cx = (x as i32).div_euclid(CHUNK_SIZE as i32) - tracked_radius + dx;
                let cy = (y as i32).div_euclid(CHUNK_SIZE as i32) - tracked_radius + dy;
                ChunkCoord(cx, cy)
            })
            .collect::<Vec<_>>();

        let updates = should_update
            .into_iter()
            .filter_map(|chunk_pos| {
                // Check is any pixel scenes are still being loaded
                let ix = chunk_pos.0 * CHUNK_SIZE as i32;
                let iy = chunk_pos.1 * CHUNK_SIZE as i32;
                let extra_margin = (CHUNK_SIZE) as i32;
                for pixel_scene in ctx.globals.game_global.m_game_world.pixel_scenes.iter() {
                    if pixel_scene.width * pixel_scene.height > 0
                        && pixel_scene.x <= ix
                        && ix <= pixel_scene.x + pixel_scene.width + extra_margin
                        && pixel_scene.y <= iy
                        && iy <= pixel_scene.y + pixel_scene.height + extra_margin
                    {
                        return None;
                    }
                }

                let mut update = NoitaWorldUpdate {
                    coord: chunk_pos,
                    pixels: std::array::from_fn(|_| Pixel::default()),
                };
                if unsafe {
                    self.particle_world_state
                        .assume_init_ref()
                        .encode_world(&mut update)
                }
                .is_ok()
                {
                    Some(update)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let msg = NoitaOutbound::WorldSyncToProxy(WorldSyncToProxy::Updates(updates));
        ctx.net.send(&msg)?;
        let Vec2 { x: cx, y: cy } = ctx.globals.game_global.m_game_world.camera_center();

        let ix = x as i32;
        let iy = y as i32;
        let msg = NoitaOutbound::WorldSyncToProxy(WorldSyncToProxy::End(
            Some((
                ix.div_euclid(CHUNK_SIZE as i32),
                iy.div_euclid(CHUNK_SIZE as i32),
                cx.div_euclid(CHUNK_SIZE as f32) as i32,
                cy.div_euclid(CHUNK_SIZE as f32) as i32,
                false,
            )),
            1,
            self.world_num,
            tracked_chunks,
        ));
        ctx.net.send(&msg)?;
        Ok(())
    }
}
impl WorldSync {
    pub fn handle_remote(&mut self, msg: ProxyToWorldSync) -> eyre::Result<()> {
        match msg {
            ProxyToWorldSync::Updates(updates) => {
                // TODO should check that updates don't touch the same chunk
                updates.into_iter().for_each(|chunk| unsafe {
                    let _ = self
                        .particle_world_state
                        .assume_init_ref()
                        .decode_world(chunk);
                });
            }
        }
        Ok(())
    }
}
pub const SCALE: isize = (512 / CHUNK_SIZE as isize).ilog2() as isize;
#[allow(unused)]
trait WorldData {
    unsafe fn encode_world(&self, chunk: &mut NoitaWorldUpdate) -> eyre::Result<()>;
    unsafe fn decode_world(&self, chunk: NoitaWorldUpdate) -> eyre::Result<()>;
}
impl WorldData for ParticleWorldState {
    unsafe fn encode_world(&self, chunk: &mut NoitaWorldUpdate) -> eyre::Result<()> {
        let ChunkCoord(cx, cy) = chunk.coord;
        let (cx, cy) = (cx as isize, cy as isize);
        let chunk = &mut chunk.pixels;
        let Some(pixel_array) = unsafe { self.world_ptr.as_mut() }
            .wrap_err("no world")?
            .chunk_map
            .get(cx >> SCALE, cy >> SCALE)
        else {
            return Err(eyre!("chunk not loaded"));
        };
        let mut chunk_iter = chunk.iter_mut();
        let (shift_x, shift_y) = self.get_shift::<CHUNK_SIZE>(cx, cy);
        for j in shift_y..shift_y + CHUNK_SIZE as isize {
            for i in shift_x..shift_x + CHUNK_SIZE as isize {
                *chunk_iter.next().unwrap() = pixel_array.get_pixel(i, j);
            }
        }

        Ok(())
    }
    unsafe fn decode_world(&self, chunk: NoitaWorldUpdate) -> eyre::Result<()> {
        let chunk_coord = chunk.coord;
        let (cx, cy) = (chunk_coord.0 as isize, chunk_coord.1 as isize);
        let Some(pixel_array) = unsafe { self.world_ptr.as_mut() }
            .wrap_err("no world")?
            .chunk_map
            .get_mut(cx >> SCALE, cy >> SCALE)
        else {
            return Err(eyre!("chunk not loaded"));
        };
        let (shift_x, shift_y) = self.get_shift::<CHUNK_SIZE>(cx, cy);
        let start_x = cx * CHUNK_SIZE as isize;
        let start_y = cy * CHUNK_SIZE as isize;
        for (i, pixel) in chunk.pixels.into_iter().enumerate() {
            let x = (i % CHUNK_SIZE) as isize;
            let y = (i / CHUNK_SIZE) as isize;

            let cell = pixel_array.get_mut_raw(shift_x + x, shift_y + y);

            if let Some(cell) = unsafe { cell.as_ref() } {
                // Don't touch box2d stuff.
                if cell.material.cell_type == CellType::Solid {
                    continue;
                }
                // No point replacing cells with themselves.
                if cell.material.material_type == pixel.mat() as isize {
                    continue;
                }
            }

            let xs = start_x + x;
            let ys = start_y + y;
            // Drop first
            pixel_array.remove_pixel(shift_x + x, shift_y + y);

            let cell = pixel_array.get_mut_raw(shift_x + x, shift_y + y);
            if pixel.is_air() {
                *cell = Ptr::null();
            } else {
                let Some(mat) = self.material_list.get_static(pixel.mat() as usize) else {
                    return Err(eyre!("mat does not exist"));
                };
                match mat.cell_type {
                    CellType::None => {}
                    CellType::Liquid => {
                        let liquid = unsafe {
                            LiquidCell::create(
                                mat,
                                self.cell_vtables.liquid(),
                                self.world_ptr,
                                xs,
                                ys,
                            )
                        };
                        *cell = Ptr::place_new(liquid).cast();
                    }
                    CellType::Gas => {
                        let mut gas = unsafe {
                            GasCell::create(mat, self.cell_vtables.gas(), self.world_ptr)
                        };
                        gas.x = xs;
                        gas.y = ys;
                        *cell = Ptr::place_new(gas).cast();
                    }
                    CellType::Solid => {}
                    CellType::Fire => {
                        let mut fire = unsafe {
                            FireCell::create(mat, self.cell_vtables.fire(), self.world_ptr)
                        };
                        fire.x = xs;
                        fire.y = ys;
                        *cell = Ptr::place_new(fire).cast();
                    }
                }
            }
        }
        Ok(())
    }
}
#[test]
pub fn test_world() {
    use noita_api::heap::{self};
    use std::ptr;
    let vtable = GridWorldThreadedVTable::default();
    let mut threaded = GridWorldThreaded {
        grid_world_threaded_vtable: unsafe { std::mem::transmute::<&_, &'static _>(&vtable) },
        unknown: [0; 287],
        update_region: Default::default(),
    };
    let mut chunks: [*mut Chunk; 512 * 512] = [ptr::null_mut(); 512 * 512];
    let chunk_map = ChunkMap {
        len: 0,
        unknown: 0,
        chunk_array: unsafe { std::mem::transmute::<&mut _, &'static mut _>(&mut chunks) },
        chunk_count: 0,
        min_chunk: Default::default(),
        max_chunk: Default::default(),
        min_pixel: Default::default(),
        max_pixel: Default::default(),
    };
    let mut grid_world = GridWorld {
        vtable: &GridWorldVTable {
            unknown: [ptr::null(); 3],
            get_chunk_map: ptr::null(),
            unknownmagic: ptr::null(),
            unknown2: [ptr::null(); 30],
            remove_vegetation: None,
        },
        rng: 0,
        unk: [0; 270],
        biome_modifiers: BiomeModifiers {
            vftable: &BiomeModifiersVFTable {},
            unk: [0; 6],
        },
        unk2: [0; 15],
        cam_pos: Default::default(),
        cam_dimen: Default::default(),
        unknown: [0; 6],
        unk_cam: Default::default(),
        unk2_cam: Default::default(),
        unkown3: 0,
        cam: Default::default(),
        unkown2: 0,
        unk_counter: 0,
        world_update_count: 0,
        chunk_map,
        unknown2: [0; 40],
        m_thread_impl: unsafe { std::mem::transmute::<&mut _, &'static mut _>(&mut threaded) },
    };
    let mut pws = ParticleWorldState {
        world_ptr: &mut grid_world,
        material_list: StdVec::new(),
        cell_vtables: CellVTables(
            [CellVTable {
                none: &NoneCellVTable {
                    unknown: [ptr::null(); 41],
                },
            }; 5],
        ),
    };
    for i in 0..256 {
        let mut celldata = CellData::default();
        celldata.material_type = i;
        pws.material_list.push(celldata);
    }
    let mut list = [0; 512 * 512];
    {
        let mut data: [*mut Cell; 512 * 512] = [ptr::null_mut(); 512 * 512];
        for (i, d) in data.iter_mut().enumerate() {
            let mut celldata = CellData::default();
            celldata.material_type = rand::random::<u8>() as isize;
            list[i] = celldata.material_type;
            let cell = Cell::create(
                heap::place_new_ref(celldata),
                CellVTable {
                    none: &NoneCellVTable {
                        unknown: [ptr::null_mut(); 41],
                    },
                },
            );
            *d = heap::place_new(cell);
        }
        let chunk = Chunk {
            data: unsafe { std::mem::transmute::<&mut _, &'static mut _>(&mut data) },
        };
        unsafe { pws.world_ptr.as_mut() }
            .unwrap()
            .chunk_map
            .insert(0, 0, chunk);
    }
    {
        let mut data: [*mut Cell; 512 * 512] = [ptr::null_mut(); 512 * 512];
        for d in data.iter_mut() {
            let celldata = CellData::default();
            let cell = Cell::create(
                heap::place_new_ref(celldata),
                CellVTable {
                    none: &NoneCellVTable {
                        unknown: [ptr::null_mut(); 41],
                    },
                },
            );
            *d = heap::place_new_ref(cell);
        }
        let chunk = Chunk {
            data: unsafe { std::mem::transmute::<&mut _, &'static mut _>(&mut data) },
        };
        unsafe { pws.world_ptr.as_mut() }
            .unwrap()
            .chunk_map
            .insert(1, 1, chunk);
    }
    let mut upd = NoitaWorldUpdate {
        coord: ChunkCoord(5, 5),
        pixels: [Pixel::default(); CHUNK_SIZE * CHUNK_SIZE],
    };
    unsafe {
        assert!(pws.encode_world(&mut upd).is_ok());
    }
    assert_eq!(
        upd.pixels[0..128]
            .iter()
            .map(|a| a.mat())
            .collect::<Vec<_>>(),
        vec![0; 128]
    );
    let tmr = std::time::Instant::now();
    upd.coord = ChunkCoord(0, 0);
    unsafe {
        assert!(pws.encode_world(&mut upd).is_ok());
    }
    println!("{}", tmr.elapsed().as_nanos());
    assert_eq!(
        upd.pixels[0..128]
            .iter()
            .map(|a| a.mat())
            .collect::<Vec<_>>(),
        list[0..128].iter().map(|a| *a as u16).collect::<Vec<_>>()
    );
    let tmr = std::time::Instant::now();
    upd.coord = ChunkCoord(5, 5);
    unsafe {
        assert!(pws.decode_world(upd.clone()).is_ok());
    }
    println!("{}", tmr.elapsed().as_nanos());
    upd.coord = ChunkCoord(0, 0);
    unsafe {
        assert!(pws.encode_world(&mut upd).is_ok());
    }
    assert_eq!(
        upd.pixels[0..128]
            .iter()
            .map(|a| a.mat())
            .collect::<Vec<_>>(),
        list[0..128].iter().map(|a| *a as u16).collect::<Vec<_>>()
    );
}
