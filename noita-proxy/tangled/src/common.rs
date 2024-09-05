//! Various common public types.

use std::{fmt::Display, time::Duration};

use serde::{Deserialize, Serialize};

/// Per-peer settings. Peers that are connected to the same host, as well as the host itself, should have the same settings.
#[derive(Debug, Clone)]
pub struct Settings {
    /// A single datagram will confirm at most this much messages. Default is 128.
    pub confirm_max_per_message: usize,
    /// How much time can elapse before another confirm is sent.
    /// Confirms are also sent when enough messages are awaiting confirm.
    /// Note that confirms also double as "heartbeats" and keep the connection alive, so this value should be much less than `connection_timeout`.
    /// Default: 1 second.
    pub confirm_max_period: Duration,
    /// Peers will be disconnected after this much time without any datagrams from them has passed.
    /// Default: 10 seconds.
    pub connection_timeout: Duration,
}

/// Tells how reliable a message is.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Reliability {
    /// Message will be delivered at most once.
    Unreliable,
    /// Message will be resent untill is's arrival will be confirmed.
    /// Will be delivered at most once.
    Reliable,
}

pub enum Destination {
    One(PeerId),
    Broadcast,
}

/// A value which refers to a specific peer.
/// Peer 0 is always the host.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
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

impl Default for Settings {
    fn default() -> Self {
        Self {
            confirm_max_per_message: 128,
            confirm_max_period: Duration::from_secs(1),
            connection_timeout: Duration::from_secs(10),
        }
    }
}
