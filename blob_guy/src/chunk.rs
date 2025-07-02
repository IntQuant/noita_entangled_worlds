use crate::CHUNK_SIZE;
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
}
