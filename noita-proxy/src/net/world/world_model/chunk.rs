use std::num::NonZeroU16;

use bitcode::{Decode, Encode};
use crossbeam::atomic::AtomicCell;

use super::{encoding::RawPixel, CHUNK_SIZE};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Encode, Decode)]
pub enum PixelFlags {
    #[default]
    Unknown,
    Normal,
    Fluid,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Encode, Decode)]
pub struct Pixel {
    pub flags: PixelFlags,
    pub material: u16,
}

impl Pixel {
    pub fn to_raw(self) -> RawPixel {
        RawPixel {
            material: if self.flags != PixelFlags::Unknown {
                self.material
            } else {
                u16::MAX
            },
            flags: if self.flags == PixelFlags::Normal {
                0
            } else {
                1
            },
        }
    }
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
            material << 1 | flag_bit
        };
        CompactPixel(NonZeroU16::new(raw).unwrap())
    }
    fn from_compact(compact: CompactPixel) -> Self {
        let raw = u16::from(compact.0);
        let material = (raw >> 1) - 1;
        let flags = if raw & 1 == 1 {
            PixelFlags::Fluid
        } else {
            PixelFlags::Normal
        };
        if raw == CompactPixel::UNKNOWN_RAW {
            Pixel {
                flags: PixelFlags::Unknown,
                material: 0,
            }
        } else {
            Pixel { flags, material }
        }
    }
}

/// An entire pixel packed into 12 bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
#[repr(transparent)]
pub struct CompactPixel(NonZeroU16);

impl CompactPixel {
    const UNKNOWN_RAW: u16 = 4095;
    fn from_raw(val: u16) -> Self {
        CompactPixel(NonZeroU16::new(val).unwrap())
    }
    fn raw(self) -> u16 {
        u16::from(self.0)
    }
}

impl Default for CompactPixel {
    fn default() -> Self {
        Self(NonZeroU16::new(CompactPixel::UNKNOWN_RAW).unwrap())
    }
}

pub struct Chunk {
    pixels: [u16; CHUNK_SIZE * CHUNK_SIZE],
    changed: [bool; CHUNK_SIZE * CHUNK_SIZE],
    any_changed: bool,
    crc: AtomicCell<Option<u64>>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            pixels: [4095; CHUNK_SIZE * CHUNK_SIZE],
            changed: [false; CHUNK_SIZE * CHUNK_SIZE],
            any_changed: false,
            crc: None.into(),
        }
    }
}

/// Chunk of pixels. Stores pixels and tracks if they were changed.
impl Chunk {
    pub fn pixel(&self, offset: usize) -> Pixel {
        Pixel::from_compact(CompactPixel::from_raw(self.pixels[offset]))
    }

    pub fn compact_pixel(&self, offset: usize) -> CompactPixel {
        CompactPixel::from_raw(self.pixels[offset])
    }

    pub fn set_pixel(&mut self, offset: usize, pixel: Pixel) {
        self.pixels[offset] = pixel.to_compact().raw();
        self.mark_changed(offset);
    }

    pub fn set_compact_pixel(&mut self, offset: usize, pixel: CompactPixel) {
        self.pixels[offset] = pixel.raw();
        self.mark_changed(offset);
    }
    pub fn changed(&self, offset: usize) -> bool {
        self.changed[offset]
    }

    pub fn mark_changed(&mut self, offset: usize) {
        self.changed[offset] = true;
        self.any_changed = true;
        self.crc.store(None);
    }

    pub fn clear_changed(&mut self) {
        self.changed = [false; CHUNK_SIZE * CHUNK_SIZE];
        self.any_changed = false;
    }
}
