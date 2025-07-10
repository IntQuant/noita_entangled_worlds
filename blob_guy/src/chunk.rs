use crate::blob_guy::OFFSET;
use crate::{CHUNK_AMOUNT, CHUNK_SIZE};
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
    Blob,
    Remove,
    Ignore,
    Physics,
    Other,
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
