use shared::world_sync::PixelRun;
/*struct ByteParser<'a> {
    data: &'a [u8],
}*/

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

/*impl<'a> ByteParser<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    fn next<T: AnyBitPattern>(&mut self) -> T {
        let size = size_of::<T>();
        let sli = &self.data[..size];
        self.data = &self.data[size..];
        pod_read_unaligned(sli)
    }

    fn next_run(&mut self) -> PixelRun<RawPixel> {
        PixelRun {
            length: self.next::<u16>() + 1,
            data: RawPixel {
                material: self.next(),
                flags: self.next(),
            },
        }
    }
}
*/
