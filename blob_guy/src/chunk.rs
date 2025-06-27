use crate::CHUNK_SIZE;
pub struct Chunk {
    pub pixels: [u16; CHUNK_SIZE * CHUNK_SIZE],
    pub is_blob: [bool; CHUNK_SIZE * CHUNK_SIZE],
    pub is_liquid: [bool; CHUNK_SIZE * CHUNK_SIZE],
    pub is_solid: [bool; CHUNK_SIZE * CHUNK_SIZE],
}
