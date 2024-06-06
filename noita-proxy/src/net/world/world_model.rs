use std::mem::size_of;

use bitcode::{Decode, Encode};
use chunk::{Chunk, Pixel, PixelFlags};
use encoding::{NoitaWorldUpdate, PixelRun, PixelRunner};
use image::{Rgb, RgbImage};
use rustc_hash::{FxHashMap, FxHashSet};

mod chunk;
pub mod encoding;

const CHUNK_SIZE: usize = 128;

type ChunkCoord = (i32, i32);

pub struct WorldModel {
    chunks: FxHashMap<ChunkCoord, Chunk>,
    pub mats: FxHashSet<u16>,
    palette: MatPalette,
    changed_chunks: FxHashSet<ChunkCoord>,
}

struct MatPalette {
    colors: Vec<Rgb<u8>>,
}

#[derive(Debug, Encode, Decode)]
pub struct ChunkDelta {
    runs: Vec<PixelRun<Option<Pixel>>>,
    chunk_coord: (i32, i32),
}

impl ChunkDelta {
    pub fn estimate_size(&self) -> usize {
        8 + self.runs.len() * size_of::<PixelRun<Option<Pixel>>>()
    }
}

impl MatPalette {
    fn new() -> Self {
        let raw_data = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/mat_colors.txt"
        ));

        let mut colors = Vec::new();

        for line in raw_data.split_ascii_whitespace() {
            let num: u32 = line.parse().unwrap();
            let color = Rgb::from([(num >> 16) as u8, (num >> 8) as u8, num as u8]);
            colors.push(color);
        }
        Self { colors }
    }
}

impl WorldModel {
    fn to_chunk_coords(x: i32, y: i32) -> (ChunkCoord, usize) {
        let chunk_x = x.div_euclid(CHUNK_SIZE as i32);
        let chunk_y = y.div_euclid(CHUNK_SIZE as i32);
        let x = x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let y = y.rem_euclid(CHUNK_SIZE as i32) as usize;
        let offset = x + y * CHUNK_SIZE;
        ((chunk_x, chunk_y), offset)
    }

    fn set_pixel(&mut self, x: i32, y: i32, pixel: Pixel) {
        self.mats.insert(pixel.material);
        let (chunk_coord, offset) = Self::to_chunk_coords(x, y);
        let chunk = self.chunks.entry(chunk_coord).or_default();
        let current = chunk.pixel(offset);
        if current != pixel {
            chunk.set_pixel(offset, pixel);
        }
        self.changed_chunks.insert(chunk_coord);
    }

    fn get_pixel(&self, x: i32, y: i32) -> Pixel {
        let (chunk_coord, offset) = Self::to_chunk_coords(x, y);
        self.chunks
            .get(&chunk_coord)
            .map(|chunk| chunk.pixel(offset))
            .unwrap_or_default()
    }

    pub fn new() -> Self {
        Self {
            chunks: Default::default(),
            mats: Default::default(),
            palette: MatPalette::new(),
            changed_chunks: Default::default(),
        }
    }

    pub fn apply_noita_update(&mut self, update: &NoitaWorldUpdate) {
        let header = &update.header;
        let runs = &update.runs;

        let mut x = 0;
        let mut y = 0;

        for run in runs {
            let flags = if run.data.flags > 0 {
                PixelFlags::Fluid
            } else {
                PixelFlags::Normal
            };
            for _ in 0..(run.length) {
                self.set_pixel(
                    header.x + x,
                    header.y + y,
                    Pixel {
                        material: run.data.material,
                        flags,
                    },
                );
                x += 1;
                if x == i32::from(header.w) + 1 {
                    x = 0;
                    y += 1;
                }
            }
        }
    }

    pub fn get_noita_update(&self, x: i32, y: i32, w: u32, h: u32) -> NoitaWorldUpdate {
        assert!(w <= 256);
        assert!(h <= 256);
        let mut runner = PixelRunner::new();
        for j in 0..(h as i32) {
            for i in 0..(w as i32) {
                runner.put_pixel(self.get_pixel(x + i, y + j).to_raw())
            }
        }
        runner.to_noita(x, y, (w - 1) as u8, (h - 1) as u8)
    }

    pub fn get_all_noita_updates(&self) -> Vec<Vec<u8>> {
        let mut updates = Vec::new();
        for chunk_coord in &self.changed_chunks {
            let update = self.get_noita_update(
                chunk_coord.0 * (CHUNK_SIZE as i32),
                chunk_coord.1 * (CHUNK_SIZE as i32),
                CHUNK_SIZE as u32,
                CHUNK_SIZE as u32,
            );
            updates.push(update.save());
        }
        updates
    }

    fn apply_chunk_delta(&mut self, delta: &ChunkDelta) {
        self.changed_chunks.insert(delta.chunk_coord);
        let chunk = self.chunks.entry(delta.chunk_coord).or_default();
        let mut offset = 0;
        for run in &delta.runs {
            for _ in 0..(run.length) {
                if let Some(pixel) = run.data {
                    chunk.set_pixel(offset, pixel)
                }
                offset += 1;
            }
        }
        assert_eq!(offset, CHUNK_SIZE * CHUNK_SIZE)
    }

    fn get_chunk_delta(&self, chunk_coord: ChunkCoord, ignore_changed: bool) -> Option<ChunkDelta> {
        let chunk = self.chunks.get(&chunk_coord)?;
        let mut runner = PixelRunner::new();
        for i in 0..CHUNK_SIZE * CHUNK_SIZE {
            runner.put_pixel((ignore_changed || chunk.changed(i)).then(|| chunk.pixel(i)))
        }
        let runs = runner.build();
        Some(ChunkDelta { chunk_coord, runs })
    }

    pub fn apply_all_deltas(&mut self, deltas: &[ChunkDelta]) {
        for delta in deltas {
            self.apply_chunk_delta(delta)
        }
    }

    pub fn get_all_deltas(&self) -> Vec<ChunkDelta> {
        self.changed_chunks
            .iter()
            .filter_map(|&chunk_coord| self.get_chunk_delta(chunk_coord, false))
            .collect()
    }

    pub fn reset_change_tracking(&mut self) {
        for chunk_pos in &self.changed_chunks {
            if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
                chunk.clear_changed();
            }
        }
        self.changed_chunks.clear();
    }

    pub fn get_start(&self) -> (i32, i32) {
        let cx = self.chunks.keys().map(|v| v.0).min().unwrap_or_default();
        let cy = self.chunks.keys().map(|v| v.1).min().unwrap_or_default();
        (cx * CHUNK_SIZE as i32, cy * CHUNK_SIZE as i32)
    }

    pub fn gen_image(&self, x: i32, y: i32, w: u32, h: u32) -> RgbImage {
        RgbImage::from_fn(w, h, |lx, ly| {
            let pixel = self.get_pixel(x + lx as i32, y + ly as i32);
            // let b = if pixel.material != 0 { 255 } else { 0 };
            // Rgb([pixel.material as u8, (pixel.material >> 8) as u8, b])
            let mat_color = self
                .palette
                .colors
                .get(usize::try_from(pixel.material).unwrap_or_default())
                .copied()
                .unwrap_or(Rgb([0, 0, 0]));
            if pixel.flags != PixelFlags::Unknown {
                mat_color
            } else {
                Rgb([25, 0, 0])
            }
        })
    }

    pub fn get_world_as_deltas(&self) -> Vec<ChunkDelta> {
        self.chunks
            .keys()
            .filter_map(|&chunk_coord| self.get_chunk_delta(chunk_coord, true))
            .collect()
    }
}
