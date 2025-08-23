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
    pub pixels: [Option<CompactPixel>; CHUNK_SIZE * CHUNK_SIZE],
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Encode, Decode)]
pub enum PixelFlags {
    /// Actual material isn't known yet.
    #[default]
    Unknown = 0,
    Normal = 32768,
    Abnormal = 16384,
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
        let raw = compact.raw();
        let material = compact.material();
        let flags = compact.flags();
        if raw == CompactPixel::UNKNOWN_RAW {
            RawPixel {
                flags: PixelFlags::Unknown,
                material: 0,
            }
        } else {
            RawPixel { flags, material }
        }
    }
    pub fn from_opt_compact(compact: Option<CompactPixel>) -> Self {
        if let Some(pixel) = compact {
            Self::from_compact(pixel)
        } else {
            RawPixel {
                material: 0,
                flags: PixelFlags::Normal,
            }
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
    pub fn from_material(val: u16) -> Option<Self> {
        if val == 0 {
            None
        } else {
            let val = (val + 1) & 2047;
            let val = val << 1;
            Some(CompactPixel(NonZeroU16::new(val).unwrap()))
        }
    }
    pub fn raw(self) -> u16 {
        u16::from(self.0)
    }
    pub fn material(self) -> u16 {
        (self.raw() >> 1) - 1
    }
    pub fn flags(self) -> PixelFlags {
        if self.raw() & 1 == 1 {
            PixelFlags::Abnormal
        } else {
            PixelFlags::Normal
        }
    }
}

impl Default for CompactPixel {
    fn default() -> Self {
        Self(NonZeroU16::new(CompactPixel::UNKNOWN_RAW).unwrap())
    }
}

#[derive(Debug, Encode, Decode, Clone)]
pub enum WorldSyncToProxy {
    Updates(Vec<Option<NoitaWorldUpdate>>),
    End(Option<(i32, i32, i32, i32, bool)>, u8, u8),
}
#[derive(Debug, Encode, Decode, Clone)]
pub enum ProxyToWorldSync {
    Updates(Vec<NoitaWorldUpdate>),
}
