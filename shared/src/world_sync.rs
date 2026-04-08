use bitcode::{Decode, Encode};
/// Stores a run of pixels.
/// Not specific to Noita side - length is an actual length
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub struct PixelRun<Pixel> {
    pub length: u16,
    pub data: Pixel,
}

/// Converts a normal sequence of pixels to a run-length-encoded one.
pub struct PixelRunner<Pixel> {
    current_pixel: Option<Pixel>,
    current_run_len: u16,
    runs: Vec<PixelRun<Pixel>>,
}

impl<Pixel: Eq + Copy> Default for PixelRunner<Pixel> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Pixel: Eq + Copy> PixelRunner<Pixel> {
    pub fn compress(data: &[Pixel]) -> Vec<PixelRun<Pixel>> {
        let mut me = Self::new();
        for pixel in data {
            me.put_pixel(*pixel);
        }
        me.build()
    }
    pub fn new() -> Self {
        Self {
            current_pixel: None,
            current_run_len: 0,
            runs: Vec::new(),
        }
    }
    pub fn put_pixel(&mut self, pixel: Pixel) {
        if let Some(current) = self.current_pixel {
            if pixel != current {
                self.runs.push(PixelRun {
                    length: self.current_run_len,
                    data: current,
                });
                self.current_pixel = Some(pixel);
                self.current_run_len = 1;
            } else {
                self.current_run_len += 1;
            }
        } else {
            self.current_pixel = Some(pixel);
            self.current_run_len = 1;
        }
    }
    pub fn build(mut self) -> Vec<PixelRun<Pixel>> {
        if self.current_run_len > 0 {
            self.runs.push(PixelRun {
                length: self.current_run_len,
                data: self.current_pixel.expect("has current pixel"),
            });
        }
        self.runs
    }
}

pub const CHUNK_SIZE: usize = 128;
pub const CHUNKLET_SIZE_POWER: isize = 7;

#[derive(Debug, Encode, Decode, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChunkCoord(pub i32, pub i32);

#[derive(Debug, Encode, Decode, Clone)]
pub struct NoitaWorldUpdate {
    pub coord: ChunkCoord,
    pub pixel_runs: Vec<PixelRun<Pixel>>,
}

impl NoitaWorldUpdate {
    pub fn iter_pixels(&self) -> impl Iterator<Item = Pixel> {
        self.pixel_runs
            .iter()
            .flat_map(|run| std::iter::repeat_n(run.data, run.length as usize))
    }
    pub fn is_all_empty_pixels(&self) -> bool {
        self.pixel_runs.iter().all(|pix| pix.data.is_air())
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Encode, Decode)]
pub enum PixelFlags {
    /// Actual material isn't known yet.
    Normal = 0,
    Abnormal = 1,
    #[default]
    Unknown = 15,
    //may have at most * = 15
}

/// An entire pixel packed into 12 bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode, Default)]
#[repr(transparent)]
pub struct Pixel(u16);

#[test]
fn test() {
    let p = Pixel::new(0, PixelFlags::Unknown);
    assert_eq!(p.mat(), 0);
    assert_eq!(p.flags(), PixelFlags::Unknown);
    let p = Pixel::new(0, PixelFlags::Normal);
    assert_eq!(p.mat(), 0);
    assert_eq!(p.flags(), PixelFlags::Normal);
    let p = Pixel::new(15, PixelFlags::Unknown);
    assert_eq!(p.mat(), 15);
    assert_eq!(p.flags(), PixelFlags::Unknown);
    let p = Pixel::new(15, PixelFlags::Normal);
    assert_eq!(p.mat(), 15);
    assert_eq!(p.flags(), PixelFlags::Normal);
}

impl Pixel {
    pub const NIL: Pixel = Pixel::new(0, PixelFlags::Abnormal);
    //mat must be less then 13 bits
    #[inline(always)]
    pub const fn new(mat: u16, flag: PixelFlags) -> Self {
        Self(mat | ((flag as u16) << 12))
    }
    #[inline(always)]
    pub fn mat(self) -> u16 {
        self.0 & 0x0FFF
    }
    #[inline(always)]
    pub fn flags(self) -> PixelFlags {
        unsafe { std::mem::transmute((self.0 >> 12) as u8) }
    }
    #[inline(always)]
    pub fn is_air(&self) -> bool {
        self.mat() == 0
    }
}

#[derive(Debug, Encode, Decode, Clone)]
pub enum WorldSyncToProxy {
    Updates(Vec<NoitaWorldUpdate>),
    End(Option<(i32, i32, i32, i32, bool)>, u8, u8, Vec<ChunkCoord>),
}
#[derive(Debug, Encode, Decode, Clone)]
pub enum ProxyToWorldSync {
    Updates(Vec<NoitaWorldUpdate>),
}
