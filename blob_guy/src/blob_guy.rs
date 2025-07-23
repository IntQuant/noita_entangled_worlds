use crate::chunk::Chunks;
use crate::chunk::{CellType, ChunkPos, Pos};
use crate::{CHUNK_AMOUNT, State};
#[cfg(target_arch = "x86")]
use noita_api::EntityID;
use rustc_hash::{FxBuildHasher, FxHashMap};
use std::f32::consts::{PI, TAU};
pub const OFFSET: isize = CHUNK_AMOUNT as isize / 2;
impl State {
    pub fn update(&mut self) -> eyre::Result<()> {
        if noita_api::raw::input_is_mouse_button_just_down(1)? {
            unsafe {
                self.particle_world_state
                    .assume_init_mut()
                    .debug_mouse_pos()?
            };
        }
        if self.blobs.is_empty() {
            self.push_new();
            return Ok(());
        }
        'upper: for blob in self.blobs.iter_mut() {
            let mean = blob.mean();
            blob.update_pos()?;
            let start = Pos::new(mean.0, mean.1).to_chunk();
            if self
                .world
                .read(
                    unsafe { self.particle_world_state.assume_init_ref() },
                    self.blob_guy,
                    start,
                )
                .is_err()
            {
                blob.update(start, &mut self.world, mean, false)?;
                continue 'upper;
            }
            blob.update(start, &mut self.world, mean, true)?;
            self.world.paint(
                unsafe { self.particle_world_state.assume_init_mut() },
                self.blob_guy,
                start,
            );
            blob.cull();
        }
        Ok(())
    }
    pub fn push_new(&mut self) {
        self.blobs.push(Blob::new(256.0, -(64.0 + 32.0)));
        let start = self.blobs[0].pos.to_chunk();
        if self
            .world
            .read(
                unsafe { self.particle_world_state.assume_init_ref() },
                self.blob_guy,
                start,
            )
            .is_err()
        {
            return;
        }
        self.blobs[0].register_pixels(start, &mut self.world);
        self.world.paint(
            unsafe { self.particle_world_state.assume_init_mut() },
            self.blob_guy,
            start,
        );
    }
}
pub const SIZE: usize = 24;
pub struct Blob {
    pub pos: Pos,
    pub pixels: FxHashMap<(isize, isize), Pixel>,
    pub count: usize,
}
#[derive(Default, Copy, Clone)]
pub struct Pixel {
    pub pos: Pos,
    velocity: Pos,
    stop: Option<usize>,
    mutated: bool,
    temp: bool,
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
    pub fn cull(&mut self) {
        self.pixels.retain(|_, p| !p.temp);
    }
    pub fn update_pos(&mut self) -> eyre::Result<()> {
        #[cfg(target_arch = "x86")]
        {
            if let Ok(player) =
                EntityID::get_closest_with_tag(self.pos.x as f64, self.pos.y as f64, "player_unit")
            {
                let (x, y) = player.position()?;
                self.pos.x = x as f32;
                self.pos.y = y as f32 - 7.0;
            }
        }
        Ok(())
    }
    pub fn mean(&self) -> (f32, f32) {
        let n = self
            .pixels
            .values()
            .fold((0.0, 0.0), |acc, p| (acc.0 + p.pos.x, acc.1 + p.pos.y));
        (
            n.0 / self.pixels.len() as f32,
            n.1 / self.pixels.len() as f32,
        )
    }
    const THETA_COUNT: usize = 16;
    pub fn get_thetas(&self, r: f32) -> [bool; Self::THETA_COUNT] {
        let mut arr = [0; Self::THETA_COUNT];
        for p in self.pixels.values() {
            let dx = self.pos.x - p.pos.x;
            let dy = self.pos.y - p.pos.y;
            if dy.hypot(dx) < r {
                let n = Self::THETA_COUNT as f32 * (dy.atan2(dx) / TAU + 0.5);
                arr[n as usize] += 1;
            }
        }
        let l = self.pixels.len().div_ceil(Self::THETA_COUNT);
        arr.map(|n| n < l / 2)
    }
    pub fn update(
        &mut self,
        start: ChunkPos,
        map: &mut Chunks,
        mean: (f32, f32),
        loaded: bool,
    ) -> eyre::Result<()> {
        let r = (self.pixels.len() as f32 / PI).sqrt().ceil();
        let array = &self.get_thetas(r);
        let theta = (mean.1 - self.pos.y).atan2(mean.0 - self.pos.x);
        for p in self.pixels.values_mut() {
            p.mutated = false;
        }
        let mut keys = self.pixels.keys().cloned().collect::<Vec<(isize, isize)>>();
        keys.sort_unstable_by(|(a, b), (x, y)| {
            let da = self.pos.x.floor() as isize - a;
            let db = self.pos.y.floor() as isize - b;
            let dx = self.pos.x.floor() as isize - x;
            let dy = self.pos.y.floor() as isize - y;
            let r1 = da * da + db * db;
            let r2 = dx * dx + dy * dy;
            r2.cmp(&r1)
        });
        while !keys.is_empty()
            && let Some((c, p)) = self.pixels.remove_entry(&keys.remove(0))
        {
            self.run(c, p, theta, map, start, array);
        }
        if !loaded {
            return Ok(());
        }
        self.register_pixels(start, map);
        Ok(())
    }
    pub fn register_pixels(&mut self, start: ChunkPos, map: &mut Chunks) {
        let mut last = ChunkPos::new(isize::MAX, isize::MAX);
        let mut k = 0;
        for (x, y) in self.pixels.keys().copied() {
            let c = ChunkPos::new(x, y);
            if c != last {
                let new = c.get_world(start);
                if new < 0 || new as usize >= CHUNK_AMOUNT * CHUNK_AMOUNT {
                    continue;
                }
                k = new as usize;
                last = c;
            }

            map.0[k][(x, y)] = match map.0[k][(x, y)] {
                CellType::Unknown | CellType::Liquid => {
                    map.0[k].modified = true;
                    CellType::Blob
                }
                _ => CellType::Ignore,
            }
        }
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
            count: SIZE * SIZE,
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn run(
        &mut self,
        c: (isize, isize),
        mut p: Pixel,
        theta: f32,
        world: &Chunks,
        start: ChunkPos,
        array: &[bool; Self::THETA_COUNT],
    ) -> bool {
        if p.mutated {
            self.pixels.insert(c, p);
            return false;
        }
        p.mutated = true;
        let dx = self.pos.x - p.pos.x;
        let dy = self.pos.y - p.pos.y;
        let dist = dy.hypot(dx).max(0.1);
        let psi = dy.atan2(dx);
        let angle = psi - theta;
        let mag = 512.0;
        let (ax, ay) = if !array[(Self::THETA_COUNT as f32 * (dy.atan2(dx) / TAU + 0.5)) as usize] {
            let n = if psi < theta + PI {
                psi + PI / 4.0
            } else {
                psi - PI / 4.0
            };
            let (s, c) = n.sin_cos();
            let (dx, dy) = (dist * c, dist * s);
            (mag * dx / dist, mag * dy / dist)
        } else {
            let phi = (3.0 * angle.abs() / PI + 1.0).min(3.0);
            (phi * mag * dx / dist, phi * mag * dy / dist)
        };
        p.velocity.x += ax / 60.0;
        p.velocity.y += ay / 60.0;
        let damping = 0.9;
        p.velocity.x *= damping;
        p.velocity.y *= damping;
        let mut n;
        if let Some(k) = p.stop.as_mut() {
            let phi = p.velocity.y.atan2(p.velocity.x);
            let t = phi + DIRECTIONS[*k / 2];
            let (sin, cos) = t.sin_cos();
            p.pos.x += cos * if k.is_multiple_of(2) { 1.0 } else { 1.5 };
            p.pos.y += sin * if k.is_multiple_of(2) { 1.0 } else { 1.5 };
            n = (p.pos.x.floor() as isize, p.pos.y.floor() as isize);
            let mut l = *k;
            if self.pixels.contains_key(&n) {
                loop {
                    if l.div_ceil(2) == DIRECTIONS.len() {
                        l = 0;
                    } else {
                        l += 1;
                    }
                    if l == *k {
                        break;
                    }
                    let t = phi + DIRECTIONS[*k / 2] + PI;
                    let (sin, cos) = t.sin_cos();
                    let x = p.pos.x + cos * if k.is_multiple_of(2) { 1.0 } else { 1.5 };
                    let y = p.pos.y + sin * if k.is_multiple_of(2) { 1.0 } else { 1.5 };
                    n = (x.floor() as isize, y.floor() as isize);
                    if !self.pixels.contains_key(&n) {
                        p.pos.x = x;
                        p.pos.y = y;
                        break;
                    }
                }
            }
            if k.div_ceil(2) == DIRECTIONS.len() {
                *k = 0;
            } else {
                *k += 1;
            }
        } else {
            p.pos.x += p.velocity.x / 60.0;
            p.pos.y += p.velocity.y / 60.0;
            n = (p.pos.x.floor() as isize, p.pos.y.floor() as isize);
        }
        if c != n {
            let pos = ChunkPos::new(n.0, n.1);
            let index = pos.get_world(start);
            if index >= 0
                && let Some(chunk) = world.0.get(index as usize)
                && match chunk[n] {
                    CellType::Unknown => false,
                    CellType::Blob => false,
                    CellType::Remove => false,
                    CellType::Ignore => false,
                    CellType::Other => false,
                    CellType::Solid => true,
                    CellType::Liquid => true,
                    CellType::Sand => true,
                    CellType::Physics => true,
                }
            {
                if matches!(chunk[n], CellType::Liquid) {
                    self.pixels.insert(n, p);
                    self.count += 1;
                }
                p.pos.x = c.0 as f32 + 0.5;
                p.pos.y = c.1 as f32 + 0.5;
                if p.stop.is_none() {
                    p.stop = Some(0)
                }
                self.pixels.insert(c, p);
                false
            } else if let Some(b) = self.pixels.remove(&n) {
                if self.run(n, b, theta, world, start, array) && !self.pixels.contains_key(&n) {
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
