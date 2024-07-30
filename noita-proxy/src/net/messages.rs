use bitcode::{Decode, Encode};

use crate::{net::world::WorldDelta, GameSettings};

use super::{omni::OmniPeerId, world::WorldNetMessage};

pub(crate) enum Destination {
    Peer(OmniPeerId),
    Host,
    Broadcast,
}

pub(crate) struct MessageRequest<T> {
    pub(crate) reliability: tangled::Reliability,
    pub(crate) dst: Destination,
    pub(crate) msg: T,
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

impl From<MessageRequest<WorldNetMessage>> for MessageRequest<NetMsg> {
    fn from(value: MessageRequest<WorldNetMessage>) -> Self {
        Self {
            msg: NetMsg::WorldMessage(value.msg),
            reliability: value.reliability,
            dst: value.dst,
        }
    }
}
