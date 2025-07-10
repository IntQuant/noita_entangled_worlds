use crate::blob_guy::OFFSET;
use crate::{CHUNK_AMOUNT, CHUNK_SIZE};
use eyre::eyre;
use noita_api::noita::types;
use noita_api::noita::world::ParticleWorldState;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use std::ops::{Index, IndexMut};
use std::slice::{Iter, IterMut};
#[derive(Debug)]
pub struct Chunk {
    pub data: [CellType; CHUNK_SIZE * CHUNK_SIZE],
    pub modified: bool,
}
#[derive(Default, Copy, Clone, Debug)]
pub enum CellType {
    #[default]
    Unknown,
    Solid,
    Liquid,
    Sand,
    Blob,
    Remove,
    Ignore,
    Physics,
    Other,
}
#[derive(Default, Debug, Clone, Copy)]
pub struct Pos {
    pub x: f32,
    pub y: f32,
}
#[derive(Default)]
pub struct Chunks(pub [Chunk; CHUNK_AMOUNT * CHUNK_AMOUNT]);
impl Chunks {
    pub fn read(
        &mut self,
        particle_world_state: &ParticleWorldState,
        blob_guy: u16,
        start: ChunkPos,
    ) -> eyre::Result<()> {
        self.0
            .par_iter_mut()
            .enumerate()
            .try_for_each(|(i, chunk)| unsafe {
                let x = i as isize / CHUNK_AMOUNT as isize + start.x;
                let y = i as isize % CHUNK_AMOUNT as isize + start.y;
                particle_world_state.encode_area(x - OFFSET, y - OFFSET, chunk, blob_guy)
            })
    }
    pub fn paint(
        &mut self,
        particle_world_state: &mut ParticleWorldState,
        blob_guy: u16,
        start: ChunkPos,
    ) {
        self.0.iter().enumerate().for_each(|(i, chunk)| unsafe {
            let x = i as isize / CHUNK_AMOUNT as isize + start.x;
            let y = i as isize % CHUNK_AMOUNT as isize + start.y;
            let _ = particle_world_state.decode_area(x - OFFSET, y - OFFSET, chunk, blob_guy);
        });
    }
}
impl Pos {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn to_chunk(self) -> ChunkPos {
        ChunkPos::new(self.x.floor() as isize, self.y.floor() as isize)
    }
}
impl Default for Chunk {
    fn default() -> Self {
        Self {
            data: [CellType::Unknown; CHUNK_SIZE * CHUNK_SIZE],
            modified: false,
        }
    }
}
impl Index<usize> for Chunk {
    type Output = CellType;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}
impl IndexMut<usize> for Chunk {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}
impl Index<(isize, isize)> for Chunk {
    type Output = CellType;
    fn index(&self, (x, y): (isize, isize)) -> &Self::Output {
        let n = x.rem_euclid(CHUNK_SIZE as isize) as usize * CHUNK_SIZE
            + y.rem_euclid(CHUNK_SIZE as isize) as usize;
        &self.data[n]
    }
}
impl IndexMut<(isize, isize)> for Chunk {
    fn index_mut(&mut self, (x, y): (isize, isize)) -> &mut Self::Output {
        let n = x.rem_euclid(CHUNK_SIZE as isize) as usize * CHUNK_SIZE
            + y.rem_euclid(CHUNK_SIZE as isize) as usize;
        &mut self.data[n]
    }
}
impl Chunk {
    pub fn iter_mut(&mut self) -> IterMut<'_, CellType> {
        self.data.iter_mut()
    }
    pub fn iter(&self) -> Iter<'_, CellType> {
        self.data.iter()
    }
}
#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub struct ChunkPos {
    pub x: isize,
    pub y: isize,
}
impl ChunkPos {
    pub fn new(x: isize, y: isize) -> Self {
        Self {
            x: x.div_euclid(CHUNK_SIZE as isize),
            y: y.div_euclid(CHUNK_SIZE as isize),
        }
    }
    pub fn get_world(self, start: Self) -> isize {
        (self.x - start.x + OFFSET) * CHUNK_AMOUNT as isize + (self.y - start.y + OFFSET)
    }
}
pub trait ChunkOps {
    ///# Safety
    unsafe fn encode_area(
        &self,
        x: isize,
        y: isize,
        chunk: &mut Chunk,
        blob: u16,
    ) -> eyre::Result<()>;
    ///# Safety
    unsafe fn decode_area(
        &mut self,
        x: isize,
        y: isize,
        chunk: &Chunk,
        blob: u16,
    ) -> eyre::Result<()>;
}
const SCALE: isize = (512 / CHUNK_SIZE as isize).ilog2() as isize;
impl ChunkOps for ParticleWorldState {
    ///# Safety
    unsafe fn encode_area(
        &self,
        x: isize,
        y: isize,
        chunk: &mut Chunk,
        blob: u16,
    ) -> eyre::Result<()> {
        let (shift_x, shift_y) = self.get_shift::<CHUNK_SIZE>(x, y);
        let Some(pixel_array) = self
            .world_ptr
            .chunk_map
            .chunk_array
            .get(x >> SCALE, y >> SCALE)
        else {
            return Err(eyre!("chunk not loaded"));
        };
        let mut modified = false;
        for ((i, j), pixel) in (0..CHUNK_SIZE as isize)
            .flat_map(|i| (0..CHUNK_SIZE as isize).map(move |j| (i, j)))
            .zip(chunk.iter_mut())
        {
            *pixel = if let Some(cell) = pixel_array.get(shift_x + i, shift_y + j) {
                match cell.material.cell_type {
                    types::CellType::Liquid => {
                        if cell.material.material_type as u16 == blob {
                            modified = true;
                            CellType::Remove
                        } else {
                            let cell: &types::LiquidCell = cell.get_liquid();
                            if cell.is_static {
                                CellType::Solid
                            } else if cell.cell.material.liquid_sand {
                                CellType::Sand
                            } else {
                                CellType::Liquid
                            }
                        }
                    }
                    types::CellType::Solid => CellType::Physics,
                    types::CellType::Fire | types::CellType::Gas => CellType::Other,
                    _ => CellType::Unknown,
                }
            } else {
                CellType::Unknown
            }
        }
        chunk.modified = modified;
        Ok(())
    }
    ///# Safety
    unsafe fn decode_area(
        &mut self,
        cx: isize,
        cy: isize,
        chunk: &Chunk,
        blob: u16,
    ) -> eyre::Result<()> {
        if !chunk.modified {
            return Ok(());
        }
        let ptr = self.world_ptr as *mut types::GridWorld;
        let (shift_x, shift_y) = self.get_shift::<CHUNK_SIZE>(cx, cy);
        let Some(pixel_array) = self
            .world_ptr
            .chunk_map
            .chunk_array
            .get_mut(cx >> SCALE, cy >> SCALE)
        else {
            return Err(eyre!("chunk not loaded"));
        };
        let x = cx * CHUNK_SIZE as isize;
        let y = cy * CHUNK_SIZE as isize;
        for ((i, j), pixel) in (0..CHUNK_SIZE as isize)
            .flat_map(|i| (0..CHUNK_SIZE as isize).map(move |j| (i, j)))
            .zip(chunk.iter())
        {
            match pixel {
                CellType::Blob => {
                    let world_x = x + i;
                    let world_y = y + j;
                    if let Some(cell) = pixel_array.get_mut(shift_x + i, shift_y + j) {
                        if !cell.0.is_null() {
                            self.remove_ptr.remove_cell(ptr, cell.0, world_x, world_y);
                        }
                        let src = self.construct_ptr.create_cell(
                            ptr,
                            world_x,
                            world_y,
                            &self.material_list[blob as usize],
                        );
                        if !src.is_null()
                            && let Some(liquid) =
                                unsafe { src.cast::<types::LiquidCell>().as_mut() }
                        {
                            liquid.is_static = true;
                        }
                        cell.0 = src;
                    }
                }
                CellType::Remove => {
                    let world_x = x + i;
                    let world_y = y + j;
                    std::thread::sleep(std::time::Duration::from_nanos(0));
                    if let Some(cell) = pixel_array.get_mut(shift_x + i, shift_y + j) {
                        self.remove_ptr.remove_cell(ptr, cell.0, world_x, world_y);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
