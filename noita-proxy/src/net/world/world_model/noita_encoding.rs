use bytemuck::{bytes_of, pod_read_unaligned, AnyBitPattern, NoUninit};
use serde::{Deserialize, Serialize};
use std::mem::size_of;

#[derive(Debug, Clone, Copy, AnyBitPattern, NoUninit, Serialize, Deserialize)]
#[repr(C)]
pub(crate) struct Header {
    pub x: i32,
    pub y: i32,
    pub w: u8,
    pub h: u8,
    pub run_count: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NoitaWorldUpdate {
    pub(crate) header: Header,
    pub(crate) runs: Vec<PixelRun>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct PixelRun {
    pub(crate) length: u16,
    pub(crate) material: i16,
    pub(crate) flags: u8,
}

struct ByteParser<'a> {
    data: &'a [u8],
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

    fn next_run(&mut self) -> PixelRun {
        PixelRun {
            length: self.next(),
            material: self.next(),
            flags: self.next(),
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
            buf.extend_from_slice(bytes_of(&run.length));
            buf.extend_from_slice(bytes_of(&run.material));
            buf.extend_from_slice(bytes_of(&run.flags));
        }

        buf
    }
}
