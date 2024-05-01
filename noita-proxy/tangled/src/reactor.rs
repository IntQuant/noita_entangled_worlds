use crate::{
    error::NetError,
    util::{RateLimiter, RingSet},
    Channel, Message, NetworkEvent, OutboundMessage, PeerId, SeqId,
};

use super::{Datagram, PeerState, DATAGRAM_MAX_LEN};
use crossbeam::{
    atomic::AtomicCell,
    channel::{bounded, Receiver, Sender},
    select,
};

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    error::Error,
    io::Cursor,
    net::{SocketAddr, UdpSocket},
    sync::{
        atomic::{AtomicBool, AtomicU16, Ordering::SeqCst},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};
use tracing::{error, info, trace, warn};

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
    /// Default: 1 second.
    pub connection_timeout: Duration,
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

pub(crate) struct Shared {
    pub settings: Settings,
    pub socket: UdpSocket,
    pub inbound_channel: Channel<NetworkEvent>,
    pub outbound_channel: Channel<OutboundMessage>,
    pub keep_alive: AtomicBool,
    pub peer_state: AtomicCell<PeerState>,
    pub remote_peers: DashMap<PeerId, RemotePeer>,
    pub max_packets_per_second: usize,
    pub host_addr: Option<SocketAddr>,
    pub my_id: AtomicCell<Option<PeerId>>,
}

struct DirectPeer {
    addr: SocketAddr,
    outbound_pending: VecDeque<NetMessageVariant>,
    resend_pending: VecDeque<(Instant, NetMessageNormal)>,
    confirmed: RingSet<SeqId>,
    rate_limit: RateLimiter,
    seq_counter: AtomicU16,
    recent_seq: RingSet<SeqId>,
    pending_confirms: VecDeque<SeqId>,
    last_confirm_sent: Instant,
    last_seen: Instant,
}

#[derive(Default)]
pub struct RemotePeer {}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub enum Destination {
    One(PeerId),
    Broadcast,
}

#[derive(Serialize, Deserialize, Clone)]
enum NetMessageVariant {
    Login,
    Normal(NetMessageNormal),
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

#[derive(Serialize, Deserialize, Clone, Debug)]
struct NetMessageNormal {
    // Source that generated sequence id.
    // Initially the same as origin_src, but can be changed when packet is retransmitted not as-is, e. g. when it is broadcasted.
    src: PeerId,
    // Original source.
    origin_src: PeerId,
    dst: Destination,
    seq_id: SeqId,
    reliability: Reliability,
    inner: NetMessageInner,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum NetMessageInner {
    RegDone { addr: SocketAddr },
    AddPeer { id: PeerId },
    DelPeer { id: PeerId },
    Confirm { confirmed_ids: Vec<SeqId> },
    Payload { data: Vec<u8> },
}

impl TryFrom<Datagram> for NetMessageVariant {
    type Error = bincode::Error;

    fn try_from(datagram: Datagram) -> Result<Self, Self::Error> {
        bincode::deserialize(&datagram.data[..datagram.size])
    }
}

impl TryFrom<&NetMessageVariant> for Datagram {
    type Error = bincode::Error;

    fn try_from(value: &NetMessageVariant) -> Result<Self, Self::Error> {
        let mut data = Cursor::new([0; DATAGRAM_MAX_LEN]);
        bincode::serialize_into(&mut data, value)?;
        let data = data.into_inner();
        Ok(Datagram {
            data,
            size: data.len(),
        })
    }
}

pub(crate) struct Reactor {
    shared: Arc<Shared>,
    direct_peers: HashMap<PeerId, DirectPeer>,
}

type AddrDatagram = (SocketAddr, Datagram);

impl Reactor {
    fn add_peer(&self, id: PeerId) -> Result<(), NetError> {
        self.shared.remote_peers.insert(id, RemotePeer::default());
        self.shared
            .inbound_channel
            .0
            .send(NetworkEvent::PeerConnected(id))?;
        Ok(())
    }

    fn direct_broadcast(
        &mut self,
        src_id: PeerId,
        msg: NetMessageInner,
        reliability: Reliability,
    ) -> Result<(), NetError> {
        for (&peer_id, peer) in self.direct_peers.iter_mut() {
            let new_seq_id = peer.seq_counter.fetch_add(1, SeqCst);
            let new_msg = Self::wrap_packet_seq_id(
                src_id,
                src_id,
                new_seq_id,
                Destination::One(peer_id),
                msg.clone(),
                reliability,
            )?;
            Self::direct_send_peer(peer, new_msg)?;
        }
        Ok(())
    }

    fn direct_send(&mut self, id: PeerId, msg: NetMessageVariant) -> Result<(), NetError> {
        let peer = self
            .direct_peers
            .get_mut(&id)
            .ok_or(NetError::UnknownPeer)?;
        Self::direct_send_peer(peer, msg)
    }

    fn direct_send_peer(peer: &mut DirectPeer, msg: NetMessageVariant) -> Result<(), NetError> {
        peer.outbound_pending.push_back(msg);
        Ok(())
    }

    fn gen_peer_id(&mut self) -> Option<PeerId> {
        (1..=u16::MAX)
            .map(PeerId)
            .find(|i| !self.shared.remote_peers.contains_key(i))
    }

    fn handle_inbound(&mut self, (incoming_addr, msg_raw): AddrDatagram) {
        let msg = match NetMessageVariant::try_from(msg_raw) {
            Ok(msg) => msg,
            Err(err) => {
                warn!("Error when converting to NetMessage: {}", err);
                return;
            }
        };
        match self.shared.my_id.load() {
            Some(id) => {
                match msg {
                    NetMessageVariant::Login => {
                        if self.is_host() {
                            //TODO check this addr is not already registered
                            match self.gen_peer_id() {
                                Some(new_id) => {
                                    self.add_peer(new_id).ok();
                                    let mut peer = DirectPeer::new(
                                        incoming_addr,
                                        self.shared.max_packets_per_second,
                                    );
                                    peer.outbound_pending.push_back(NetMessageVariant::Normal(
                                        NetMessageNormal {
                                            src: id,
                                            origin_src: id,
                                            dst: Destination::One(new_id),
                                            seq_id: u16::MAX,
                                            inner: NetMessageInner::RegDone {
                                                addr: incoming_addr,
                                            },
                                            reliability: Reliability::Reliable,
                                        },
                                    ));
                                    self.direct_peers.insert(new_id, peer);
                                    self.direct_broadcast(
                                        id,
                                        NetMessageInner::AddPeer { id: new_id },
                                        Reliability::Reliable,
                                    )
                                    .ok();
                                    let shared = self.shared.clone();
                                    for re in shared.remote_peers.iter() {
                                        let id = *re.key();
                                        if id != new_id {
                                            self.wrap_packet(
                                                id,
                                                Destination::One(new_id),
                                                NetMessageInner::AddPeer { id },
                                                Reliability::Reliable,
                                            )
                                            .and_then(|msg| self.direct_send(new_id, msg))
                                            .ok();
                                        }
                                    }
                                }
                                None => warn!("Out of ids"),
                            }
                        } else {
                            warn!("Not a host, registration attempt ignored");
                        }
                    }
                    NetMessageVariant::Normal(msg) => {
                        match self.handle_inbound_normal(msg, incoming_addr, id) {
                            Ok(_) => {}
                            Err(NetError::Dropped) => {}
                            Err(err) => {
                                info!("Error while handling normal inbound message: {}", err)
                            }
                        }
                    }
                }
            }
            None => match msg {
                NetMessageVariant::Normal(NetMessageNormal {
                    inner: NetMessageInner::RegDone { addr: _ },
                    dst,
                    src,
                    ..
                }) => {
                    let expected_host_addr = self
                        .shared
                        .host_addr
                        .expect("Can't have both my_id and host_addr be None");
                    if incoming_addr == expected_host_addr && src == PeerId(0) {
                        if let Destination::One(id) = dst {
                            self.shared.my_id.store(Some(id));
                            self.add_peer(PeerId(0)).ok();
                            self.shared.peer_state.store(PeerState::Connected);
                        } else {
                            warn!("Malformed registration message");
                        }
                    } else {
                        warn!("Registration message recieved not from the right address ({}, {} expected)", incoming_addr, expected_host_addr);
                    }
                }
                _ => warn!("Message ignored as registration is not done yet"),
            },
        }
    }

    fn handle_inbound_normal(
        &mut self,
        msg: NetMessageNormal,
        _incoming_addr: SocketAddr,
        my_id: PeerId,
    ) -> Result<(), NetError> {
        let peer = self.direct_peers.get_mut(&msg.src);
        if peer
            .as_ref()
            .map_or(true, |peer| peer.recent_seq.contains(&msg.seq_id))
        {
            return Err(NetError::Dropped);
        }
        {
            let peer = peer.expect("Expected to exist");
            peer.recent_seq.add(msg.seq_id); //TODO backpressure
            peer.pending_confirms.push_back(msg.seq_id);
            peer.last_seen = Instant::now()
        }

        if Destination::One(my_id) == msg.dst || msg.dst == Destination::Broadcast {
            // TODO eliminate this clone
            match msg.inner.clone() {
                NetMessageInner::RegDone { addr: _ } => {
                    warn!("Already registered, request ignored");
                }
                NetMessageInner::AddPeer { id } => {
                    if !self.is_host() {
                        self.add_peer(id).ok();
                        info!("Peer {} added", id);
                    }
                }
                NetMessageInner::DelPeer { id } => {
                    if !self.is_host() {
                        self.del_peer(id).ok();
                        info!("Peer {} removed", id);
                    }
                }
                NetMessageInner::Confirm { confirmed_ids } => {
                    if let Some(peer) = self.direct_peers.get_mut(&msg.src) {
                        for id in confirmed_ids {
                            peer.confirmed.add(id);
                        }
                    }
                }
                NetMessageInner::Payload { data } => {
                    self.shared
                        .inbound_channel
                        .0
                        .send(NetworkEvent::Message(Message {
                            src: msg.origin_src,
                            data,
                        }))?;
                }
            }
        }
        if self.is_host() && Destination::One(my_id) != msg.dst {
            match msg.dst {
                Destination::One(dst) => {
                    let new_msg =
                        self.wrap_packet(dst, Destination::One(dst), msg.inner, msg.reliability)?;
                    self.direct_send(dst, new_msg)?;
                }
                Destination::Broadcast => {
                    let mut buf = Vec::new();
                    for peer in &self.direct_peers {
                        if *peer.0 == msg.src {
                            continue;
                        }
                        let seq_id = self.next_seq_id_for_peer(*peer.0)?;
                        if let Ok(wrapped_msg) = Self::wrap_packet_seq_id(
                            PeerId(0),
                            msg.origin_src,
                            seq_id,
                            Destination::One(*peer.0),
                            msg.inner.clone(),
                            msg.reliability,
                        ) {
                            buf.push((*peer.0, wrapped_msg));
                        }
                    }
                    for (peer_id, wrapped_msg) in buf {
                        self.direct_send(peer_id, wrapped_msg).ok();
                    }
                }
            }
        }

        Ok(())
    }

    fn del_peer(&mut self, id: PeerId) -> Result<(), NetError> {
        self.shared.remote_peers.remove(&id);
        self.shared
            .inbound_channel
            .0
            .send(NetworkEvent::PeerDisconnected(id))?;
        Ok(())
    }

    fn handle_outbound(&mut self, msg: OutboundMessage) -> Result<(), NetError> {
        let dst = msg.dst;
        if self.is_host() {
            match dst {
                Destination::One(id) => {
                    let net_msg = self.wrap_packet(
                        id,
                        dst,
                        NetMessageInner::Payload { data: msg.data },
                        msg.reliability,
                    )?;
                    self.direct_send(id, net_msg)?;
                }
                Destination::Broadcast => self.direct_broadcast(
                    PeerId(0),
                    NetMessageInner::Payload { data: msg.data },
                    msg.reliability,
                )?,
            }
        } else {
            let net_msg = self.wrap_packet(
                PeerId(0),
                dst,
                NetMessageInner::Payload { data: msg.data },
                msg.reliability,
            )?;
            self.direct_send(PeerId(0), net_msg)?;
        }
        Ok(())
    }

    pub fn is_host(&self) -> bool {
        self.shared.host_addr.is_none()
    }

    pub fn next_seq_id_for_peer(&self, peer_id: PeerId) -> Result<SeqId, NetError> {
        Ok(self
            .direct_peers
            .get(&peer_id)
            .or_else(|| {
                if !self.is_host() {
                    self.direct_peers.get(&PeerId(0))
                } else {
                    None
                }
            })
            .ok_or(NetError::UnknownPeer)?
            .seq_counter
            .fetch_add(1, SeqCst))
    }

    fn run(mut self, inbound_r: Receiver<AddrDatagram>) -> Result<(), Box<dyn Error>> {
        while self.shared.keep_alive.load(SeqCst) {
            select! {
                recv(inbound_r) -> addr_msg => self.handle_inbound(addr_msg?),
                recv(self.shared.outbound_channel.1) -> msg => {self.handle_outbound(msg?).ok();}
                default => {thread::sleep(Duration::from_micros(100));}
            }
            let mut dc = Vec::new();
            self.direct_peers.retain(|&k, v| {
                let stays = v.last_seen.elapsed() < self.shared.settings.connection_timeout;
                if !stays {
                    dc.push(k);
                }
                stays
            });
            if self.is_host() {
                for peer_id in dc {
                    let src_id = self.shared.my_id.load().unwrap(); // Should always be PeerId(0)
                    assert_eq!(src_id, PeerId(0));
                    self.direct_broadcast(
                        src_id,
                        NetMessageInner::DelPeer { id: peer_id },
                        Reliability::Reliable,
                    )?;
                    self.del_peer(peer_id).ok();
                    info!("[Host] Peer {} removed", peer_id);
                }
            }
            if !self.is_host() && self.direct_peers.is_empty() {
                self.shared.peer_state.store(PeerState::Disconnected);
                self.shared.keep_alive.store(false, SeqCst);
            }
            'peers: for (&id, peer) in self.direct_peers.iter_mut() {
                let resend_in = Instant::now() + Duration::from_secs(1);

                if let Some(my_id) = self.shared.my_id.load() {
                    if peer.last_confirm_sent.elapsed() > self.shared.settings.confirm_max_period
                        || peer.pending_confirms.len()
                            > self.shared.settings.confirm_max_per_message
                    {
                        peer.last_confirm_sent = Instant::now();
                        let max_per_message = self.shared.settings.confirm_max_per_message;
                        let mut confirmed_ids = Vec::with_capacity(max_per_message);
                        while let Some(confirm) = peer.pending_confirms.pop_front() {
                            confirmed_ids.push(confirm);
                            if confirmed_ids.len() == max_per_message {
                                break;
                            }
                        }
                        peer.resend_pending.push_front((
                            Instant::now(),
                            NetMessageNormal {
                                src: my_id,
                                origin_src: my_id,
                                dst: Destination::One(id),
                                seq_id: peer.seq_counter.fetch_add(1, SeqCst),
                                reliability: Reliability::Reliable,
                                inner: NetMessageInner::Confirm { confirmed_ids },
                            },
                        ))
                    }
                }

                while peer
                    .resend_pending
                    .front()
                    .map_or(false, |x| x.0 < Instant::now())
                {
                    let (moment, msg) = peer
                        .resend_pending
                        .pop_front()
                        .expect("Checked that deque is not empty");

                    if !peer.confirmed.contains(&msg.seq_id) {
                        if !peer.rate_limit.get_token() {
                            peer.resend_pending.push_front((moment, msg));
                            continue 'peers;
                        }
                        peer.resend_pending.push_back((resend_in, msg.clone()));
                        trace!("Sent {:?} to {}", msg, peer.addr);
                        let datagram = Datagram::try_from(&NetMessageVariant::Normal(msg)).unwrap();
                        self.shared
                            .socket
                            .send_to(&datagram.data[..datagram.size], peer.addr)
                            .expect("Could not send");
                    }
                }

                while !peer.outbound_pending.is_empty() && peer.rate_limit.get_token() {
                    let msg = peer
                        .outbound_pending
                        .pop_front()
                        .expect("Checked that deque is not empty");
                    if let NetMessageVariant::Normal(ref msg) = msg {
                        if msg.reliability == Reliability::Reliable {
                            peer.resend_pending.push_back((resend_in, msg.clone()));
                        }
                    }
                    let datagram = Datagram::try_from(&msg).unwrap();
                    self.shared
                        .socket
                        .send_to(&datagram.data[..datagram.size], peer.addr)
                        .expect("Could not send");
                }
            }
        }
        Ok(())
    }

    fn run_pipe(
        shared: Arc<Shared>,
        sender: Sender<(SocketAddr, Datagram)>,
    ) -> Result<(), Box<dyn Error>> {
        while shared.keep_alive.load(SeqCst) {
            let mut buf = [0u8; DATAGRAM_MAX_LEN];
            match shared.socket.recv_from(&mut buf) {
                Ok((len, addr)) => sender
                    .send((
                        addr,
                        Datagram {
                            size: len,
                            data: buf,
                        },
                    ))
                    .map_err(Box::new)?,
                //Err(err)
                //    if err.kind() == ErrorKind::WouldBlock || err.kind() == ErrorKind::TimedOut => {
                //}
                Err(err) => return Err(Box::new(err)),
            }
        }
        Ok(())
    }

    pub(crate) fn start(shared: Arc<Shared>) {
        let mut me = Reactor {
            shared,
            direct_peers: Default::default(),
        };
        if !me.is_host() {
            me.direct_peers.insert(
                PeerId(0),
                DirectPeer::new(
                    me.shared
                        .host_addr
                        .expect("Can't be a client without a host addr"),
                    me.shared.max_packets_per_second,
                ),
            );
            me.direct_send(PeerId(0), NetMessageVariant::Login).unwrap();
        }
        if me.is_host() {
            me.shared.peer_state.store(PeerState::Connected);
        }
        let shared_c = Arc::clone(&me.shared);
        let (inbound_s, inbound_r) = bounded(16);
        thread::spawn(move || {
            let shared_c_2 = Arc::clone(&shared_c);
            if let Err(err) = Self::run_pipe(shared_c_2, inbound_s) {
                shared_c.keep_alive.store(false, SeqCst);
                shared_c.peer_state.store(PeerState::Disconnected);
                error!("Reactor pipe error: {}", err);
            }
        });
        let shared_c = Arc::clone(&me.shared);
        thread::spawn(move || {
            if let Err(err) = me.run(inbound_r) {
                shared_c.keep_alive.store(false, SeqCst);
                shared_c.peer_state.store(PeerState::Disconnected);
                error!("Reactor error: {}", err);
            }
        });
    }

    fn wrap_packet(
        &self,
        id: PeerId,
        dst: Destination,
        msg: NetMessageInner,
        reliability: Reliability,
    ) -> Result<NetMessageVariant, NetError> {
        let seq_id = self.next_seq_id_for_peer(id)?;
        let src = self.shared.my_id.load().expect("Should know own id by now");
        Self::wrap_packet_seq_id(src, src, seq_id, dst, msg, reliability)
    }

    fn wrap_packet_seq_id(
        src: PeerId,
        origin_src: PeerId,
        seq_id: SeqId,
        dst: Destination,
        msg: NetMessageInner,
        reliability: Reliability,
    ) -> Result<NetMessageVariant, NetError> {
        Ok(NetMessageVariant::Normal(NetMessageNormal {
            src,
            origin_src,
            dst,
            seq_id,
            inner: msg,
            reliability,
        }))
    }
}

impl DirectPeer {
    fn new(incoming_addr: SocketAddr, rate_limit: usize) -> DirectPeer {
        let now = Instant::now();
        DirectPeer {
            addr: incoming_addr,
            outbound_pending: Default::default(),
            resend_pending: Default::default(),
            confirmed: RingSet::new(1024),
            rate_limit: RateLimiter::new(rate_limit, Duration::from_secs(1)),
            seq_counter: AtomicU16::new(0),
            recent_seq: RingSet::new(1024),
            pending_confirms: VecDeque::new(),
            last_confirm_sent: now,
            last_seen: now,
        }
    }
}
