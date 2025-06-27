use crate::chunk::{Chunk, ChunkPos};
use crate::{CHUNK_SIZE, State};
use noita_api::{game_print, print};
use rustc_hash::FxHashMap;
use std::collections::hash_map::Entry;
#[derive(Default, Debug)]
pub struct Pos {
    x: f64,
    y: f64,
}
impl Pos {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    pub fn to_chunk(&self) -> ChunkPos {
        ChunkPos::new(self.x as i32, self.y as i32)
    }
    pub fn to_chunk_inner(&self) -> usize {
        self.y.rem_euclid(CHUNK_SIZE as f64) as usize * CHUNK_SIZE
            + self.x.rem_euclid(CHUNK_SIZE as f64) as usize
    }
}
impl State {
    pub fn update(&mut self) {
        if let Some(pws) = self.particle_world_state.as_mut() {
            self.world.clear();
            'upper: for blob in &self.blobs {
                let c = blob.pos.to_chunk();
                for x in c.x - 1..=c.x + 1 {
                    for y in c.y - 1..=c.y + 1 {
                        let c = ChunkPos { x, y };
                        match self.world.entry(c) {
                            Entry::Occupied(_) => {}
                            Entry::Vacant(c) => {
                                let chunk = unsafe { pws.encode_area(x, y) };
                                match chunk {
                                    Some(v) => {
                                        c.insert(v);
                                    }
                                    None => continue 'upper,
                                }
                            }
                        }
                    }
                }
            }
        }
        if self.blobs.is_empty() {
            self.blobs.push(Blob::new(16.0, -128.0))
        }
        for blob in self.blobs.iter_mut() {
            blob.update(&mut self.world, self.blob_guy)
        }
    }
}
const SIZE: usize = 9;
pub struct Blob {
    pub pos: Pos,
    pixels: [Pos; SIZE * SIZE],
}
impl Blob {
    pub fn update(&mut self, map: &mut FxHashMap<ChunkPos, Chunk>, blob_guy: u16) {
        game_print(self.pos.x.to_string());
        game_print(self.pos.y.to_string());
        print(format!("{:?}", self.pixels));
        let mut last = ChunkPos::new(i32::MAX, i32::MAX);
        let mut chunk = Chunk::default();
        for p in &self.pixels {
            let c = p.to_chunk();
            if c != last {
                chunk = map.remove(&c).unwrap_or_default();
                last = c;
            }
            let k = p.to_chunk_inner();
            chunk.pixels[k] = blob_guy
        }
    }
    pub fn new(x: f64, y: f64) -> Self {
        Blob {
            pos: Pos::new(x, y),
            pixels: std::array::from_fn(|i| {
                let a = (i / SIZE) as f64 - SIZE as f64 / 2.0;
                let b = (i % SIZE) as f64 - SIZE as f64 / 2.0;
                Pos::new(x + a, y + b)
            }),
        }
    }
}
