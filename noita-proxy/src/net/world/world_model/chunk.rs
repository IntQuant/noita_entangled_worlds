use super::{ChunkData, encoding::PixelRunner};
use shared::world_sync::{CHUNK_SIZE, Pixel};

pub struct Chunk {
    pub pixels: [Pixel; CHUNK_SQUARE],
    changed: Changed<bool, CHUNK_SQUARE>,
    any_changed: bool,
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
            pixels: [Pixel::NIL; CHUNK_SQUARE],
            changed: Changed([false; CHUNK_SQUARE]),
            any_changed: false,
        }
    }
}

/// Chunk of pixels. Stores pixels and tracks if they were changed.
impl Chunk {
    pub fn pixel(&self, offset: usize) -> Pixel {
        self.pixels[offset]
    }

    pub fn set_pixel(&mut self, offset: usize, pixel: Pixel) -> bool {
        if self.pixels[offset] != pixel {
            self.pixels[offset] = pixel;
            self.mark_changed(offset);
            true
        } else {
            false
        }
    }

    pub fn changed(&self, offset: usize) -> bool {
        self.changed.get(offset)
    }

    pub fn mark_changed(&mut self, offset: usize) {
        self.changed.set(offset);
        self.any_changed = true;
    }

    pub fn clear_changed(&mut self) {
        self.changed = Changed([false; CHUNK_SQUARE]);
        self.any_changed = false;
    }

    pub fn to_chunk_data(&self) -> ChunkData {
        let mut runner = PixelRunner::new();
        for i in 0..CHUNK_SQUARE {
            runner.put_pixel(self.pixel(i))
        }
        let runs = runner.build();
        ChunkData { runs }
    }
}
