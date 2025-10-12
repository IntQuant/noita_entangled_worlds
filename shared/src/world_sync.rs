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
    pub pixels: [Pixel; CHUNK_SIZE * CHUNK_SIZE],
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Encode, Decode)]
pub enum PixelFlags {
    /// Actual material isn't known yet.
    Normal = 0,
    Abnormal = 1,
    #[default]
    Unknown = 15,
    //may have at most * = 15
}

/// An entire pixel packed into 12 bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode, Default)]
#[repr(transparent)]
pub struct Pixel(u16);

#[test]
fn test() {
    let p = Pixel::new(0, PixelFlags::Unknown);
    assert_eq!(p.mat(), 0);
    assert_eq!(p.flags(), PixelFlags::Unknown);
    let p = Pixel::new(0, PixelFlags::Normal);
    assert_eq!(p.mat(), 0);
    assert_eq!(p.flags(), PixelFlags::Normal);
    let p = Pixel::new(15, PixelFlags::Unknown);
    assert_eq!(p.mat(), 15);
    assert_eq!(p.flags(), PixelFlags::Unknown);
    let p = Pixel::new(15, PixelFlags::Normal);
    assert_eq!(p.mat(), 15);
    assert_eq!(p.flags(), PixelFlags::Normal);
}

impl Pixel {
    pub const NIL: Pixel = Pixel::new(0, PixelFlags::Abnormal);
    //mat must be less then 13 bits
    pub const fn new(mat: u16, flag: PixelFlags) -> Self {
        Self(mat | ((flag as u16) << 12))
    }
    pub fn mat(self) -> u16 {
        self.0 & 0x0FFF
    }
    pub fn flags(self) -> PixelFlags {
        unsafe { std::mem::transmute((self.0 >> 12) as u8) }
    }

    pub fn is_air(&self) -> bool {
        self.mat() == 0
    }
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
