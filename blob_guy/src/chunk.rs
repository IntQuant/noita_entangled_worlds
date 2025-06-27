use crate::CHUNK_SIZE;
pub struct Chunk {
    pub pixels: [u16; CHUNK_SIZE * CHUNK_SIZE],
    pub is_blob: [bool; CHUNK_SIZE * CHUNK_SIZE],
    pub is_liquid: [bool; CHUNK_SIZE * CHUNK_SIZE],
    pub is_solid: [bool; CHUNK_SIZE * CHUNK_SIZE],
}
impl Default for Chunk {
    fn default() -> Self {
        Self {
            pixels: [0; CHUNK_SIZE * CHUNK_SIZE],
            is_blob: [false; CHUNK_SIZE * CHUNK_SIZE],
            is_liquid: [false; CHUNK_SIZE * CHUNK_SIZE],
            is_solid: [false; CHUNK_SIZE * CHUNK_SIZE],
        }
    }
}
#[derive(Eq, Hash, PartialEq)]
pub struct ChunkPos {
    x: i32,
    y: i32,
}
impl ChunkPos {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x: x.div_euclid(CHUNK_SIZE as i32),
            y: y.div_euclid(CHUNK_SIZE as i32),
        }
    }
}
