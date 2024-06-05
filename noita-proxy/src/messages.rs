use bitcode::{Decode, Encode};

use crate::{net::world::world_model::ChunkDelta, GameSettings};

#[derive(Debug, Decode, Encode)]
pub enum NetMsg {
    Welcome,
    StartGame { settings: GameSettings },
    ModRaw { data: Vec<u8> },
    ModCompressed { data: Vec<u8> },
    WorldDeltas { deltas: Vec<ChunkDelta> },
}
