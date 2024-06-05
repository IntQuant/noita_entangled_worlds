use bitcode::{Decode, Encode};

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
}

pub struct Chunk {
    pixels: [Pixel; CHUNK_SIZE * CHUNK_SIZE],
    changed: [bool; CHUNK_SIZE * CHUNK_SIZE],
    any_changed: bool,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            pixels: [Pixel::default(); CHUNK_SIZE * CHUNK_SIZE],
            changed: [false; CHUNK_SIZE * CHUNK_SIZE],
            any_changed: false,
        }
    }
}

/// Chunk of pixels. Stores pixels and tracks if they were changed.
impl Chunk {
    pub fn pixel(&self, offset: usize) -> Pixel {
        self.pixels[offset]
    }

    pub fn set_pixel(&mut self, offset: usize, pixel: Pixel) {
        self.pixels[offset] = pixel;
        self.mark_changed(offset);
    }

    pub fn changed(&self, offset: usize) -> bool {
        self.changed[offset]
    }

    pub fn mark_changed(&mut self, offset: usize) {
        self.changed[offset] = true;
        self.any_changed = true;
    }

    pub fn clear_changed(&mut self) {
        self.changed = [false; CHUNK_SIZE * CHUNK_SIZE];
        self.any_changed = false;
    }
}
