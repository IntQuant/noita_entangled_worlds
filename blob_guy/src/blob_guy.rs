use crate::chunk::{CellType, Chunk, ChunkPos};
use crate::{CHUNK_AMOUNT, CHUNK_SIZE, State};
#[derive(Default, Debug, Clone, Copy)]
pub struct Pos {
    x: f64,
    y: f64,
}
impl Pos {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    pub fn to_chunk(self) -> ChunkPos {
        ChunkPos::new(self.x.floor() as isize, self.y.floor() as isize)
    }
    pub fn to_chunk_inner(self) -> usize {
        self.x.rem_euclid(CHUNK_SIZE as f64) as usize * CHUNK_SIZE
            + self.y.rem_euclid(CHUNK_SIZE as f64) as usize
    }
}
const OFFSET: isize = CHUNK_AMOUNT as isize / 2;
impl State {
    pub fn update(&mut self) -> eyre::Result<()> {
        if self.blobs.is_empty() {
            self.blobs
                .push(Blob::new(128.0 + 16.0, -(128.0 + 64.0 + 16.0)));
        }
        'upper: for blob in self.blobs.iter_mut() {
            let c = blob.pos.to_chunk();
            for (k, (x, y)) in (-OFFSET..=OFFSET)
                .flat_map(|i| (-OFFSET..=OFFSET).map(move |j| (i, j)))
                .enumerate()
            {
                if unsafe {
                    !self
                        .particle_world_state
                        .encode_area(c.x + x, c.y + y, &mut self.world[k])
                } {
                    continue 'upper;
                }
            }
            blob.update(&mut self.world);
            for (k, (x, y)) in (-OFFSET..=OFFSET)
                .flat_map(|i| (-OFFSET..=OFFSET).map(move |j| (i, j)))
                .enumerate()
            {
                unsafe {
                    self.particle_world_state
                        .decode_area(c.x + x, c.y + y, &self.world[k]);
                }
            }
        }
        Ok(())
    }
}
const SIZE: usize = 8;
pub struct Blob {
    pub pos: Pos,
    pixels: [Option<Pos>; SIZE * SIZE],
}
impl Blob {
    pub fn update(&mut self, map: &mut [Chunk; 9]) {
        let mut last = ChunkPos::new(isize::MAX, isize::MAX);
        let mut k = 0;
        let start = self.pos.to_chunk();
        self.pos.x += 0.1;
        self.pos.y += 0.1;
        for p in self.pixels.iter_mut().flatten() {
            p.x += 0.1;
            p.y += 0.1;
            let c = p.to_chunk();
            if c != last {
                k = ((c.x - start.x + OFFSET) * CHUNK_AMOUNT as isize + (c.y - start.y + OFFSET))
                    as usize;
                last = c;
            }
            let n = p.to_chunk_inner();

            map[k][n] = if matches!(map[k][n], CellType::Remove) {
                CellType::Ignore
            } else {
                CellType::Blob
            }
        }
    }
    pub fn new(x: f64, y: f64) -> Self {
        Blob {
            pos: Pos::new(x, y),
            pixels: std::array::from_fn(|i| {
                let a = (i / SIZE) as f64 - SIZE as f64 / 2.0 + 0.5;
                let b = (i % SIZE) as f64 - SIZE as f64 / 2.0 + 0.5;
                Some(Pos::new(x + a, y + b))
            }),
        }
    }
}
