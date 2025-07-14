use std::num::NonZeroU16;

use super::{ChunkData, encoding::PixelRunner};
use bitcode::{Decode, Encode};
use crossbeam::atomic::AtomicCell;
use shared::world_sync::{CHUNK_SIZE, RawPixel};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Encode, Decode)]
pub enum PixelFlags {
    /// Actual material isn't known yet.
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
            (material << 1) | flag_bit
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
pub struct CompactPixel(pub NonZeroU16);

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
    pixels: [u16; CHUNK_SQUARE],
    changed: Changed<bool, CHUNK_SQUARE>,
    any_changed: bool,
    crc: AtomicCell<Option<u64>>,
}

struct Changed<T: Default, const N: usize>([T; N]);
#[cfg(test)]
impl Changed<u128, CHUNK_SIZE> {
    fn get(&self, n: usize) -> bool {
        self.0[n / CHUNK_SIZE] & (1 << (n % CHUNK_SIZE)) != 0
    }
    fn set(&mut self, n: usize) {
        self.0[n / CHUNK_SIZE] |= 1 << (n % CHUNK_SIZE)
    }
}
#[cfg(test)]
const _: () = assert!(u128::BITS as usize == CHUNK_SIZE);
const CHUNK_SQUARE: usize = CHUNK_SIZE * CHUNK_SIZE;
impl Changed<bool, CHUNK_SQUARE> {
    fn get(&self, n: usize) -> bool {
        self.0[n]
    }
    fn set(&mut self, n: usize) {
        self.0[n] = true
    }
}
#[test]
fn test_changed() {
    let tmr = std::time::Instant::now();
    for _ in 0..8192 {
        let mut chunk = Changed([0; CHUNK_SIZE]);
        for i in 0..CHUNK_SQUARE {
            std::hint::black_box(chunk.get(i));
            chunk.set(CHUNK_SQUARE - i - 1);
        }
        std::hint::black_box(chunk);
    }
    println!("u128 {}", tmr.elapsed().as_nanos());
    let tmr = std::time::Instant::now();
    for _ in 0..8192 {
        let mut chunk = Changed([false; CHUNK_SQUARE]);
        for i in 0..CHUNK_SQUARE {
            std::hint::black_box(chunk.get(i));
            chunk.set(CHUNK_SQUARE - i - 1);
        }
        std::hint::black_box(chunk);
    }
    println!("bool {}", tmr.elapsed().as_nanos())
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            pixels: [4095; CHUNK_SQUARE],
            changed: Changed([false; CHUNK_SQUARE]),
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
        let px = pixel.to_compact().raw();
        if self.pixels[offset] != px {
            self.pixels[offset] = px;
            self.mark_changed(offset);
        }
    }

    pub fn set_compact_pixel(&mut self, offset: usize, pixel: CompactPixel) {
        let px = pixel.raw();
        if self.pixels[offset] != px {
            self.pixels[offset] = px;
            self.mark_changed(offset);
        }
    }
    pub fn changed(&self, offset: usize) -> bool {
        self.changed.get(offset)
    }

    pub fn mark_changed(&mut self, offset: usize) {
        self.changed.set(offset);
        self.any_changed = true;
        self.crc.store(None);
    }

    pub fn clear_changed(&mut self) {
        self.changed = Changed([false; CHUNK_SQUARE]);
        self.any_changed = false;
    }

    pub fn to_chunk_data(&self) -> ChunkData {
        let mut runner = PixelRunner::new();
        for i in 0..CHUNK_SQUARE {
            runner.put_pixel(self.compact_pixel(i))
        }
        let runs = runner.build();
        ChunkData { runs }
    }
}
