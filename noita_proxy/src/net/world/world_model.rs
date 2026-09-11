use std::num::NonZeroU16;
use std::sync::Arc;

use bitcode::{Decode, Encode};
use chunk::{Chunk, CompactPixel, Pixel, PixelFlags};
use encoding::{NoitaWorldUpdate, PixelRun, PixelRunner};
use rustc_hash::{FxHashMap, FxHashSet};
use tracing::{info, warn};

/// Number of pixels in a chunk. Run lengths decoded from the network are
/// clamped against this: `Chunk` stores a fixed `[u16; CHUNK_AREA]`, so a run
/// set that sums past it would index out of bounds and panic the net thread.
const CHUNK_AREA: usize = CHUNK_SIZE * CHUNK_SIZE;

pub(crate) mod chunk;
pub mod encoding;

pub(crate) const CHUNK_SIZE: usize = 128;

#[derive(Debug, Encode, Decode, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ChunkCoord(pub i32, pub i32);

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
    pub runs: Vec<PixelRun<CompactPixel>>,
}

/// Contains a diff, only pixels that were updated, for a given chunk.
#[derive(Debug, Encode, Decode, Clone)]
pub(crate) struct ChunkDelta {
    pub chunk_coord: ChunkCoord,
    runs: Arc<Vec<PixelRun<Option<CompactPixel>>>>,
}

impl ChunkData {
    pub(crate) fn make_random() -> Self {
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
    }

    #[cfg(test)]
    pub(crate) fn new(mat: u16) -> Self {
        let mut runner = PixelRunner::new();
        for _ in 0..CHUNK_SIZE * CHUNK_SIZE {
            runner.put_pixel(
                Pixel {
                    flags: PixelFlags::Normal,
                    material: mat,
                }
                .to_compact(),
            )
        }
        let runs = runner.build();
        ChunkData { runs }
    }

    pub(crate) fn apply_to_chunk(&self, chunk: &mut Chunk) {
        let nil = CompactPixel(NonZeroU16::new(4095).unwrap());
        let mut offset = 0;
        for run in &self.runs {
            let len = (run.length as usize).min(CHUNK_AREA - offset);
            if len != run.length as usize {
                warn!("Truncating over-long chunk data from peer");
            }
            let pixel = run.data;
            if pixel != nil {
                for _ in 0..len {
                    chunk.set_compact_pixel(offset, pixel);
                    offset += 1;
                }
            } else {
                offset += len
            }
            if offset >= CHUNK_AREA {
                break;
            }
        }
    }
    pub(crate) fn apply_delta(&mut self, delta: ChunkData) {
        let nil = CompactPixel(NonZeroU16::new(4095).unwrap());
        let mut chunk = Chunk::default();
        self.apply_to_chunk(&mut chunk);
        let mut offset = 0;
        for run in delta.runs.iter() {
            let len = (run.length as usize).min(CHUNK_AREA - offset);
            if len != run.length as usize {
                warn!("Truncating over-long chunk delta from peer");
            }
            if run.data != nil {
                for _ in 0..len {
                    chunk.set_compact_pixel(offset, run.data);
                    offset += 1;
                }
            } else {
                offset += len
            }
            if offset >= CHUNK_AREA {
                break;
            }
        }
        *self = chunk.to_chunk_data()
    }
}

impl WorldModel {
    fn get_chunk_coords(x: i32, y: i32) -> (ChunkCoord, usize) {
        let chunk_x = x.div_euclid(CHUNK_SIZE as i32);
        let chunk_y = y.div_euclid(CHUNK_SIZE as i32);
        let x = x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let y = y.rem_euclid(CHUNK_SIZE as i32) as usize;
        let offset = x + y * CHUNK_SIZE;
        (ChunkCoord(chunk_x, chunk_y), offset)
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

    pub fn apply_noita_update(
        &mut self,
        update: &NoitaWorldUpdate,
        changed: &mut FxHashSet<ChunkCoord>,
    ) {
        fn set_pixel(pixel: Pixel, chunk: &mut Chunk, offset: usize) -> bool {
            let current = chunk.pixel(offset);
            if current != pixel {
                chunk.set_pixel(offset, pixel);
                true
            } else {
                false
            }
        }
        let header = &update.header;
        let runs = &update.runs;
        let mut x = 0;
        let mut y = 0;
        // The declared rectangle bounds how many pixels this update may write.
        // Runs come off the wire, so without this a run set that sums past
        // (w+1)*(h+1) would keep writing into rows below the rectangle - i.e.
        // into unrelated neighbouring chunks.
        let capacity = (i64::from(header.w) + 1) * (i64::from(header.h) + 1);
        let mut written: i64 = 0;
        let (mut chunk_coord, _) = Self::get_chunk_coords(header.x, header.y);
        let mut chunk = self.chunks.entry(chunk_coord).or_default();
        // Both sets below are keyed by chunk, so touching them per pixel was
        // 2-3 hash operations per changed pixel. Accumulate and flush on chunk
        // transition instead.
        let mut chunk_dirty = false;
        'outer: for run in runs {
            let flags = if run.data.flags > 0 {
                PixelFlags::Fluid
            } else {
                PixelFlags::Normal
            };
            for _ in 0..run.length {
                if written >= capacity {
                    break 'outer;
                }
                written += 1;
                let xs = header.x + x;
                let ys = header.y + y;
                let (new_chunk_coord, offset) = Self::get_chunk_coords(xs, ys);
                if chunk_coord != new_chunk_coord {
                    if chunk_dirty {
                        self.updated_chunks.insert(chunk_coord);
                        changed.remove(&chunk_coord);
                        chunk_dirty = false;
                    }
                    chunk_coord = new_chunk_coord;
                    chunk = self.chunks.entry(chunk_coord).or_default();
                }
                if set_pixel(
                    Pixel {
                        material: run.data.material,
                        flags,
                    },
                    chunk,
                    offset,
                ) {
                    chunk_dirty = true;
                }
                x += 1;
                if x == i32::from(header.w) + 1 {
                    x = 0;
                    y += 1;
                }
            }
        }
        if chunk_dirty {
            self.updated_chunks.insert(chunk_coord);
            changed.remove(&chunk_coord);
        }
    }

    /// Serialize one whole chunk. Callers always want chunk-aligned,
    /// CHUNK_SIZE-square regions, so this indexes the chunk directly instead of
    /// going through a hash lookup per pixel.
    fn get_chunk_noita_update(&self, chunk_coord: ChunkCoord) -> NoitaWorldUpdate {
        let x = chunk_coord.0 * (CHUNK_SIZE as i32);
        let y = chunk_coord.1 * (CHUNK_SIZE as i32);
        let mut runner = PixelRunner::new();
        match self.chunks.get(&chunk_coord) {
            // Offsets are laid out as `x + y * CHUNK_SIZE`, so a linear walk
            // visits pixels in the same order as the old row-major loop.
            Some(chunk) => {
                for offset in 0..(CHUNK_SIZE * CHUNK_SIZE) {
                    runner.put_pixel(chunk.pixel(offset).to_raw())
                }
            }
            None => {
                let unknown = Pixel::default().to_raw();
                for _ in 0..(CHUNK_SIZE * CHUNK_SIZE) {
                    runner.put_pixel(unknown)
                }
            }
        }
        runner.into_noita_update(x, y, (CHUNK_SIZE - 1) as u8, (CHUNK_SIZE - 1) as u8)
    }

    pub fn get_all_noita_updates(&self) -> Vec<Vec<u8>> {
        let mut updates = Vec::new();
        for chunk_coord in &self.updated_chunks {
            updates.push(self.get_chunk_noita_update(*chunk_coord).save());
        }
        updates
    }

    pub(crate) fn apply_chunk_delta(&mut self, delta: &ChunkDelta) {
        self.updated_chunks.insert(delta.chunk_coord);
        let chunk = self.chunks.entry(delta.chunk_coord).or_default();
        let mut offset = 0;
        for run in delta.runs.iter() {
            let len = (run.length as usize).min(CHUNK_AREA - offset);
            if len != run.length as usize {
                warn!("Truncating over-long chunk delta from peer");
            }
            if let Some(pixel) = run.data {
                for _ in 0..len {
                    chunk.set_compact_pixel(offset, pixel);
                    offset += 1;
                }
            } else {
                offset += len
            }
            if offset >= CHUNK_AREA {
                break;
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
            runner.put_pixel((ignore_changed || chunk.changed(i)).then(|| chunk.compact_pixel(i)))
        }
        let runs = runner.build().into();
        Some(ChunkDelta { chunk_coord, runs })
    }

    pub fn updated_chunks(&self) -> &FxHashSet<ChunkCoord> {
        &self.updated_chunks
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

#[cfg(test)]
mod wire_bounds_tests {
    use super::encoding::{Header, RawPixel};
    use super::*;

    fn px(v: u16) -> CompactPixel {
        CompactPixel(NonZeroU16::new(v).unwrap())
    }

    /// Run lengths arrive from peers and are not otherwise validated; a run set
    /// summing past the chunk used to index a fixed [u16; 16384] out of bounds
    /// and take down the net thread.
    #[test]
    fn over_long_runs_are_clamped() {
        let data = ChunkData {
            runs: vec![
                PixelRun {
                    length: u32::MAX,
                    data: px(42),
                },
                PixelRun {
                    length: u32::MAX,
                    data: px(43),
                },
            ],
        };
        let mut chunk = Chunk::default();
        data.apply_to_chunk(&mut chunk);

        let mut model = WorldModel::default();
        model.apply_chunk_delta(&ChunkDelta {
            chunk_coord: ChunkCoord(0, 0),
            runs: Arc::new(vec![PixelRun {
                length: u32::MAX,
                data: Some(px(42)),
            }]),
        });

        let mut target = ChunkData {
            runs: vec![PixelRun {
                length: (CHUNK_SIZE * CHUNK_SIZE) as u32,
                data: px(7),
            }],
        };
        target.apply_delta(data);
    }

    /// The declared rectangle bounds how many pixels an update may write, so
    /// excess runs must not spill into neighbouring chunks.
    #[test]
    fn noita_update_respects_declared_rect() {
        // w/h are width/height minus one, so this declares an 8x8 = 64 pixel
        // rectangle while supplying 4096 pixels of runs.
        let update = NoitaWorldUpdate {
            header: Header {
                x: 0,
                y: 0,
                w: 7,
                h: 7,
                run_count: 1,
            },
            runs: vec![PixelRun {
                length: 4096,
                data: RawPixel {
                    material: 1,
                    flags: 0,
                },
            }],
        };
        let mut model = WorldModel::default();
        let mut changed = FxHashSet::default();
        model.apply_noita_update(&update, &mut changed);
        // 8x8 declared; nothing outside chunk (0,0) may have been touched.
        assert!(
            model.updated_chunks.iter().all(|c| *c == ChunkCoord(0, 0)),
            "update escaped its declared rectangle: {:?}",
            model.updated_chunks
        );
    }
}
