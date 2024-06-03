use std::{fs::File, io::BufWriter, mem::size_of};

use bytemuck::{pod_read_unaligned, AnyBitPattern};
use serde::{Deserialize, Serialize};

pub mod world_model;

#[derive(Debug, Clone, Copy, AnyBitPattern, Serialize, Deserialize)]
#[repr(C)]
struct Header {
    x: i32,
    y: i32,
    w: u8,
    h: u8,
    run_count: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunLengthUpdate {
    header: Header,
    runs: Vec<PixelRun>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct PixelRun {
    length: u16,
    material: i16,
    flags: u8,
}

struct ByteParser<'a> {
    data: &'a [u8],
}

pub struct WorldManager {
    writer: BufWriter<File>,
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

impl RunLengthUpdate {
    pub fn load(data: &[u8]) -> Self {
        let mut parser = ByteParser::new(data);

        let header: Header = parser.next();
        let mut runs = Vec::with_capacity(header.run_count.into());

        for _ in 0..header.run_count {
            runs.push(parser.next_run());
        }

        Self { header, runs }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WorldUpdateKind {
    Update(RunLengthUpdate),
    End,
}

impl WorldManager {
    pub fn new() -> Self {
        Self {
            writer: BufWriter::new(File::create("worldlog.bin").unwrap()),
        }
    }

    pub fn add_update(&mut self, update: RunLengthUpdate) {
        bincode::serialize_into(&mut self.writer, &WorldUpdateKind::Update(update)).unwrap();
    }

    pub fn add_end(&mut self) {
        bincode::serialize_into(&mut self.writer, &WorldUpdateKind::End).unwrap();
    }
}

#[cfg(test)]
mod test {
    use std::{fs::File, io::BufReader};

    use super::{world_model::WorldModel, WorldUpdateKind};

    #[test]
    fn read_replay() {
        let mut file = BufReader::new(File::open("worldlog.bin").unwrap());
        let mut model = WorldModel::new();
        let mut entry_id = 0;
        while let Ok(entry) = bincode::deserialize_from::<_, WorldUpdateKind>(&mut file) {
            if let WorldUpdateKind::Update(entry) = entry {
                model.apply_noita_update(&entry);
                // println!("{:?}", entry.header)
                entry_id += 1;
                // if entry_id > 1000000 {
                //     break;
                // }
                if entry_id % 10000 == 0 {
                    let (x, y) = model.get_start();
                    let img = model.gen_image(x, y, 2048, 2048);
                    img.save(format!("/tmp/img_{}.png", entry_id)).unwrap();
                }
            }
        }

        let (x, y) = model.get_start();
        let img = model.gen_image(x, y, 2048 * 2, 2048 * 2);
        img.save(format!("/tmp/img_{}.png", entry_id)).unwrap();

        let mut mats = model.mats.iter().copied().collect::<Vec<_>>();
        mats.sort();
        for mat in mats {
            println!("{}", mat)
        }
    }
}
