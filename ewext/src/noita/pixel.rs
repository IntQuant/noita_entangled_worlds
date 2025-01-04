#[allow(clippy::repr_packed_without_abi)]
#[repr(packed)]
pub(crate) struct NoitaPixelRun {
    pub(crate) length: u16,
    pub(crate) material: u16,
    pub(crate) flags: u8,
}

/// Copied from proxy.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub(crate) struct RawPixel {
    pub material: u16,
    pub flags: u8,
}

/// Copied from proxy.
/// Stores a run of pixels.
/// Not specific to Noita side - length is an actual length
#[derive(Debug)]
pub(crate) struct PixelRun<Pixel> {
    pub length: u32,
    pub data: Pixel,
}

/// Copied from proxy.
/// Converts a normal sequence of pixels to a run-length-encoded one.
pub(crate) struct PixelRunner<Pixel> {
    pub(crate) current_pixel: Option<Pixel>,
    pub(crate) current_run_len: u32,
    pub(crate) runs: Vec<PixelRun<Pixel>>,
}

impl<Pixel: Eq + Copy> Default for PixelRunner<Pixel> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Pixel: Eq + Copy> PixelRunner<Pixel> {
    pub(crate) fn new() -> Self {
        Self {
            current_pixel: None,
            current_run_len: 0,
            runs: Vec::new(),
        }
    }
    pub(crate) fn put_pixel(&mut self, pixel: Pixel) {
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
    pub(crate) fn build(&mut self) -> &[PixelRun<Pixel>] {
        if self.current_run_len > 0 {
            self.runs.push(PixelRun {
                length: self.current_run_len,
                data: self.current_pixel.expect("has current pixel"),
            });
        }
        &mut self.runs
    }

    pub(crate) fn clear(&mut self) {
        self.current_pixel = None;
        self.current_run_len = 0;
        self.runs.clear();
    }
}
