use bitcode::{Decode, Encode};
use shared::des::DesToProxy;

use crate::bookkeeping::save_state::{SaveState, SaveStateEntry};

#[derive(Encode, Decode, Default)]
struct EntityStorage {}

impl SaveStateEntry for EntityStorage {
    const FILENAME: &'static str = "des_entity_storage";
}

pub(crate) struct DesManager {
    entity_storage: EntityStorage,
}

impl DesManager {
    pub(crate) fn new(is_host: bool, save_state: SaveState) -> Self {
        let entity_storage = save_state.load().unwrap_or_default();
        Self { entity_storage }
    }

    pub(crate) fn handle_noita_msg(&mut self, msg: DesToProxy) {
        todo!()
    }
}
