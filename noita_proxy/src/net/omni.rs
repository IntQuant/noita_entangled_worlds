use super::steam_networking::{self, ExtraPeerState};
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use steamworks::{LobbyId, SteamError, SteamId};
use tangled::{PeerId, Reliability};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Decode, Encode)]
pub struct OmniPeerId(pub u64);

impl From<shared::PeerId> for OmniPeerId {
    fn from(value: shared::PeerId) -> Self {
        OmniPeerId(value.0)
    }
}

impl From<OmniPeerId> for shared::PeerId {
    fn from(value: OmniPeerId) -> Self {
        shared::PeerId(value.0)
    }
}

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

/// `OmniPeerId` is a 64-bit union of a tangled `PeerId` (16 bits) and a
/// `SteamId` (64 bits), with no discriminant. Ids reach us from the wire (e.g.
/// `NetMsg::PlayerColor`) and from the local mod, so narrowing is genuinely
/// fallible - this used to be an infallible `From` with an `expect`, which meant
/// one oversized id from any peer killed the net thread.
impl TryFrom<OmniPeerId> for PeerId {
    type Error = tangled::NetError;

    fn try_from(value: OmniPeerId) -> Result<Self, Self::Error> {
        value
            .0
            .try_into()
            .map(Self)
            .map_err(|_| tangled::NetError::UnknownPeer)
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

    pub(crate) fn as_hex(&self) -> String {
        format!("{:016x}", self.0)
    }
}

pub enum OmniNetworkEvent {
    PeerConnected(OmniPeerId),
    PeerDisconnected(OmniPeerId),
    Message {
        src: OmniPeerId,
        data: Vec<u8>,
    },
    /// Reliable messages to this peer have been backlogged for too long. Steam only.
    SendBacklogStuck(OmniPeerId),
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

#[allow(clippy::large_enum_variant)]
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
            PeerVariant::Tangled(p) => p.send(peer.try_into()?, msg, reliability),
            PeerVariant::Steam(p) => {
                p.send_message(peer.into(), &msg, reliability)
                    .map_err(|e| match e {
                        SteamError::InvalidSteamID => tangled::NetError::UnknownPeer,
                        // Only unreliable sends get LimitExceeded; reliable ones are backlogged.
                        SteamError::Ignored | SteamError::LimitExceeded => {
                            tangled::NetError::Dropped
                        }
                        SteamError::InvalidParameter => tangled::NetError::MessageTooLong,
                        SteamError::NoConnection | SteamError::InvalidState => {
                            tangled::NetError::Disconnected
                        }
                        _ => tangled::NetError::Other,
                    })
            }
        }
    }

    pub(crate) fn flush(&self) {
        if let PeerVariant::Steam(p) = self {
            p.flush()
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

    pub(crate) fn my_id(&self) -> OmniPeerId {
        match self {
            PeerVariant::Tangled(p) => p
                .my_id()
                .map(OmniPeerId::from)
                .expect("Peer id to be available"),
            PeerVariant::Steam(p) => p.my_id().into(),
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

    pub fn state(&self) -> ExtraPeerState {
        match self {
            PeerVariant::Tangled(p) => ExtraPeerState::Tangled(p.state()),
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

    pub fn is_host(&self) -> bool {
        match self {
            PeerVariant::Tangled(_) => self.host_id() == self.my_id(),
            PeerVariant::Steam(p) => p.is_host(),
        }
    }
}
