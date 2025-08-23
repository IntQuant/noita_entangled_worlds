use crate::modules::{Module, ModuleCtx};
use crate::{WorldSync, my_peer_id};
use eyre::{ContextCompat, eyre};
use noita_api::noita::types::{CellType, FireCell, GasCell, LiquidCell};
use noita_api::noita::world::ParticleWorldState;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use shared::NoitaOutbound;
use shared::world_sync::{
    CHUNK_SIZE, ChunkCoord, CompactPixel, NoitaWorldUpdate, ProxyToWorldSync, WorldSyncToProxy,
};
use std::mem::MaybeUninit;
use std::ptr;
impl Module for WorldSync {
    fn on_world_init(&mut self, _ctx: &mut ModuleCtx) -> eyre::Result<()> {
        self.particle_world_state = MaybeUninit::new(ParticleWorldState::new()?);
        Ok(())
    }
    fn on_world_update(&mut self, ctx: &mut ModuleCtx) -> eyre::Result<()> {
        let Some(ent) = ctx.player_map.get_by_left(&my_peer_id()) else {
            return Ok(());
        };
        let Some(ent) = ctx.globals.entity_manager.get_entity(ent.0.get() as usize) else {
            return Ok(());
        };
        let (x, y) = (ent.transform.pos.x, ent.transform.pos.y);
        let updates = (0..9)
            .into_par_iter()
            .map(|i| {
                let dx = i % 3;
                let dy = i / 3;
                let cx = x as i32 / CHUNK_SIZE as i32 - 1 + dx;
                let cy = y as i32 / CHUNK_SIZE as i32 - 1 + dy;
                let mut update = NoitaWorldUpdate {
                    coord: ChunkCoord(cx, cy),
                    pixels: std::array::from_fn(|_| None),
                };
                if unsafe {
                    self.particle_world_state
                        .assume_init_ref()
                        .encode_world(update.coord, &mut update.pixels)
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
        Ok(())
    }
}
impl WorldSync {
    pub fn handle_remote(&mut self, msg: ProxyToWorldSync) -> eyre::Result<()> {
        match msg {
            ProxyToWorldSync::Updates(updates) => {
                updates.into_par_iter().for_each(|chunk| unsafe {
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
    unsafe fn encode_world(
        &self,
        coord: ChunkCoord,
        chunk: &mut [Option<CompactPixel>; CHUNK_SIZE * CHUNK_SIZE],
    ) -> eyre::Result<()>;
    unsafe fn decode_world(&self, chunk: NoitaWorldUpdate) -> eyre::Result<()>;
}
impl WorldData for ParticleWorldState {
    unsafe fn encode_world(
        &self,
        coord: ChunkCoord,
        chunk: &mut [Option<CompactPixel>; CHUNK_SIZE * CHUNK_SIZE],
    ) -> eyre::Result<()> {
        let (cx, cy) = (coord.0 as isize, coord.1 as isize);
        let Some(pixel_array) = unsafe { self.world_ptr.as_mut() }
            .wrap_err("no world")?
            .chunk_map
            .get(cx >> SCALE, cy >> SCALE)
        else {
            return Err(eyre!("chunk not loaded"));
        };
        let (shift_x, shift_y) = self.get_shift::<CHUNK_SIZE>(cx, cy);
        for ((i, j), p) in (shift_x..shift_x + CHUNK_SIZE as isize)
            .flat_map(|i| (shift_y..shift_y + CHUNK_SIZE as isize).map(move |j| (i, j)))
            .zip(chunk.iter_mut())
        {
            *p = pixel_array.get_compact_pixel(i, j);
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
        for (i, pixel) in chunk.pixels.iter().enumerate() {
            let x = (i % CHUNK_SIZE) as isize;
            let y = (i / CHUNK_SIZE) as isize;
            let cell = pixel_array.get_mut_raw(shift_x + x, shift_y + y);
            let xs = start_x + x;
            let ys = start_y + y;
            let Some(pixel) = pixel else {
                *cell = ptr::null_mut();
                continue;
            };
            let mat = self
                .material_list
                .get_static(pixel.material() as usize)
                .unwrap();
            match mat.cell_type {
                CellType::None => {
                    *cell = ptr::null_mut();
                }
                CellType::Liquid => {
                    let liquid = Box::leak(Box::new(unsafe {
                        LiquidCell::create(mat, self.cell_vtables.liquid(), self.world_ptr)
                    }));
                    liquid.x = xs;
                    liquid.y = ys;
                    *cell = (liquid as *mut LiquidCell).cast();
                }
                CellType::Gas => {
                    let gas = Box::leak(Box::new(unsafe {
                        GasCell::create(mat, self.cell_vtables.gas(), self.world_ptr)
                    }));
                    gas.x = xs;
                    gas.y = ys;
                    *cell = (gas as *mut GasCell).cast();
                }
                CellType::Solid => {}
                CellType::Fire => {
                    let fire = Box::leak(Box::new(unsafe {
                        FireCell::create(mat, self.cell_vtables.fire(), self.world_ptr)
                    }));
                    fire.x = xs;
                    fire.y = ys;
                    *cell = (fire as *mut FireCell).cast();
                }
            }
        }
        Ok(())
    }
}
