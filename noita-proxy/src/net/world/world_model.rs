use std::sync::Arc;

use bitcode::{Decode, Encode};
use chunk::Chunk;
use encoding::PixelRunner;
use rustc_hash::{FxHashMap, FxHashSet};
use shared::world_sync::{CHUNK_SIZE, ChunkCoord, NoitaWorldUpdate, Pixel, PixelRun};
use tracing::info;
pub(crate) mod chunk;
pub mod encoding;

#[derive(Default)]
pub(crate) struct WorldModel {
    chunks: FxHashMap<ChunkCoord, Chunk>,
    /// Tracks chunks which we written to.
    /// This includes any write, not just those that actually changed at least one pixel.
    updated_chunks: FxHashSet<ChunkCoord>,
}

/// Contains full info abount a chunk, RLE encoded.
/// Kinda close to ChunkDelta, but doesn't assume we know anything about the chunk.
#[derive(Debug, Encode, Decode, Clone)]
pub(crate) struct ChunkData {
    pub runs: Vec<PixelRun<Pixel>>,
}

/// Contains a diff, only pixels that were updated, for a given chunk.
#[derive(Debug, Encode, Decode, Clone)]
pub(crate) struct ChunkDelta {
    pub chunk_coord: ChunkCoord,
    runs: Arc<Vec<PixelRun<Option<Pixel>>>>,
}

impl ChunkData {
    /*pub(crate) fn make_random() -> Self {
        let mut runner = PixelRunner::new();
        for i in 0..CHUNK_SIZE * CHUNK_SIZE {
            runner.put_pixel(
                Pixel {
                    flags: PixelFlags::Normal,
                    material: (i as u16) % 512,
                }
                .to_compact(),
            )
        }
        let runs = runner.build();
        ChunkData { runs }
    }*/

    #[cfg(test)]
    pub(crate) fn new(mat: u16) -> Self {
        let mut runner = PixelRunner::new();
        for _ in 0..CHUNK_SIZE * CHUNK_SIZE {
            runner.put_pixel(Pixel::new(mat, shared::world_sync::PixelFlags::Normal))
        }
        let runs = runner.build();
        ChunkData { runs }
    }

    pub(crate) fn apply_to_chunk(&self, chunk: &mut Chunk) {
        let nil = Pixel::NIL;
        let mut offset = 0;
        for run in &self.runs {
            let pixel = run.data;
            if pixel != nil {
                for _ in 0..run.length {
                    chunk.set_pixel(offset, pixel);
                    offset += 1;
                }
            } else {
                offset += run.length as usize
            }
        }
    }
    pub(crate) fn apply_delta(&mut self, delta: ChunkData) {
        let nil = Pixel::NIL;
        let mut chunk = Chunk::default();
        self.apply_to_chunk(&mut chunk);
        let mut offset = 0;
        for run in delta.runs.iter() {
            if run.data != nil {
                for _ in 0..run.length {
                    chunk.set_pixel(offset, run.data);
                    offset += 1;
                }
            } else {
                offset += run.length as usize
            }
        }
        *self = chunk.to_chunk_data()
    }
}

impl WorldModel {
    fn get_chunk_offset(x: i32, y: i32) -> usize {
        let x = x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let y = y.rem_euclid(CHUNK_SIZE as i32) as usize;
        x + y * CHUNK_SIZE
    }

    /*fn set_pixel(&mut self, x: i32, y: i32, pixel: Pixel) {
        let (chunk_coord, offset) = Self::get_chunk_coords(x, y);
        let chunk = self.chunks.entry(chunk_coord).or_default();
        let current = chunk.pixel(offset);
        if current != pixel {
            chunk.set_pixel(offset, pixel);
        }
        self.updated_chunks.insert(chunk_coord);
    }*/

    /*fn get_pixel(&self, x: i32, y: i32) -> Pixel {
        let (chunk_coord, offset) = Self::get_chunk_coords(x, y);
        self.chunks
            .get(&chunk_coord)
            .map(|chunk| chunk.pixel(offset))
            .unwrap_or_default()
    }*/

    pub fn apply_noita_update(
        &mut self,
        update: NoitaWorldUpdate,
        changed: &mut FxHashSet<ChunkCoord>,
    ) {
        let (start_x, start_y) = (
            update.coord.0 * CHUNK_SIZE as i32,
            update.coord.1 * CHUNK_SIZE as i32,
        );
        let chunk_coord = update.coord;
        let chunk = self.chunks.entry(update.coord).or_default();
        for (i, pixel) in update.pixels.into_iter().enumerate() {
            let x = (i % CHUNK_SIZE) as i32;
            let y = (i / CHUNK_SIZE) as i32;
            let xs = start_x + x;
            let ys = start_y + y;
            let offset = Self::get_chunk_offset(xs, ys);
            if chunk.set_pixel(offset, pixel) {
                self.updated_chunks.insert(chunk_coord);
                changed.remove(&chunk_coord);
            }
        }
    }

    pub fn get_all_noita_updates(&mut self) -> Vec<NoitaWorldUpdate> {
        let mut updates = Vec::new();
        for coord in self.updated_chunks.drain() {
            if let Some(chunk) = self.chunks.get_mut(&coord) {
                updates.push(NoitaWorldUpdate {
                    coord,
                    pixels: chunk.pixels,
                });
            }
        }
        updates
    }

    pub(crate) fn apply_chunk_delta(&mut self, delta: &ChunkDelta) {
        self.updated_chunks.insert(delta.chunk_coord);
        let chunk = self.chunks.entry(delta.chunk_coord).or_default();
        let mut offset = 0;
        for run in delta.runs.iter() {
            if let Some(pixel) = run.data {
                for _ in 0..run.length {
                    chunk.set_pixel(offset, pixel);
                    offset += 1;
                }
            } else {
                offset += run.length as usize
            }
        }
    }

    pub(crate) fn get_chunk_delta(
        &self,
        chunk_coord: ChunkCoord,
        ignore_changed: bool,
    ) -> Option<ChunkDelta> {
        let chunk = self.chunks.get(&chunk_coord)?;
        let mut runner = PixelRunner::new();
        for i in 0..CHUNK_SIZE * CHUNK_SIZE {
            runner.put_pixel((ignore_changed || chunk.changed(i)).then(|| chunk.pixel(i)))
        }
        let runs = runner.build().into();
        Some(ChunkDelta { chunk_coord, runs })
    }

    pub fn reset_change_tracking(&mut self) {
        for chunk_pos in &self.updated_chunks {
            if let Some(chunk) = self.chunks.get_mut(chunk_pos) {
                chunk.clear_changed();
            }
        }
        self.updated_chunks.clear();
    }

    pub fn reset(&mut self) {
        self.chunks.clear();
        self.updated_chunks.clear();
        info!("World model reset");
    }

    pub(crate) fn apply_chunk_data(&mut self, chunk: ChunkCoord, chunk_data: &ChunkData) {
        self.updated_chunks.insert(chunk);
        let chunk = self.chunks.entry(chunk).or_default();
        chunk_data.apply_to_chunk(chunk);
    }

    pub(crate) fn get_chunk_data(&self, chunk: ChunkCoord) -> Option<ChunkData> {
        let chunk = self.chunks.get(&chunk)?;
        Some(chunk.to_chunk_data())
    }

    pub(crate) fn forget_chunk(&mut self, chunk: ChunkCoord) {
        self.chunks.remove(&chunk);
        self.updated_chunks.remove(&chunk);
    }
}
