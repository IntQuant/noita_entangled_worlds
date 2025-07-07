use crate::chunk::{CellType, Chunk, ChunkPos};
use crate::{CHUNK_AMOUNT, CHUNK_SIZE, State};
#[cfg(target_arch = "x86")]
use noita_api::EntityID;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use rustc_hash::{FxBuildHasher, FxHashMap};
use std::f32::consts::PI;
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
        if noita_api::raw::input_is_mouse_button_just_down(1)? {
            let (x, y) = noita_api::raw::debug_get_mouse_world()?;
            let pos = Pos {
                x: x as f32,
                y: y as f32,
            }
            .to_chunk();
            if let Ok((_, _, pixel_array)) = self.particle_world_state.set_chunk(pos.x, pos.y) {
                if let Some(cell) = self.particle_world_state.get_cell_raw(
                    (x.floor() as isize).rem_euclid(512),
                    (y.floor() as isize).rem_euclid(512),
                    pixel_array,
                ) {
                    noita_api::print(format!("{cell:?}"));
                    noita_api::print(format!("{:?}", unsafe { cell.material_ptr.as_ref() }));
                } else {
                    noita_api::print("mat nil");
                }
            }
        }
        if self.blobs.is_empty() {
            self.blobs.push(Blob::new(256.0, -64.0 - 32.0));
        }
        'upper: for blob in self.blobs.iter_mut() {
            blob.update_pos()?;
            let c = blob.pos.to_chunk();
            if self
                .world
                .par_iter_mut()
                .enumerate()
                .try_for_each(|(i, chunk)| unsafe {
                    let x = i as isize / CHUNK_AMOUNT as isize + c.x;
                    let y = i as isize % CHUNK_AMOUNT as isize + c.y;
                    self.particle_world_state
                        .encode_area(x - OFFSET, y - OFFSET, chunk)
                })
                .is_err()
            {
                blob.update(&mut [])?;
                continue 'upper;
            }
            blob.update(&mut self.world)?;
            self.world.iter().enumerate().for_each(|(i, chunk)| unsafe {
                let x = i as isize / CHUNK_AMOUNT as isize + c.x;
                let y = i as isize % CHUNK_AMOUNT as isize + c.y;
                let _ = self
                    .particle_world_state
                    .decode_area(x - OFFSET, y - OFFSET, chunk);
            });
        }
        Ok(())
    }
}
pub const SIZE: usize = 24;
pub struct Blob {
    pub pos: Pos,
    pub pixels: FxHashMap<(isize, isize), Pixel>,
}
#[derive(Default, Copy, Clone)]
pub struct Pixel {
    pub pos: Pos,
    velocity: Pos,
    acceleration: Pos,
    stop: Option<usize>,
    mutated: bool,
}
const DIRECTIONS: [f32; 7] = [
    0.0,
    PI / 4.0,
    -PI / 4.0,
    PI / 3.0,
    -PI / 3.0,
    PI / 2.3,
    -PI / 2.3,
];
impl Pixel {
    fn new(pos: Pos) -> Self {
        Pixel {
            pos,
            ..Default::default()
        }
    }
}
impl Blob {
    pub fn update_pos(&mut self) -> eyre::Result<()> {
        #[cfg(target_arch = "x86")]
        {
            let player = EntityID::get_closest_with_tag(
                self.pos.x as f64,
                self.pos.y as f64,
                "player_unit",
            )?;
            let (x, y) = player.position()?;
            self.pos.x = x as f32;
            self.pos.y = y as f32 - 7.0;
        }
        Ok(())
    }
    fn mean(&self) -> (isize, isize) {
        let n = self
            .pixels
            .keys()
            .fold((0, 0), |acc, x| (acc.0 + x.0, acc.1 + x.1));
        (
            n.0 / self.pixels.len() as isize,
            n.1 / self.pixels.len() as isize,
        )
    }
    pub fn update(&mut self, map: &mut [Chunk]) -> eyre::Result<()> {
        let mean = self.mean();
        let theta = (mean.1 as f32 - self.pos.y).atan2(mean.0 as f32 - self.pos.x);
        for p in self.pixels.values_mut() {
            p.mutated = false;
        }
        let mut keys = self.pixels.keys().cloned().collect::<Vec<(isize, isize)>>();
        while !keys.is_empty()
            && let Some((c, p)) = self.pixels.remove_entry(&keys.remove(0))
        {
            self.run(c, p, theta);
        }
        for _ in 0..5 {
            let mut boundary: FxHashMap<(isize, isize), i8> = FxHashMap::default();
            for (a, b) in self.pixels.keys().cloned() {
                let k = (a - 1, b);
                boundary.entry(k).and_modify(|u| *u += 1).or_default();
                let k = (a, b - 1);
                boundary.entry(k).and_modify(|u| *u += 1).or_default();
                let k = (a + 1, b);
                boundary.entry(k).and_modify(|u| *u += 1).or_default();
                let k = (a, b + 1);
                boundary.entry(k).and_modify(|u| *u += 1).or_default();
                boundary.insert((a, b), i8::MIN);
            }
            let mut boundary2: FxHashMap<(isize, isize), i8> = FxHashMap::default();
            for ((a, b), p) in boundary {
                boundary2.insert((a, b), p);
                let k = (a - 1, b);
                boundary2.entry(k).and_modify(|u| *u += 1);
                let k = (a, b - 1);
                boundary2.entry(k).and_modify(|u| *u += 1);
                let k = (a + 1, b);
                boundary2.entry(k).and_modify(|u| *u += 1);
                let k = (a, b + 1);
                boundary2.entry(k).and_modify(|u| *u += 1);
            }
            let mut is_some = false;
            for (a, b) in boundary2 {
                if b >= 3 {
                    let far = self.far();
                    self.pixels.insert(a, far);
                    is_some = true
                }
            }
            if !is_some {
                break;
            }
        }
        let start = self.pos.to_chunk();
        let mut last = ChunkPos::new(isize::MAX, isize::MAX);
        let mut k = 0;
        for (x, y) in self.pixels.keys() {
            let c = ChunkPos::new(*x, *y);
            if c != last {
                let new =
                    (c.x - start.x + OFFSET) * CHUNK_AMOUNT as isize + (c.y - start.y + OFFSET);
                if new < 0 || new as usize >= CHUNK_AMOUNT * CHUNK_AMOUNT {
                    continue;
                }
                k = new as usize;
                last = c;
            }
            let n = x.rem_euclid(CHUNK_SIZE as isize) as usize * CHUNK_SIZE
                + y.rem_euclid(CHUNK_SIZE as isize) as usize;

            map[k][n] = match map[k][n] {
                CellType::Unknown => {
                    map[k].modified = true;
                    CellType::Blob
                }
                _ => CellType::Ignore,
            }
        }
        Ok(())
    }
    pub fn far(&mut self) -> Pixel {
        let (a, b) = (self.pos.x.floor() as isize, self.pos.y.floor() as isize);
        let mut l = isize::MIN;
        let mut ret = (0, 0);
        for (c, d) in self.pixels.keys() {
            let dx = c - a;
            let dy = d - b;
            let m = dx * dx + dy * dy;
            if m > l {
                l = m;
                ret = (*c, *d);
            }
        }
        self.pixels.remove(&ret).unwrap()
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
    pub fn run(&mut self, c: (isize, isize), mut p: Pixel, theta: f32) -> bool {
        if p.mutated {
            self.pixels.insert(c, p);
            return false;
        }
        p.mutated = true;
        let dx = self.pos.x - p.pos.x;
        let dy = self.pos.y - p.pos.y;
        let dist = dy.hypot(dx).max(0.1);
        let phi = (3.0 * (dy.atan2(dx) - theta).abs() / PI + 1.0).min(3.0);
        let mag = 512.0 * phi;
        let ax = mag * dx / dist;
        let ay = mag * dy / dist;
        p.velocity.x += ax / 60.0;
        p.velocity.y += ay / 60.0;
        let damping = 0.9;
        p.velocity.x *= damping;
        p.velocity.y *= damping;
        if let Some(n) = p.stop.as_mut() {
            let t = p.acceleration.y.atan2(p.acceleration.x) + DIRECTIONS[*n / 2];
            p.pos.x = p.pos.x.floor() + 0.5 + t.cos() * if n.is_multiple_of(2) { 1.0 } else { 1.5 };
            p.pos.y = p.pos.y.floor() + 0.5 + t.sin() * if n.is_multiple_of(2) { 1.0 } else { 1.5 };
            let mut m = (p.pos.x.floor() as isize, p.pos.y.floor() as isize);
            let mut k = *n;
            if self.pixels.contains_key(&m) {
                loop {
                    if k.div_ceil(2) == DIRECTIONS.len() {
                        k = 0;
                    } else {
                        k += 1;
                    }
                    if k == *n {
                        break;
                    }
                    let t = p.acceleration.y.atan2(p.acceleration.x) + DIRECTIONS[*n / 2];
                    let x = p.pos.x.floor()
                        + 0.5
                        + t.cos() * if n.is_multiple_of(2) { 1.0 } else { 1.5 };
                    let y = p.pos.y.floor()
                        + 0.5
                        + t.sin() * if n.is_multiple_of(2) { 1.0 } else { 1.5 };
                    m = (x.floor() as isize, y.floor() as isize);
                    if !self.pixels.contains_key(&m) {
                        p.pos.x = x;
                        p.pos.y = y;
                        break;
                    }
                }
            }
            if n.div_ceil(2) == DIRECTIONS.len() {
                *n = 0;
            } else {
                *n += 1;
            }
        } else {
            p.pos.x += p.velocity.x / 60.0;
            p.pos.y += p.velocity.y / 60.0;
        }
        let n = (p.pos.x.floor() as isize, p.pos.y.floor() as isize);
        if c != n {
            if let Some(b) = self.pixels.remove(&n) {
                if self.run(n, b, theta) && !self.pixels.contains_key(&n) {
                    p.stop = None;
                    self.pixels.insert(n, p);
                    true
                } else {
                    p.pos.x = c.0 as f32 + 0.5;
                    p.pos.y = c.1 as f32 + 0.5;
                    if p.stop.is_none() {
                        p.stop = Some(0)
                    }
                    self.pixels.insert(c, p);
                    false
                }
            } else {
                p.stop = None;
                self.pixels.insert(n, p);
                true
            }
        } else {
            self.pixels.insert(c, p);
            false
        }
    }
}
