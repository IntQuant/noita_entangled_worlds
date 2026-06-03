//! Various common public types.

use std::fmt::Display;

use bitcode::{Decode, Encode};

/// Per-peer settings. Peers that are connected to the same host, as well as the host itself, should have the same settings.
#[derive(Debug, Clone, Default)]
pub struct Settings {}

/// Tells how reliable a message is.
#[derive(Encode, Decode, Clone, Copy, PartialEq, Debug)]
pub enum Reliability {
    /// Message will be delivered at most once.
    Unreliable,
    /// Message will be resent untill is's arrival will be confirmed.
    /// Will be delivered at most once.
    Reliable,
}

impl Reliability {
    pub fn from_reliability_bool(reliable: bool) -> Reliability {
        if reliable {
            Reliability::Reliable
        } else {
            Reliability::Unreliable
        }
    }
}

#[derive(Debug, Encode, Decode, Clone, Copy, PartialEq, Eq)]
pub enum Destination {
    One(PeerId),
    Broadcast,
}

/// A value which refers to a specific peer.
/// Peer 0 is always the host.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Encode, Decode)]
pub struct PeerId(pub u16);

/// Possible network events, returned by `Peer.recv()`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NetworkEvent {
    /// A new peer has connected.
    PeerConnected(PeerId),
    /// Peer has disconnected.
    PeerDisconnected(PeerId),
    /// Message has been received.
    Message(Message),
}

/// A message received from a peer.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Message {
    /// Original peer who sent the message.
    pub src: PeerId,
    /// The data that has been sent.
    pub data: Vec<u8>,
}

/// Current peer state
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeerState {
    /// Waiting for connection. Switches to `Connected` right after id from the host has been acquired.
    /// Note: hosts switches to 'Connected' basically instantly.
    #[default]
    PendingConnection,
    /// Connected to host and ready to send/receive messages.
    Connected,
    /// No longer connected, won't reconnect.
    Disconnected,
}

impl Display for PeerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PeerState::PendingConnection => write!(f, "Connection pending..."),
            PeerState::Connected => write!(f, "Connected"),
            PeerState::Disconnected => write!(f, "Disconnected"),
        }
    }
}

impl PeerId {
    pub const HOST: PeerId = PeerId(0);
}

impl Display for PeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Destination {
    pub(crate) fn is_broadcast(self) -> bool {
        matches!(self, Destination::Broadcast)
    }
    pub(crate) fn to_one(self) -> Option<PeerId> {
        if let Self::One(peer_id) = self {
            Some(peer_id)
        } else {
            None
        }
    }
}
