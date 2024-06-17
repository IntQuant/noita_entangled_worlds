use std::mem;

use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use world_model::{ChunkDelta, WorldModel};

pub use world_model::encoding::NoitaWorldUpdate;

pub mod world_model;

#[derive(Debug, Serialize, Deserialize)]
pub enum WorldUpdateKind {
    Update(NoitaWorldUpdate),
    End,
}

#[derive(Default)]
pub struct WorldManager {
    model: WorldModel,
}

#[derive(Debug, Decode, Encode)]
pub struct WorldDelta(Vec<ChunkDelta>);

impl WorldDelta {
    pub fn split(self, limit: usize) -> Vec<WorldDelta> {
        let mut res = Vec::new();
        let mut current = Vec::new();
        let mut current_size = 0;
        for delta in self.0 {
            if current_size < limit || current.is_empty() {
                current_size += delta.estimate_size();
                current.push(delta);
            } else {
                res.push(WorldDelta(mem::take(&mut current)));
                current_size = 0;
            }
        }
        if !current.is_empty() {
            res.push(WorldDelta(mem::take(&mut current)));
        }
        res
    }
}

impl WorldManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_update(&mut self, update: NoitaWorldUpdate) {
        self.model.apply_noita_update(&update);
    }

    pub fn add_end(&mut self) -> WorldDelta {
        let deltas = self.model.get_all_deltas();
        self.model.reset_change_tracking();
        WorldDelta(deltas)
    }

    pub fn handle_deltas(&mut self, deltas: WorldDelta) {
        self.model.apply_all_deltas(&deltas.0);
    }

    pub fn get_noita_updates(&mut self) -> Vec<Vec<u8>> {
        let updates = self.model.get_all_noita_updates();
        self.model.reset_change_tracking();
        updates
    }

    pub fn send_world(&self) -> WorldDelta {
        WorldDelta(self.model.get_world_as_deltas())
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
        let mut model2 = WorldModel::new();
        let mut entry_id = 0;
        let mut deltas_size = 0;

        while let Ok(entry) = bincode::deserialize_from::<_, WorldUpdateKind>(&mut file)
            .inspect_err(|e| println!("{}", e))
        {
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
                    let deltas = model.get_all_deltas();
                    deltas_size += lz4_flex::compress_prepend_size(&bitcode::encode(&deltas)).len();

                    model.reset_change_tracking();
                    model2.apply_all_deltas(&deltas);
                }
            }
        }

        let (x, y) = model.get_start();
        let img = model.gen_image(x, y, 2048 * 2, 2048 * 2);
        img.save(format!("/tmp/img_{}.png", entry_id)).unwrap();

        let img = model2.gen_image(x, y, 2048 * 2, 2048 * 2);
        img.save(format!("/tmp/img_model2.png")).unwrap();

        println!("Deltas: {} bytes", deltas_size)
        // let mut mats = model.mats.iter().copied().collect::<Vec<_>>();
        // mats.sort();
        // for mat in mats {
        //     println!("{}", mat)
        // }
    }
}
