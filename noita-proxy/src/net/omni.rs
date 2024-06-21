use super::steam_networking;
use std::fmt::Display;
use steamworks::{LobbyId, SteamId};
use tangled::{PeerId, PeerState, Reliability};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OmniPeerId(pub u64);

impl From<PeerId> for OmniPeerId {
    fn from(value: PeerId) -> Self {
        Self(value.0.into())
    }
}

impl From<SteamId> for OmniPeerId {
    fn from(value: SteamId) -> Self {
        Self(value.raw())
    }
}

impl From<OmniPeerId> for PeerId {
    fn from(value: OmniPeerId) -> Self {
        Self(
            value
                .0
                .try_into()
                .expect("Assuming PeerId was stored here, so conversion should succeed"),
        )
    }
}

impl From<OmniPeerId> for SteamId {
    fn from(value: OmniPeerId) -> Self {
        Self::from_raw(value.0)
    }
}

impl Display for OmniPeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl OmniPeerId {
    pub fn from_hex(val: &str) -> Option<Self> {
        let raw = u64::from_str_radix(val, 16).ok()?;
        Some(Self(raw))
    }

    pub(crate) fn to_hex(&self) -> String {
        format!("{:016x}", self.0)
    }
}

pub enum OmniNetworkEvent {
    PeerConnected(OmniPeerId),
    PeerDisconnected(OmniPeerId),
    Message { src: OmniPeerId, data: Vec<u8> },
}

impl From<tangled::NetworkEvent> for OmniNetworkEvent {
    fn from(value: tangled::NetworkEvent) -> Self {
        match value {
            tangled::NetworkEvent::PeerConnected(id) => Self::PeerConnected(id.into()),
            tangled::NetworkEvent::PeerDisconnected(id) => Self::PeerDisconnected(id.into()),
            tangled::NetworkEvent::Message(msg) => Self::Message {
                src: msg.src.into(),
                data: msg.data,
            },
        }
    }
}

pub enum PeerVariant {
    Tangled(tangled::Peer),
    Steam(steam_networking::SteamPeer),
}

impl PeerVariant {
    pub(crate) fn send(
        &self,
        peer: OmniPeerId,
        msg: Vec<u8>,
        reliability: Reliability,
    ) -> Result<(), tangled::NetError> {
        match self {
            PeerVariant::Tangled(p) => p.send(peer.into(), msg, reliability),
            PeerVariant::Steam(p) => {
                p.send_message(peer.into(), &msg, reliability);
                Ok(())
            }
        }
    }

    pub(crate) fn broadcast(
        &self,
        msg: Vec<u8>,
        reliability: Reliability,
    ) -> Result<(), tangled::NetError> {
        match self {
            PeerVariant::Tangled(p) => p.broadcast(msg, reliability),
            PeerVariant::Steam(p) => {
                p.broadcast_message(&msg, reliability);
                Ok(())
            }
        }
    }

    pub(crate) fn my_id(&self) -> Option<OmniPeerId> {
        match self {
            PeerVariant::Tangled(p) => p.my_id().map(OmniPeerId::from),
            PeerVariant::Steam(p) => Some(p.my_id().into()),
        }
    }

    pub fn iter_peer_ids(&self) -> Vec<OmniPeerId> {
        match self {
            PeerVariant::Tangled(p) => p.iter_peer_ids().map(OmniPeerId::from).collect(),
            PeerVariant::Steam(p) => p.get_peer_ids().into_iter().map(OmniPeerId::from).collect(),
        }
    }

    pub(crate) fn recv(&self) -> Vec<OmniNetworkEvent> {
        match self {
            PeerVariant::Tangled(p) => p.recv().map(OmniNetworkEvent::from).collect(),
            PeerVariant::Steam(p) => p.recv(),
        }
    }

    pub fn state(&self) -> PeerState {
        match self {
            PeerVariant::Tangled(p) => p.state(),
            PeerVariant::Steam(p) => p.state(),
        }
    }

    pub fn host_id(&self) -> OmniPeerId {
        match self {
            PeerVariant::Tangled(_) => PeerId::HOST.into(),
            PeerVariant::Steam(p) => p.host_id().into(),
        }
    }

    pub fn lobby_id(&self) -> Option<LobbyId> {
        match self {
            PeerVariant::Tangled(_) => None,
            PeerVariant::Steam(p) => p.lobby_id(),
        }
    }

    pub fn is_steam(&self) -> bool {
        matches!(self, PeerVariant::Steam(_))
    }
}
