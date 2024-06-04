use bitcode::{Decode, Encode};
use bytemuck::{bytes_of, pod_read_unaligned, AnyBitPattern, NoUninit};
use serde::{Deserialize, Serialize};
use std::mem::size_of;

#[derive(Debug, Clone, Copy, AnyBitPattern, NoUninit, Serialize, Deserialize, PartialEq, Eq)]
#[repr(C)]
pub(crate) struct Header {
    pub x: i32,
    pub y: i32,
    pub w: u8,
    pub h: u8,
    pub run_count: u16,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NoitaWorldUpdate {
    pub(crate) header: Header,
    pub(crate) runs: Vec<PixelRun<RawPixel>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub(crate) struct RawPixel {
    pub material: u16,
    pub flags: u8,
}

/// Stores a run of pixels.
/// Not specific to Noita side - length is an actual length
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Encode, Decode)]
pub struct PixelRun<Pixel> {
    pub length: u32,
    pub data: Pixel,
}

struct ByteParser<'a> {
    data: &'a [u8],
}

pub struct PixelRunner<Pixel> {
    current_pixel: Option<Pixel>,
    current_run_len: u32,
    runs: Vec<PixelRun<Pixel>>,
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

impl PixelRunner<RawPixel> {
    /// Note: w/h are actualy width/height -1
    pub fn to_noita(self, x: i32, y: i32, w: u8, h: u8) -> NoitaWorldUpdate {
        let runs = self.build();
        NoitaWorldUpdate {
            header: Header {
                x,
                y,
                w,
                h,
                run_count: runs.len() as u16,
            },
            runs,
        }
    }
}

impl<'a> ByteParser<'a> {
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
            length: u32::from(self.next::<u16>()) + 1,
            data: RawPixel {
                material: self.next(),
                flags: self.next(),
            },
        }
    }
}

impl NoitaWorldUpdate {
    pub fn load(data: &[u8]) -> Self {
        let mut parser = ByteParser::new(data);

        let header: Header = parser.next();
        let mut runs = Vec::with_capacity(header.run_count.into());

        for _ in 0..header.run_count {
            runs.push(parser.next_run());
        }

        assert!(parser.data.is_empty());

        Self { header, runs }
    }
    pub fn save(&self) -> Vec<u8> {
        let header = Header {
            run_count: self.runs.len() as u16,
            ..self.header
        };
        let mut buf = Vec::new();
        buf.extend_from_slice(bytes_of(&header));

        for run in &self.runs {
            let len = u16::try_from(run.length - 1).unwrap();
            buf.extend_from_slice(bytes_of(&len));
            buf.extend_from_slice(bytes_of(&run.data.material));
            buf.extend_from_slice(bytes_of(&run.data.flags));
        }

        buf
    }
}
