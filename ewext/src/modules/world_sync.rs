use crate::WorldSync;
use crate::modules::{Module, ModuleCtx};
use eyre::{ContextCompat, eyre};
use noita_api::noita::types::{CellType, FireCell, GasCell, LiquidCell};
use noita_api::noita::world::ParticleWorldState;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use shared::NoitaOutbound;
use shared::world_sync::{
    CHUNK_SIZE, ChunkCoord, NoitaWorldUpdate, PixelFlags, ProxyToWorldSync, RawPixel,
    WorldSyncToProxy,
};
use std::ptr;
impl Module for WorldSync {
    fn on_world_update(&mut self, ctx: &mut ModuleCtx) -> eyre::Result<()> {
        let update = NoitaWorldUpdate {
            coord: ChunkCoord(0, 0),
            runs: Vec::with_capacity(16384),
        };
        let upd0 = std::array::from_fn(|_| RawPixel {
            material: 0,
            flags: PixelFlags::Unknown,
        });
        let upd1 = std::array::from_fn(|_| RawPixel {
            material: 0,
            flags: PixelFlags::Unknown,
        });
        let upd2 = std::array::from_fn(|_| RawPixel {
            material: 0,
            flags: PixelFlags::Unknown,
        });
        let upd3 = std::array::from_fn(|_| RawPixel {
            material: 0,
            flags: PixelFlags::Unknown,
        });
        let mut arr = [upd0, upd1, upd2, upd3];
        arr.par_iter_mut().try_for_each(|upd| unsafe {
            self.particle_world_state
                .assume_init_ref()
                .encode_world(ChunkCoord(-2, -7), upd)
        })?;
        std::hint::black_box(arr);
        let msg = NoitaOutbound::WorldSyncToProxy(WorldSyncToProxy::Updates(vec![update]));
        ctx.net.send(&msg)?;
        Ok(())
    }
}
impl WorldSync {
    pub fn handle_remote(&mut self, msg: ProxyToWorldSync) -> eyre::Result<()> {
        match msg {
            ProxyToWorldSync::Updates(updates) => {
                for chunk in updates {
                    let time = std::time::Instant::now();
                    unsafe {
                        self.particle_world_state
                            .assume_init_ref()
                            .decode_world(chunk)?
                    }
                    noita_api::print!("de {}", time.elapsed().as_micros());
                }
            }
        }
        Ok(())
    }
}
pub const SCALE: isize = (512 / CHUNK_SIZE as isize).ilog2() as isize;
trait WorldData {
    unsafe fn encode_world(
        &self,
        coord: ChunkCoord,
        chunk: &mut [RawPixel; CHUNK_SIZE * CHUNK_SIZE],
    ) -> eyre::Result<()>;
    unsafe fn decode_world(&self, chunk: NoitaWorldUpdate) -> eyre::Result<()>;
}
impl WorldData for ParticleWorldState {
    unsafe fn encode_world(
        &self,
        coord: ChunkCoord,
        chunk: &mut [RawPixel; CHUNK_SIZE * CHUNK_SIZE],
    ) -> eyre::Result<()> {
        let (cx, cy) = (coord.0 as isize, coord.1 as isize);
        let Some(pixel_array) = unsafe { self.world_ptr.as_mut() }
            .wrap_err("no world")?
            .chunk_map
            .chunk_array
            .get(cx >> SCALE, cy >> SCALE)
        else {
            return Err(eyre!("chunk not loaded"));
        };
        let (shift_x, shift_y) = self.get_shift::<CHUNK_SIZE>(cx, cy);
        for ((i, j), p) in (shift_x..shift_x + CHUNK_SIZE as isize)
            .flat_map(|i| (shift_y..shift_y + CHUNK_SIZE as isize).map(move |j| (i, j)))
            .zip(chunk.iter_mut())
        {
            *p = pixel_array.get_raw_pixel(i, j);
        }
        Ok(())
    }
    unsafe fn decode_world(&self, chunk: NoitaWorldUpdate) -> eyre::Result<()> {
        let chunk_coord = chunk.coord;
        let (cx, cy) = (chunk_coord.0 as isize, chunk_coord.1 as isize);
        let Some(pixel_array) = unsafe { self.world_ptr.as_mut() }
            .wrap_err("no world")?
            .chunk_map
            .chunk_array
            .get_mut(cx >> SCALE, cy >> SCALE)
        else {
            return Err(eyre!("chunk not loaded"));
        };
        let (shift_x, shift_y) = self.get_shift::<CHUNK_SIZE>(cx, cy);
        let start_x = cx * CHUNK_SIZE as isize;
        let start_y = cy * CHUNK_SIZE as isize;
        let mut x = 0;
        let mut y = 0;
        for run in chunk.runs {
            for _ in 0..run.length {
                if let Some(cell) = pixel_array.get_mut(shift_x + x, shift_y + y) {
                    let xs = start_x + x;
                    let ys = start_y + y;
                    let mat = &self.material_list[run.data.material as usize];
                    match mat.cell_type {
                        CellType::None => {
                            cell.0 = ptr::null_mut();
                        }
                        CellType::Liquid => {
                            let liquid = Box::leak(Box::new(unsafe {
                                LiquidCell::create(mat, self.cell_vtables.liquid(), self.world_ptr)
                            }));
                            liquid.x = xs;
                            liquid.y = ys;
                            cell.0 = (liquid as *mut LiquidCell).cast();
                        }
                        CellType::Gas => {
                            let gas = Box::leak(Box::new(unsafe {
                                GasCell::create(mat, self.cell_vtables.gas(), self.world_ptr)
                            }));
                            gas.x = xs;
                            gas.y = ys;
                            cell.0 = (gas as *mut GasCell).cast();
                        }
                        CellType::Solid => {}
                        CellType::Fire => {
                            let fire = Box::leak(Box::new(unsafe {
                                FireCell::create(mat, self.cell_vtables.fire(), self.world_ptr)
                            }));
                            fire.x = xs;
                            fire.y = ys;
                            cell.0 = (fire as *mut FireCell).cast();
                        }
                    }
                }
                if x == CHUNK_SIZE as isize {
                    x = 0;
                    y += 1;
                } else {
                    x += 1;
                }
            }
        }
        Ok(())
    }
}
