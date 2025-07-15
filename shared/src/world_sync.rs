use bitcode::{Decode, Encode};
use std::num::NonZeroU16;
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Encode, Decode)]
pub enum PixelFlags {
    /// Actual material isn't known yet.
    #[default]
    Unknown,
    Normal,
    Abnormal,
}

#[derive(Debug, Encode, Decode, PartialEq, Eq, Clone, Copy)]
pub struct RawPixel {
    pub material: u16,
    pub flags: PixelFlags,
}

impl RawPixel {
    pub fn to_compact(self) -> CompactPixel {
        let flag_bit = if self.flags == PixelFlags::Normal {
            0
        } else {
            1
        };
        let material = (self.material + 1) & 2047; // 11 bits for material
        let raw = if self.flags == PixelFlags::Unknown {
            CompactPixel::UNKNOWN_RAW
        } else {
            (material << 1) | flag_bit
        };
        CompactPixel(NonZeroU16::new(raw).unwrap())
    }
    pub fn from_compact(compact: CompactPixel) -> Self {
        let raw = u16::from(compact.0);
        let material = (raw >> 1) - 1;
        let flags = if raw & 1 == 1 {
            PixelFlags::Abnormal
        } else {
            PixelFlags::Normal
        };
        if raw == CompactPixel::UNKNOWN_RAW {
            RawPixel {
                flags: PixelFlags::Unknown,
                material: 0,
            }
        } else {
            RawPixel { flags, material }
        }
    }
}

/// An entire pixel packed into 12 bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
#[repr(transparent)]
pub struct CompactPixel(pub NonZeroU16);

impl CompactPixel {
    const UNKNOWN_RAW: u16 = 4095;
    pub fn from_raw(val: u16) -> Self {
        CompactPixel(NonZeroU16::new(val).unwrap())
    }
    pub fn raw(self) -> u16 {
        u16::from(self.0)
    }
}

impl Default for CompactPixel {
    fn default() -> Self {
        Self(NonZeroU16::new(CompactPixel::UNKNOWN_RAW).unwrap())
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
