use crate::chunk::{CellType, Chunk, ChunkPos};
use crate::{CHUNK_AMOUNT, CHUNK_SIZE, State};
#[cfg(target_arch = "x86")]
use noita_api::EntityID;
use rustc_hash::{FxBuildHasher, FxHashMap};
#[derive(Default, Debug, Clone, Copy)]
pub struct Pos {
    pub x: f32,
    pub y: f32,
}
impl Pos {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn to_chunk(self) -> ChunkPos {
        ChunkPos::new(self.x.floor() as isize, self.y.floor() as isize)
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
            blob.update(&mut self.world)?;
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
pub const SIZE: usize = 12;
pub struct Blob {
    pub pos: Pos,
    pub pixels: FxHashMap<(isize, isize), Pixel>,
}
#[derive(Default, Copy, Clone)]
pub struct Pixel {
    pub pos: Pos,
    velocity: Pos,
    acceleration: Pos,
}
impl Pixel {
    fn new(pos: Pos) -> Self {
        Pixel {
            pos,
            ..Default::default()
        }
    }
}
impl Blob {
    pub fn update(&mut self, map: &mut [Chunk; 9]) -> eyre::Result<()> {
        let mut last = ChunkPos::new(isize::MAX, isize::MAX);
        let mut k = 0;
        let start = self.pos.to_chunk();
        #[cfg(target_arch = "x86")]
        {
            let player = EntityID::get_closest_with_tag(
                self.pos.x as f64,
                self.pos.y as f64,
                "player_unit",
            )?;
            let (x, y) = player.position()?;
            let dx = x as f32 - self.pos.x;
            let dy = y as f32 - self.pos.y;
            self.pos.x += dx / 100.0;
            self.pos.y += dy / 100.0;
        }
        for (_, p) in self.pixels.iter_mut() {
            p.acceleration.x = 0.0;
            p.acceleration.y = 0.0;
        }
        {
            let mut pixels_vec: Vec<_> = self.pixels.values_mut().collect();
            for p in &mut pixels_vec {
                let dx = self.pos.x - p.pos.x;
                let dy = self.pos.y - p.pos.y;
                let dist = (dx * dx + dy * dy).sqrt().max(0.1);
                let force = 10.0 / dist;
                p.acceleration.x += dx / dist * force;
                p.acceleration.y += dy / dist * force;
            }
        }
        let mut to_change = Vec::new();
        for ((x, y), p) in self.pixels.iter_mut() {
            p.velocity.x += p.acceleration.x / 60.0;
            p.velocity.y += p.acceleration.y / 60.0;
            p.pos.x += p.velocity.x / 60.0;
            p.pos.y += p.velocity.y / 60.0;
            let (nx, ny) = (p.pos.x.floor() as isize, p.pos.y.floor() as isize);
            if *x != nx || *y != ny {
                to_change.push(((*x, *y), (nx, ny)));
            }
        }
        for (k, n) in to_change {
            let px = self.pixels.remove(&k).unwrap();
            self.pixels.insert(n, px);
        }
        noita_api::print(format!(
            "{{{}}}",
            self.pixels
                .keys()
                .map(|(a, b)| format!("{{{a},{b}}}"))
                .collect::<Vec<String>>()
                .join(",")
        ));
        for (x, y) in self.pixels.keys() {
            let c = ChunkPos::new(*x, *y);
            if c != last {
                k = ((c.x - start.x + OFFSET) * CHUNK_AMOUNT as isize + (c.y - start.y + OFFSET))
                    as usize;
                last = c;
            }
            let n = x.rem_euclid(CHUNK_SIZE as isize) as usize * CHUNK_SIZE
                + y.rem_euclid(CHUNK_SIZE as isize) as usize;

            map[k][n] = if matches!(map[k][n], CellType::Remove) {
                CellType::Ignore
            } else {
                CellType::Blob
            }
        }
        Ok(())
    }
    pub fn new(x: f32, y: f32) -> Self {
        let mut pixels = FxHashMap::with_capacity_and_hasher(SIZE * SIZE, FxBuildHasher);
        for i in 0..SIZE * SIZE {
            let a = (i / SIZE) as f32 - SIZE as f32 / 2.0 + 0.5;
            let b = (i % SIZE) as f32 - SIZE as f32 / 2.0 + 0.5;
            let p = Pixel::new(Pos::new(x + a, y + b));
            pixels.insert((p.pos.x.floor() as isize, p.pos.y.floor() as isize), p);
        }
        Blob {
            pos: Pos::new(x, y),
            pixels,
        }
    }
}
