use bitcode::{Decode, Encode};

use crate::{net::world::WorldDelta, GameSettings};

use super::{omni::OmniPeerId, world::WorldNetMessage};

pub(crate) enum Destination {
    Peer(OmniPeerId),
    Host,
    Broadcast,
}

pub(crate) struct MessageRequest<T> {
    reliability: tangled::Reliability,
    dest: Destination,
    datum: T,
}

#[derive(Debug, Decode, Encode)]
pub enum NetMsg {
    Welcome,
    StartGame { settings: GameSettings },
    ModRaw { data: Vec<u8> },
    ModCompressed { data: Vec<u8> },
    WorldDelta { delta: WorldDelta },
    WorldFrame,
    WorldMessage(WorldNetMessage),
}
