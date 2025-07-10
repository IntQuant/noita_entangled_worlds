use crate::blob_guy::OFFSET;
use crate::{CHUNK_AMOUNT, CHUNK_SIZE};
use noita_api::noita::types;
use noita_api::noita::world::ParticleWorldState;
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
#[allow(clippy::result_unit_err)]
pub trait Chunks {
    ///# Safety
    unsafe fn encode_area(
        &self,
        x: isize,
        y: isize,
        chunk: &mut Chunk,
        blob: u16,
    ) -> Result<(), ()>;
    ///# Safety
    unsafe fn decode_area(
        &mut self,
        x: isize,
        y: isize,
        chunk: &Chunk,
        blob: u16,
    ) -> Result<(), ()>;
}
const SCALE: isize = (512 / CHUNK_SIZE as isize).ilog2() as isize;
impl Chunks for ParticleWorldState {
    ///# Safety
    unsafe fn encode_area(
        &self,
        x: isize,
        y: isize,
        chunk: &mut Chunk,
        blob: u16,
    ) -> Result<(), ()> {
        let (shift_x, shift_y, pixel_array) = self.set_chunk::<CHUNK_SIZE, SCALE>(x, y)?;
        let pixel_array = unsafe { pixel_array.as_ref() }.unwrap();
        let mut modified = false;
        for ((i, j), pixel) in (0..CHUNK_SIZE as isize)
            .flat_map(|i| (0..CHUNK_SIZE as isize).map(move |j| (i, j)))
            .zip(chunk.iter_mut())
        {
            *pixel = if let Some(cell) = self.get_cell_raw(shift_x + i, shift_y + j, pixel_array) {
                match cell.material.cell_type {
                    types::CellType::Liquid => {
                        if cell.material.material_type as u16 == blob {
                            modified = true;
                            CellType::Remove
                        } else {
                            let cell: &types::LiquidCell = unsafe { cell.get_liquid() };
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
        x: isize,
        y: isize,
        chunk: &Chunk,
        blob: u16,
    ) -> Result<(), ()> {
        if !chunk.modified {
            return Ok(());
        }
        let (shift_x, shift_y, pixel_array) = self.set_chunk::<CHUNK_SIZE, SCALE>(x, y)?;
        let pixel_array = unsafe { pixel_array.as_mut() }.unwrap();
        let x = x * CHUNK_SIZE as isize;
        let y = y * CHUNK_SIZE as isize;
        macro_rules! get_cell {
            ($x:expr, $y:expr, $pixel_array:expr) => {{
                let index = ($y << 9) | $x;
                &mut $pixel_array[index as usize].0
            }};
        }
        for ((i, j), pixel) in (0..CHUNK_SIZE as isize)
            .flat_map(|i| (0..CHUNK_SIZE as isize).map(move |j| (i, j)))
            .zip(chunk.iter())
        {
            match pixel {
                CellType::Blob => {
                    let world_x = x + i;
                    let world_y = y + j;
                    let cell = get_cell!(shift_x + i, shift_y + j, pixel_array);
                    if !(*cell).is_null() {
                        self.remove_cell(*cell, world_x, world_y);
                    }
                    let src = self.create_cell(world_x, world_y, blob);
                    if !src.is_null()
                        && let Some(liquid) = unsafe { src.cast::<types::LiquidCell>().as_mut() }
                    {
                        liquid.is_static = true;
                    }
                    *cell = src;
                }
                CellType::Remove => {
                    let world_x = x + i;
                    let world_y = y + j;
                    std::thread::sleep(std::time::Duration::from_nanos(0));
                    let cell = get_cell!(shift_x + i, shift_y + j, pixel_array);
                    if !(*cell).is_null() {
                        self.remove_cell(*cell, world_x, world_y);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
