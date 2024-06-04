use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufWriter};

pub use world_model::encoding::NoitaWorldUpdate;

pub mod world_model;

#[derive(Debug, Serialize, Deserialize)]
pub enum WorldUpdateKind {
    Update(NoitaWorldUpdate),
    End,
}

pub struct WorldManager {
    pub(crate) writer: BufWriter<File>,
}

impl WorldManager {
    pub fn new() -> Self {
        Self {
            writer: BufWriter::new(File::create("worldlog.bin").unwrap()),
        }
    }

    pub fn add_update(&mut self, update: NoitaWorldUpdate) {
        bincode::serialize_into(&mut self.writer, &WorldUpdateKind::Update(update)).unwrap();
    }

    pub fn add_end(&mut self) {
        bincode::serialize_into(&mut self.writer, &WorldUpdateKind::End).unwrap();
    }
}

#[cfg(test)]
mod test {
    use std::{fs::File, io::BufReader};

    use super::{world_model::WorldModel, NoitaWorldUpdate, WorldUpdateKind};

    #[test]
    fn read_replay() {
        let mut file = BufReader::new(File::open("worldlog.bin").unwrap());
        let mut model = WorldModel::new();
        let mut entry_id = 0;
        while let Ok(entry) = bincode::deserialize_from::<_, WorldUpdateKind>(&mut file) {
            match entry {
                WorldUpdateKind::Update(entry) => {
                    let saved = entry.save();
                    let loaded = NoitaWorldUpdate::load(&saved);
                    assert_eq!(entry, loaded);

                    model.apply_noita_update(&entry);
                    let new_update = model.get_noita_update(
                        entry.header.x,
                        entry.header.y,
                        entry.header.w as u32 + 1,
                        entry.header.h as u32 + 1,
                    );
                    assert_eq!(entry, new_update);
                }
                WorldUpdateKind::End => {
                    entry_id += 1;
                    if entry_id % 10000 == 0 {
                        let (x, y) = model.get_start();
                        let img = model.gen_image(x, y, 2048, 2048);
                        img.save(format!("/tmp/img_{}.png", entry_id)).unwrap();
                    }
                }
            }
        }

        let (x, y) = model.get_start();
        let img = model.gen_image(x, y, 2048 * 2, 2048 * 2);
        img.save(format!("/tmp/img_{}.png", entry_id)).unwrap();

        // let mut mats = model.mats.iter().copied().collect::<Vec<_>>();
        // mats.sort();
        // for mat in mats {
        //     println!("{}", mat)
        // }
    }
}
