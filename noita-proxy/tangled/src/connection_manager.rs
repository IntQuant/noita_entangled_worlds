use std::{
    io,
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use bitcode::{Decode, Encode};
use crossbeam::{
    atomic::AtomicCell,
    channel::{Receiver, Sender, unbounded},
};
use dashmap::DashMap;
use quinn::{
    ClientConfig, ConnectError, Connecting, ConnectionError, Endpoint, Incoming, RecvStream,
    ServerConfig, TransportConfig,
    crypto::rustls::QuicClientConfig,
    rustls::{
        self,
        pki_types::{CertificateDer, PrivatePkcs8KeyDer},
    },
};
use socket2::{Domain, Socket, Type};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, error, info, trace, warn};

use crate::{
    common::{Destination, NetworkEvent, PeerId, PeerState, Reliability, Settings},
    helpers::SkipServerVerification,
};

mod message_stream;

#[derive(Debug, Encode, Decode)]
enum InternalMessage {
    Normal(OutboundMessage),
    RemoteConnected(PeerId),
    RemoteDisconnected(PeerId),
}

#[derive(Default)]
pub(crate) struct RemotePeer;

#[derive(Debug, Error)]
enum DirectConnectionError {
    #[error("QUIC Connection error: {0}")]
    QUICConnectionError(#[from] ConnectionError),
    #[error("Initial exchange failed")]
    InitialExchangeFailed,
    #[error("Message read failed")]
    MessageIoFailed,
    #[error("Failed to decode message")]
    DecodeError,
}

struct DirectPeer {
    my_id: PeerId,
    remote_id: PeerId,
    send_stream: message_stream::SendMessageStream<InternalMessage>,
}

impl DirectPeer {
    async fn recv_task(shared: Arc<Shared>, recv_stream: RecvStream, remote_id: PeerId) {
        let mut recv_stream = message_stream::RecvMessageStream::new(recv_stream);
        while let Ok(msg) = recv_stream.recv().await {
            trace!("Received message from {remote_id}");
            if let Err(err) = shared
                .internal_incoming_messages_s
                .send((remote_id, msg))
                .await
            {
                warn!("Could not send message to channel: {err}. Stopping.");
                break;
            }
        }
        shared
            .internal_events_s
            .send(InternalEvent::Disconnected(remote_id))
            .ok();
    }

    async fn accept(
        shared: Arc<Shared>,
        incoming: Incoming,
        assigned_peer_id: PeerId,
    ) -> Result<Self, DirectConnectionError> {
        let connection = incoming
            .await
            .inspect_err(|err| warn!("Failed to accept connection: {err}"))?;

        let mut sender = connection
            .open_uni()
            .await
            .inspect_err(|err| warn!("Failed to get send stream: {err}"))?;
        sender
            .write_u16(assigned_peer_id.0)
            .await
            .map_err(|_err| DirectConnectionError::InitialExchangeFailed)?;

        let (send_stream, recv_stream) = connection.open_bi().await?;
        tokio::spawn(Self::recv_task(shared, recv_stream, assigned_peer_id));
        debug!("Server: spawned recv task");

        Ok(Self {
            my_id: PeerId::HOST,
            remote_id: assigned_peer_id,
            send_stream: message_stream::SendMessageStream::new(send_stream),
        })
    }

    async fn connect(
        shared: Arc<Shared>,
        connection: Connecting,
    ) -> Result<Self, DirectConnectionError> {
        let connection = connection
            .await
            .inspect_err(|err| warn!("Failed to initiate connection: {err}"))?;

        let mut receiver = connection.accept_uni().await?;
        let peer_id = receiver
            .read_u16()
            .await
            .map_err(|_err| DirectConnectionError::InitialExchangeFailed)?;
        debug!("Got peer id {peer_id}");

        let (send_stream, recv_stream) = connection.accept_bi().await?;
        tokio::spawn(Self::recv_task(shared, recv_stream, PeerId::HOST));
        debug!("Client: spawned recv task");

        Ok(Self {
            my_id: PeerId(peer_id),
            remote_id: PeerId::HOST,
            send_stream: message_stream::SendMessageStream::new(send_stream),
        })
    }
}

#[derive(Debug, Encode, Decode, Clone)]
pub(crate) struct OutboundMessage {
    pub src: PeerId,
    pub dst: Destination,
    pub reliability: Reliability,
    pub data: Vec<u8>,
}

pub(crate) type Channel<T> = (Sender<T>, Receiver<T>);

#[derive(Debug, Error)]
pub enum TangledInitError {
    #[error("Could not create endpoint.\nReason: {0}")]
    CouldNotCreateEndpoint(io::Error),
    #[error("Could not connect to host.\nReason: {0}")]
    CouldNotConnectToHost(ConnectError),
    #[error("Async runtime not found")]
    NoRuntimeFound,
}

enum InternalEvent {
    Connected(PeerId),
    Disconnected(PeerId),
}

pub(crate) struct Shared {
    pub inbound_channel: Channel<NetworkEvent>,
    pub outbound_messages_s: tokio::sync::mpsc::UnboundedSender<OutboundMessage>,
    pub keep_alive: AtomicBool,
    pub peer_state: AtomicCell<PeerState>,
    pub remote_peers: DashMap<PeerId, RemotePeer>,
    pub host_addr: Option<SocketAddr>,
    pub my_id: AtomicCell<Option<PeerId>>,
    // ConnectionManager-specific stuff
    direct_peers: DashMap<PeerId, DirectPeer>,
    internal_incoming_messages_s: tokio::sync::mpsc::Sender<(PeerId, InternalMessage)>,
    internal_events_s: tokio::sync::mpsc::UnboundedSender<InternalEvent>,
}

pub(crate) struct ConnectionManager {
    shared: Arc<Shared>,
    endpoint: Endpoint,
    host_conn: Option<DirectPeer>,
    is_server: bool,
    incoming_messages_r: tokio::sync::mpsc::Receiver<(PeerId, InternalMessage)>,
    outbound_messages_r: tokio::sync::mpsc::UnboundedReceiver<OutboundMessage>,
    internal_events_r: tokio::sync::mpsc::UnboundedReceiver<InternalEvent>,
}

impl ConnectionManager {
    pub(crate) fn new(
        host_addr: Option<SocketAddr>,
        _settings: Option<Settings>,
        bind_addr: SocketAddr,
    ) -> Result<Self, TangledInitError> {
        let is_server = host_addr.is_none();

        let (internal_incoming_messages_s, incoming_messages_r) = tokio::sync::mpsc::channel(512);
        let (outbound_messages_s, outbound_messages_r) = tokio::sync::mpsc::unbounded_channel();
        let (internal_events_s, internal_events_r) = tokio::sync::mpsc::unbounded_channel();

        let shared = Arc::new(Shared {
            inbound_channel: unbounded(),
            outbound_messages_s,
            keep_alive: AtomicBool::new(true),
            host_addr,
            peer_state: Default::default(),
            remote_peers: Default::default(),
            my_id: AtomicCell::new(is_server.then_some(PeerId(0))),
            direct_peers: DashMap::default(),
            internal_incoming_messages_s,
            internal_events_s,
        });

        let server_config = default_server_config();

        let mut endpoint = if is_server {
            // Endpoint::server(config, bind_addr).map_err(TangledInitError::CouldNotCreateEndpoint)?
            let socket = Socket::new(Domain::for_address(bind_addr), Type::DGRAM, None)
                .map_err(TangledInitError::CouldNotCreateEndpoint)?;
            if bind_addr.is_ipv6() {
                if let Err(err) = socket.set_only_v6(false) {
                    warn!("Failed to set socket to be not only v6: {}", err);
                } else {
                    info!("Enabled dualstack mode for socket");
                };
            }
            socket
                .bind(&bind_addr.into())
                .map_err(TangledInitError::CouldNotCreateEndpoint)?;

            let runtime = quinn::default_runtime().ok_or(TangledInitError::NoRuntimeFound)?;
            Endpoint::new_with_abstract_socket(
                Default::default(),
                Some(server_config),
                runtime
                    .wrap_udp_socket(socket.into())
                    .map_err(TangledInitError::CouldNotCreateEndpoint)?,
                runtime,
            )
            .map_err(TangledInitError::CouldNotCreateEndpoint)?
        } else {
            Endpoint::client(bind_addr).map_err(TangledInitError::CouldNotCreateEndpoint)?
        };

        endpoint.set_default_client_config(ClientConfig::new(Arc::new(
            QuicClientConfig::try_from(
                rustls::ClientConfig::builder()
                    .dangerous()
                    .with_custom_certificate_verifier(SkipServerVerification::new())
                    .with_no_client_auth(),
            )
            .unwrap(),
        )));

        Ok(Self {
            shared,
            is_server,
            endpoint,
            host_conn: None,
            incoming_messages_r,
            outbound_messages_r,
            internal_events_r,
        })
    }

    async fn accept_connections(shared: Arc<Shared>, endpoint: Endpoint) {
        let mut peer_id_counter = 1;
        while shared.keep_alive.load(Ordering::Relaxed) {
            let Some(incoming) = endpoint.accept().await else {
                debug!("Endpoint closed, stopping connection accepter task.");
                return;
            };
            match DirectPeer::accept(shared.clone(), incoming, PeerId(peer_id_counter)).await {
                Ok(direct_peer) => {
                    shared
                        .direct_peers
                        .insert(PeerId(peer_id_counter), direct_peer);
                    shared
                        .internal_events_s
                        .send(InternalEvent::Connected(PeerId(peer_id_counter)))
                        .expect("channel to be open");
                    peer_id_counter += 1;
                }
                Err(err) => {
                    warn!("Failed to accept connection: {err}")
                }
            };
        }
    }

    async fn handle_incoming_message(&mut self, msg: InternalMessage) {
        match msg {
            InternalMessage::Normal(msg) => {
                let intended_for_me = self
                    .shared
                    .my_id
                    .load()
                    .map(|my_id| msg.dst == Destination::One(my_id))
                    .unwrap_or(false);
                if self.is_server && !intended_for_me && !msg.dst.is_broadcast() {
                    self.server_send_internal_message(
                        msg.dst.to_one().unwrap(),
                        &InternalMessage::Normal(msg),
                    )
                    .await;
                    return;
                }
                if self.is_server && msg.dst.is_broadcast() {
                    self.server_send_to_peers(msg.clone()).await;
                }
                if msg.dst.is_broadcast() || intended_for_me {
                    self.shared
                        .inbound_channel
                        .0
                        .send(NetworkEvent::Message(crate::Message {
                            src: msg.src,
                            data: msg.data,
                        }))
                        .expect("channel to be open");
                }
            }
            InternalMessage::RemoteConnected(peer_id) => {
                debug!("Got notified of peer {peer_id}");
                self.shared
                    .internal_events_s
                    .send(InternalEvent::Connected(peer_id))
                    .expect("channel to be open");
            }
            InternalMessage::RemoteDisconnected(peer_id) => self
                .shared
                .internal_events_s
                .send(InternalEvent::Disconnected(peer_id))
                .expect("channel to be open"),
        }
    }

    async fn handle_internal_event(&mut self, ev: InternalEvent) {
        match ev {
            InternalEvent::Connected(peer_id) => {
                if self.shared.remote_peers.contains_key(&peer_id) {
                    // Already connected, no need to emit an event.
                    return;
                }
                self.shared
                    .inbound_channel
                    .0
                    .send(NetworkEvent::PeerConnected(peer_id))
                    .expect("channel to be open");
                self.shared.remote_peers.insert(peer_id, RemotePeer);
                debug!(
                    "Peer {} connected, total connected: {}",
                    peer_id,
                    self.shared.remote_peers.len()
                );
                if self.is_server {
                    self.server_broadcast_internal_message(
                        PeerId::HOST,
                        InternalMessage::RemoteConnected(peer_id),
                    )
                    .await;

                    let peers = self
                        .shared
                        .remote_peers
                        .iter()
                        .map(|i| *i.key())
                        .collect::<Vec<_>>();
                    for conn_peer in peers {
                        debug!("Notifying peer of {conn_peer}");
                        self.server_send_internal_message(
                            peer_id,
                            &InternalMessage::RemoteConnected(conn_peer),
                        )
                        .await;
                    }
                }
            }
            InternalEvent::Disconnected(peer_id) => {
                debug!("Peer {} disconnected", peer_id);
                self.shared.direct_peers.remove(&peer_id);
                self.shared
                    .inbound_channel
                    .0
                    .send(NetworkEvent::PeerDisconnected(peer_id))
                    .expect("channel to be open");
                self.shared.remote_peers.remove(&peer_id);
                if self.is_server {
                    self.server_broadcast_internal_message(
                        PeerId::HOST,
                        InternalMessage::RemoteDisconnected(peer_id),
                    )
                    .await;
                }
            }
        }
    }

    async fn server_send_to_peers(&mut self, msg: OutboundMessage) {
        match msg.dst {
            Destination::One(peer_id) => {
                self.server_send_internal_message(peer_id, &InternalMessage::Normal(msg))
                    .await;
            }
            Destination::Broadcast => {
                let msg_src = msg.src;
                let value = InternalMessage::Normal(msg);
                self.server_broadcast_internal_message(msg_src, value).await;
            }
        }
    }

    async fn server_send_internal_message(&mut self, peer_id: PeerId, msg: &InternalMessage) {
        let peer = self.shared.direct_peers.get_mut(&peer_id);
        // TODO handle lack of peer?
        if let Some(mut peer) = peer {
            // TODO handle errors
            peer.send_stream.send(msg).await.ok();
        }
    }

    async fn server_broadcast_internal_message(
        &mut self,
        excluded: PeerId,
        value: InternalMessage,
    ) {
        for mut peer in self.shared.direct_peers.iter_mut() {
            let peer_id = *peer.key();
            if peer_id != excluded {
                // TODO handle errors
                peer.send_stream.send(&value).await.ok();
            }
        }
    }

    async fn astart(mut self, host_conn: Option<Connecting>) {
        debug!("astart running");
        if let Some(host_conn) = host_conn {
            match DirectPeer::connect(self.shared.clone(), host_conn).await {
                Ok(host_conn) => {
                    self.shared.my_id.store(Some(host_conn.my_id));
                    self.shared
                        .internal_events_s
                        .send(InternalEvent::Connected(host_conn.remote_id))
                        .expect("channel to be open");
                    self.host_conn = Some(host_conn);
                    self.shared.peer_state.store(PeerState::Connected);
                }
                Err(err) => {
                    error!("Could not connect to host: {}", err);
                    self.shared.peer_state.store(PeerState::Disconnected);
                    return;
                }
            }
        }
        if self.is_server {
            let endpoint = self.endpoint.clone();
            tokio::spawn(Self::accept_connections(self.shared.clone(), endpoint));
            debug!("Started connection acceptor task");
        }

        while self.shared.keep_alive.load(Ordering::Relaxed) {
            tokio::select! {
                msg = self.incoming_messages_r.recv() => {
                    let msg = msg.expect("channel to not be closed");
                    self.handle_incoming_message(msg.1).await;
                }
                msg = self.outbound_messages_r.recv() => {
                    let msg = msg.expect("channel to not be closed");
                    if self.is_server {
                        self.server_send_to_peers(msg).await;
                    } else {
                        // TODO handle error
                        self.host_conn.as_mut().unwrap().send_stream.send(&InternalMessage::Normal(msg)).await.ok();
                    }
                }
                ev = self.internal_events_r.recv() => {
                    let ev = ev.expect("channel to not be closed");
                    self.handle_internal_event(ev).await;
                }
                // Check if we need to stop periodically.
                _ = tokio::time::sleep(Duration::from_millis(1000)) => {}
            }
        }

        debug!("Closing endpoint");
        self.endpoint
            .close(0u32.into(), b"peer decided to disconnect");
    }

    pub(crate) fn start(self) -> Result<(), TangledInitError> {
        let host_conn = self
            .shared
            .host_addr
            .as_ref()
            .map(|host_addr| {
                self.endpoint
                    .connect(*host_addr, "tangled")
                    .map_err(TangledInitError::CouldNotConnectToHost)
            })
            .transpose()?;

        debug!("Spawning astart task");
        tokio::spawn(self.astart(host_conn));
        Ok(())
    }

    pub(crate) fn shared(&self) -> Arc<Shared> {
        self.shared.clone()
    }
}

fn default_server_config() -> ServerConfig {
    let cert = rcgen::generate_simple_self_signed(vec!["tangled".into()]).unwrap();
    let cert_der = CertificateDer::from(cert.cert);
    let priv_key = PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der());

    let mut config =
        ServerConfig::with_single_cert(vec![cert_der.clone()], priv_key.into()).unwrap();
    let mut transport_config = TransportConfig::default();
    transport_config.keep_alive_interval(Some(Duration::from_secs(10)));
    config.transport_config(Arc::new(transport_config));
    config
}
