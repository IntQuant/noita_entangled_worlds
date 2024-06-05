use bitcode::{Decode, Encode};

use crate::{net::world::WorldDelta, GameSettings};

#[derive(Debug, Decode, Encode)]
pub enum NetMsg {
    Welcome,
    StartGame { settings: GameSettings },
    ModRaw { data: Vec<u8> },
    ModCompressed { data: Vec<u8> },
    WorldDelta { delta: WorldDelta },
    WorldFrame,
}
