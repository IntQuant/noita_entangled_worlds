use super::{omni::OmniPeerId, world::WorldNetMessage};
use crate::net::world::world_model::ChunkData;
use crate::{GameSettings, player_cosmetics::PlayerPngDesc};
use bitcode::{Decode, Encode};
use rustc_hash::FxHashMap;
use shared::world_sync::ChunkCoord;
pub(crate) type Destination = shared::Destination<OmniPeerId>;

pub(crate) struct MessageRequest<T> {
    pub(crate) reliability: tangled::Reliability,
    pub(crate) dst: Destination,
    pub(crate) msg: T,
}

#[derive(Debug, Decode, Encode, Clone)]
pub(crate) enum NetMsg {
    Welcome,
    RequestMods,
    Mods { mods: Vec<String> },
    EndRun,
    Kick,
    PeerDisconnected { id: OmniPeerId },
    StartGame { settings: GameSettings, init: bool },
    ModRaw { data: Vec<u8> },
    ModCompressed { data: Vec<u8> },
    WorldMessage(WorldNetMessage),
    PlayerColor(PlayerPngDesc, bool, Option<OmniPeerId>, String),
    RemoteMsg(shared::RemoteMessage),
    ForwardDesToProxy(shared::des::DesToProxy),
    ForwardWorldSyncToProxy(shared::world_sync::WorldSyncToProxy),
    ForwardProxyToDes(shared::des::ProxyToDes),
    ForwardProxyToWorldSync(shared::world_sync::ProxyToWorldSync),
    NoitaDisconnected,
    Flags(String),
    RespondFlagNormal(String, bool),
    RespondFlagSlow(usize, bool),
    RespondFlagMoon(i32, i32, bool),
    PlayerPosition(i32, i32, bool, bool),
    RespondFlagStevari(i32, i32, OmniPeerId),
    AudioData(Vec<Vec<u8>>, bool, i32, i32, f32),
    MapData(FxHashMap<ChunkCoord, ChunkData>),
    MatData(FxHashMap<u16, u32>),
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
