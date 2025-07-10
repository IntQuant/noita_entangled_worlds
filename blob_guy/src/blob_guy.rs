use crate::chunk::Chunks;
use crate::chunk::{CellType, ChunkPos, Pos};
use crate::{CHUNK_AMOUNT, State};
#[cfg(target_arch = "x86")]
use noita_api::EntityID;
use rustc_hash::{FxBuildHasher, FxHashMap};
use std::f32::consts::PI;
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
            blob.update_pos()?;
            let start = blob.pos.to_chunk();
            if self
                .world
                .read(
                    unsafe { self.particle_world_state.assume_init_ref() },
                    self.blob_guy,
                    start,
                )
                .is_err()
            {
                blob.update(start, &mut self.world)?;
                continue 'upper;
            }
            blob.update(start, &mut self.world)?;
            self.world.paint(
                unsafe { self.particle_world_state.assume_init_mut() },
                self.blob_guy,
                start,
            );
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
}
#[derive(Default, Copy, Clone)]
pub struct Pixel {
    pub pos: Pos,
    velocity: Pos,
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
    pub fn mean(&self) -> (f64, f64) {
        let n = self
            .pixels
            .keys()
            .fold((0, 0), |acc, x| (acc.0 + x.0, acc.1 + x.1));
        (
            n.0 as f64 / self.pixels.len() as f64,
            n.1 as f64 / self.pixels.len() as f64,
        )
    }
    pub fn update(&mut self, start: ChunkPos, map: &mut Chunks) -> eyre::Result<()> {
        let mean = self.mean();
        let theta = (mean.1 as f32 - self.pos.y).atan2(mean.0 as f32 - self.pos.x);
        for p in self.pixels.values_mut() {
            p.mutated = false;
        }
        let mut keys = self.pixels.keys().cloned().collect::<Vec<(isize, isize)>>();
        while !keys.is_empty()
            && let Some((c, p)) = self.pixels.remove_entry(&keys.remove(0))
        {
            self.run(c, p, theta, map, start);
        }
        for _ in 0..3 {
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
    pub fn run(
        &mut self,
        c: (isize, isize),
        mut p: Pixel,
        theta: f32,
        world: &Chunks,
        start: ChunkPos,
    ) -> bool {
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
        let mut n;
        if let Some(k) = p.stop.as_mut() {
            let phi = p.velocity.y.atan2(p.velocity.x);
            let t = phi + DIRECTIONS[*k / 2] + PI;
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
                }
                p.pos.x = c.0 as f32 + 0.5;
                p.pos.y = c.1 as f32 + 0.5;
                if p.stop.is_none() {
                    p.stop = Some(0)
                }
                self.pixels.insert(c, p);
                false
            } else if let Some(b) = self.pixels.remove(&n) {
                if self.run(n, b, theta, world, start) && !self.pixels.contains_key(&n) {
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
