use std::num::NonZeroU16;

use bitcode::{Decode, Encode};

use super::{
    CHUNK_SIZE, ChunkData,
    encoding::{PixelRunner, RawPixel},
};

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
    /// Materials must fit in 11 bits once incremented, so ids at or above this
    /// cannot be represented in the compact form.
    pub const MAX_MATERIAL: u16 = 2046;

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
        // Only 11 bits are available for the material, and the encoding stores
        // `material + 1` so that raw 0 is never produced (CompactPixel is a
        // NonZeroU16). Ids at or above the limit used to wrap: 2047+Normal
        // produced raw 0 and panicked the unwrap, 2046+Fluid collided with
        // UNKNOWN_RAW, and anything >= 2048 silently aliased mod 2048. Encode
        // them as Unknown instead - wrong, but neither a crash nor a silent
        // impersonation of an unrelated material.
        let raw = if self.flags == PixelFlags::Unknown || self.material >= Self::MAX_MATERIAL {
            CompactPixel::UNKNOWN_RAW
        } else {
            ((self.material + 1) << 1) | flag_bit
        };
        CompactPixel(NonZeroU16::new(raw).unwrap())
    }
    fn from_compact(compact: CompactPixel) -> Self {
        let raw = u16::from(compact.0);
        // Must be checked before the arithmetic below: UNKNOWN_RAW is 4095, and
        // `(4095 >> 1) - 1` is fine, but raw values of 0 or 1 would underflow.
        if raw == CompactPixel::UNKNOWN_RAW || raw < 2 {
            return Pixel {
                flags: PixelFlags::Unknown,
                material: 0,
            };
        }
        let material = (raw >> 1) - 1;
        let flags = if raw & 1 == 1 {
            PixelFlags::Fluid
        } else {
            PixelFlags::Normal
        };
        Pixel { flags, material }
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
    // A plain bool array, deliberately, even though a bitset would be 2 KiB
    // instead of 16 KiB. get/set run per pixel, while the size only matters on
    // rehash and clone, and the u128 bitset measures ~6x slower on exactly this
    // access pattern - see test_changed below, and commit 71c934d3 which made
    // this same call the first time.
    changed: Changed<bool, CHUNK_SQUARE>,
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

#[test]
fn compact_pixel_round_trip() {
    // Every representable material, both flag states. This previously panicked
    // on material 2047 (raw 0 -> NonZeroU16::new().unwrap()) and on the
    // `(raw >> 1) - 1` underflow, and silently aliased ids >= 2048.
    for material in 0..=u16::MAX {
        for flags in [PixelFlags::Normal, PixelFlags::Fluid] {
            let px = Pixel { flags, material };
            let round = Pixel::from_compact(px.to_compact());
            if material < Pixel::MAX_MATERIAL {
                assert_eq!(round, px, "material {material} with {flags:?}");
            } else {
                assert_eq!(
                    round.flags,
                    PixelFlags::Unknown,
                    "out-of-range material {material} should degrade to Unknown, not alias"
                );
            }
        }
    }
    let unknown = Pixel {
        flags: PixelFlags::Unknown,
        material: 0,
    };
    assert_eq!(Pixel::from_compact(unknown.to_compact()), unknown);
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            pixels: [4095; CHUNK_SQUARE],
            changed: Changed([false; CHUNK_SQUARE]),
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
        // This used to also store to a `crc: AtomicCell<Option<u64>>`, which is
        // 16 bytes and therefore *not* lock-free - every changed pixel took a
        // lock in crossbeam's process-global seqlock table, contended across the
        // rayon terraforming workers. The field was never read.
        self.changed.set(offset);
    }

    pub fn clear_changed(&mut self) {
        self.changed = Changed([false; CHUNK_SQUARE]);
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
