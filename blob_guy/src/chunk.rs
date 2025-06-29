use crate::CHUNK_SIZE;
use std::ops::{Index, IndexMut};
#[derive(Debug)]
pub struct Chunk(pub [CellType; CHUNK_SIZE * CHUNK_SIZE]);
#[derive(Default, Copy, Clone, Debug)]
pub enum CellType {
    #[default]
    Unknown,
    Solid,
    Liquid,
    Blob,
    Remove,
    Ignore,
}
impl Default for Chunk {
    fn default() -> Self {
        Self([CellType::Unknown; CHUNK_SIZE * CHUNK_SIZE])
    }
}
impl Index<usize> for Chunk {
    type Output = CellType;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
impl IndexMut<usize> for Chunk {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
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
