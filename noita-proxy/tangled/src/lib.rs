//! Tangled - a work-in-progress UDP networking crate.

use std::{
    fmt::Display,
    io,
    net::{SocketAddr, UdpSocket},
    sync::{atomic::AtomicBool, Arc},
};

use crossbeam::{
    self,
    atomic::AtomicCell,
    channel::{unbounded, Receiver, Sender},
};

pub use error::NetError;
use reactor::{Destination, RemotePeer, Shared};
pub use reactor::{Reliability, Settings};
use serde::{Deserialize, Serialize};

const DATAGRAM_MAX_LEN: usize = 30000; // TODO this probably should be 1500

/// Maximum size of a message which fits into a single datagram.
pub const MAX_MESSAGE_LEN: usize = DATAGRAM_MAX_LEN - 100;

mod error;
mod reactor;
mod util;

struct Datagram {
    pub size: usize,
    pub data: [u8; DATAGRAM_MAX_LEN],
}

/// A value which refers to a specific peer.
/// Peer 0 is always the host.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct PeerId(pub u16);

impl Display for PeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

type SeqId = u16;

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

struct OutboundMessage {
    pub dst: Destination,
    pub data: Vec<u8>,
    pub reliability: Reliability,
}

/// Current peer state
#[derive(Default, Clone, Copy, Debug)]
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

type Channel<T> = (Sender<T>, Receiver<T>);

/// Represents a network endpoint. Can be constructed in either `host` or `client` mode.
/// Client can only connect to hosts, but they are able to send messages to any other peer connected to the same host, including the host itself.
#[derive(Clone)]
pub struct Peer {
    shared: Arc<Shared>,
}

impl Peer {
    fn new(
        bind_addr: SocketAddr,
        host_addr: Option<SocketAddr>,
        settings: Option<Settings>,
    ) -> io::Result<Self> {
        let socket = UdpSocket::bind(bind_addr)?;
        let shared = Arc::new(Shared {
            socket,
            inbound_channel: unbounded(),
            outbound_channel: unbounded(),
            keep_alive: AtomicBool::new(true),
            host_addr,
            peer_state: Default::default(),
            remote_peers: Default::default(),
            max_packets_per_second: 256,
            my_id: AtomicCell::new(if host_addr.is_none() {
                Some(PeerId(0))
            } else {
                None
            }),
            settings: settings.unwrap_or_default(),
        });
        if host_addr.is_none() {
            shared.remote_peers.insert(PeerId(0), RemotePeer::default());
            shared
                .inbound_channel
                .0
                .send(NetworkEvent::PeerConnected(PeerId(0)))
                .unwrap();
        }
        reactor::Reactor::start(Arc::clone(&shared));
        Ok(Peer { shared })
    }

    /// Host at a specified `bind_addr`.
    pub fn host(bind_addr: SocketAddr, settings: Option<Settings>) -> io::Result<Self> {
        Self::new(bind_addr, None, settings)
    }

    /// Connect to a specified `host_addr`.
    pub fn connect(host_addr: SocketAddr, settings: Option<Settings>) -> io::Result<Self> {
        Self::new("0.0.0.0:0".parse().unwrap(), Some(host_addr), settings)
    }

    /// Send a message to a specified single peer.
    pub fn send(
        &self,
        destination: PeerId,
        data: Vec<u8>,
        reliability: Reliability,
    ) -> Result<(), NetError> {
        self.send_internal(Destination::One(destination), data, reliability)
    }

    pub fn broadcast(&self, data: Vec<u8>, reliability: Reliability) -> Result<(), NetError> {
        self.send_internal(Destination::Broadcast, data, reliability)
    }

    fn send_internal(
        &self,
        destination: Destination,
        data: Vec<u8>,
        reliability: Reliability,
    ) -> Result<(), NetError> {
        if data.len() > MAX_MESSAGE_LEN {
            return Err(NetError::MessageTooLong);
        }
        if reliability == Reliability::Unreliable
            && self.shared.outbound_channel.0.len() * 2 > self.shared.max_packets_per_second
        {
            return Err(NetError::Dropped);
        }
        self.shared.outbound_channel.0.send(OutboundMessage {
            dst: destination,
            data,
            reliability,
        })?;
        Ok(())
    }

    /// Return an iterator over recieved messages.
    /// Does not block.
    pub fn recv(&self) -> impl Iterator<Item = NetworkEvent> + '_ {
        self.shared.inbound_channel.1.try_iter()
    }

    /// Return an iterator over recieved messages.
    /// Blocking.
    pub fn recv_blocking(&self) -> impl Iterator<Item = NetworkEvent> + '_ {
        self.shared.inbound_channel.1.iter()
    }

    /// Returns own `PeerId`, whcih can be used by any remote peer to send a message to this one.
    /// None is returned when not connected yet.
    pub fn my_id(&self) -> Option<PeerId> {
        self.shared.my_id.load()
    }

    /// Current state of the peer.
    pub fn state(&self) -> PeerState {
        self.shared.peer_state.load()
    }

    /// Iterate over connected peers, returning ther `PeerId`.
    pub fn iter_peer_ids(&self) -> impl Iterator<Item = PeerId> + '_ {
        self.shared
            .remote_peers
            .iter()
            .map(|item| item.key().to_owned())
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        self.shared
            .keep_alive
            .store(false, std::sync::atomic::Ordering::SeqCst)
    }
}

#[cfg(test)]
mod test {
    use std::{thread, time::Duration};

    use crate::{reactor::Settings, Message, NetworkEvent, Peer, PeerId, Reliability};

    #[test_log::test]
    fn test_peer() {
        let settings = Some(Settings {
            confirm_max_period: Duration::from_millis(100),
            connection_timeout: Duration::from_millis(1000),
            ..Default::default()
        });
        let addr = "127.0.0.1:56001".parse().unwrap();
        let host = Peer::host(addr, settings.clone()).unwrap();
        assert_eq!(host.shared.remote_peers.len(), 1);
        let peer = Peer::connect(addr, settings.clone()).unwrap();
        thread::sleep(Duration::from_millis(100));
        assert_eq!(peer.shared.remote_peers.len(), 2);
        assert_eq!(host.shared.remote_peers.len(), 2);
        let data = vec![128, 51, 32];
        peer.send(PeerId(0), data.clone(), Reliability::Reliable)
            .unwrap();
        thread::sleep(Duration::from_millis(10));
        let host_events: Vec<_> = host.recv().collect();
        assert!(host_events.contains(&NetworkEvent::PeerConnected(PeerId(1))));
        assert!(host_events.contains(&NetworkEvent::Message(Message {
            data,
            src: PeerId(1)
        })));
        let peer_events: Vec<_> = peer.recv().collect();
        assert!(peer_events.contains(&NetworkEvent::PeerConnected(PeerId(0))));
        assert!(peer_events.contains(&NetworkEvent::PeerConnected(PeerId(1))));
        drop(peer);
        thread::sleep(Duration::from_millis(1200));
        assert_eq!(
            host.recv().next(),
            Some(NetworkEvent::PeerDisconnected(PeerId(1)))
        );
        assert_eq!(host.shared.remote_peers.len(), 1);
    }

    #[test_log::test]
    fn test_broadcast() {
        let settings = Some(Settings {
            confirm_max_period: Duration::from_millis(100),
            connection_timeout: Duration::from_millis(1000),
            ..Default::default()
        });
        let addr = "127.0.0.1:56002".parse().unwrap();
        let host = Peer::host(addr, settings.clone()).unwrap();
        assert_eq!(host.shared.remote_peers.len(), 1);
        let peer1 = Peer::connect(addr, settings.clone()).unwrap();
        let peer2 = Peer::connect(addr, settings.clone()).unwrap();
        thread::sleep(Duration::from_millis(10));
        assert_eq!(host.shared.remote_peers.len(), 3);

        let data = vec![123, 112, 51, 23];
        peer1
            .broadcast(data.clone(), Reliability::Reliable)
            .unwrap();
        thread::sleep(Duration::from_millis(10));

        let host_events: Vec<_> = dbg!(host.recv().collect());
        let peer1_events: Vec<_> = dbg!(peer1.recv().collect());
        let peer2_events: Vec<_> = dbg!(peer2.recv().collect());

        assert!(peer2_events.contains(&NetworkEvent::Message(Message {
            src: peer1.my_id().unwrap(),
            data: data.clone(),
        })));
        assert!(!peer1_events.contains(&NetworkEvent::Message(Message {
            src: peer1.my_id().unwrap(),
            data: data.clone(),
        })));
        assert!(host_events.contains(&NetworkEvent::Message(Message {
            src: peer1.my_id().unwrap(),
            data: data.clone(),
        })));
    }

    #[test_log::test]
    fn test_host_has_conn() {
        let settings = Some(Settings {
            confirm_max_period: Duration::from_millis(100),
            connection_timeout: Duration::from_millis(1000),
            ..Default::default()
        });
        let addr = "127.0.0.1:56003".parse().unwrap();
        let host = Peer::host(addr, settings.clone()).unwrap();
        thread::sleep(Duration::from_millis(10));
        assert_eq!(
            host.recv().next(),
            Some(NetworkEvent::PeerConnected(PeerId(0)))
        );
    }
}
