use bitcode::{Decode, Encode};
/// Stores a run of pixels.
/// Not specific to Noita side - length is an actual length
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub struct PixelRun<Pixel> {
    pub length: u16,
    pub data: Pixel,
}

pub const CHUNK_SIZE: usize = 128;

#[derive(Debug, Encode, Decode, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ChunkCoord(pub i32, pub i32);

#[derive(Debug, Encode, Decode, Clone)]
pub struct NoitaWorldUpdate {
    pub coord: ChunkCoord,
    pub runs: Vec<PixelRun<RawPixel>>,
}

#[derive(Debug, Encode, Decode, PartialEq, Eq, Clone, Copy)]
pub struct RawPixel {
    pub material: u16,
    pub flags: u8,
}

#[derive(Debug, Encode, Decode, Clone)]
pub enum WorldSyncToProxy {
    Updates(Vec<NoitaWorldUpdate>),
    End(Option<(i32, i32, i32, i32, bool)>, u8, u8),
}
#[derive(Debug, Encode, Decode, Clone)]
pub enum ProxyToWorldSync {
    Updates(Vec<NoitaWorldUpdate>),
}
