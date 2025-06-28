use crate::CHUNK_SIZE;
use std::ops::{Index, IndexMut};
pub struct Chunk(pub [CellType; CHUNK_SIZE * CHUNK_SIZE]);
#[derive(Default, Copy, Clone)]
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
#[derive(Eq, Hash, PartialEq)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
}
impl ChunkPos {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x: x.div_euclid(CHUNK_SIZE as i32),
            y: y.div_euclid(CHUNK_SIZE as i32),
        }
    }
}
