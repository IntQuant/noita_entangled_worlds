use std::{
    io,
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crossbeam::{
    atomic::AtomicCell,
    channel::{unbounded, Receiver, Sender},
};
use dashmap::DashMap;
use quinn::{
    crypto::rustls::QuicClientConfig,
    rustls::{
        self,
        pki_types::{CertificateDer, PrivatePkcs8KeyDer},
    },
    ClientConfig, ConnectError, Connecting, ConnectionError, Endpoint, Incoming, RecvStream,
    ServerConfig,
};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{error, info, warn};

use crate::{
    common::{Destination, NetworkEvent, PeerId, PeerState, Reliability, Settings},
    helpers::SkipServerVerification,
};

mod message_stream;

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
}

struct DirectPeer {
    my_id: PeerId,
    remote_id: PeerId,
    send_stream: message_stream::SendMessageStream,
}

impl DirectPeer {
    async fn recv_task(shared: Arc<Shared>, recv_stream: RecvStream, remote_id: PeerId) {
        let mut recv_stream = message_stream::RecvMessageStream::new(recv_stream);
        while let Ok(msg) = recv_stream.recv().await {
            if let Err(err) = shared.incoming_messages.0.send((remote_id, msg)) {
                warn!("Could not send message to channel: {err}. Stopping.");
                break;
            }
        }
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

        let (send_stream, recv_stream) = connection.open_bi().await?;
        tokio::spawn(Self::recv_task(shared, recv_stream, PeerId::HOST));

        Ok(Self {
            my_id: PeerId(peer_id),
            remote_id: PeerId::HOST,
            send_stream: message_stream::SendMessageStream::new(send_stream),
        })
    }
}

pub(crate) struct OutboundMessage {
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
}

pub(crate) struct Shared {
    pub settings: Settings,
    pub inbound_channel: Channel<NetworkEvent>,
    pub outbound_channel: Channel<OutboundMessage>,
    pub keep_alive: AtomicBool,
    pub peer_state: AtomicCell<PeerState>,
    pub remote_peers: DashMap<PeerId, RemotePeer>,
    pub host_addr: Option<SocketAddr>,
    pub my_id: AtomicCell<Option<PeerId>>,
    // ConnectionManager-specific stuff
    direct_peers: DashMap<PeerId, DirectPeer>,
    incoming_messages: Channel<(PeerId, Vec<u8>)>,
}

impl Shared {
    pub(crate) fn new(host_addr: Option<SocketAddr>, settings: Option<Settings>) -> Self {
        Self {
            inbound_channel: unbounded(),
            outbound_channel: unbounded(),
            keep_alive: AtomicBool::new(true),
            host_addr,
            peer_state: Default::default(),
            remote_peers: Default::default(),
            my_id: AtomicCell::new(if host_addr.is_none() {
                Some(PeerId(0))
            } else {
                None
            }),
            settings: settings.unwrap_or_default(),
            direct_peers: DashMap::default(),
            incoming_messages: unbounded(),
        }
    }
}

pub(crate) struct ConnectionManager {
    shared: Arc<Shared>,
    endpoint: Endpoint,
    host_conn: Option<DirectPeer>,
    is_server: bool,
}

impl ConnectionManager {
    pub(crate) fn new(shared: Arc<Shared>, addr: SocketAddr) -> Result<Self, TangledInitError> {
        let is_server = shared.host_addr.is_none();

        let config = default_server_config();

        let mut endpoint = if is_server {
            Endpoint::server(config, addr).map_err(TangledInitError::CouldNotCreateEndpoint)?
        } else {
            Endpoint::client(addr).map_err(TangledInitError::CouldNotCreateEndpoint)?
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
        })
    }

    async fn accept_connections(shared: Arc<Shared>, endpoint: Endpoint) {
        let mut peer_id_counter = 1;
        while shared.keep_alive.load(Ordering::Relaxed) {
            let Some(incoming) = endpoint.accept().await else {
                info!("Endpoint closed, stopping connection accepter task.");
                return;
            };
            match DirectPeer::accept(shared.clone(), incoming, PeerId(peer_id_counter)).await {
                Ok(direct_peer) => {
                    shared
                        .direct_peers
                        .insert(PeerId(peer_id_counter), direct_peer);
                    peer_id_counter += 1;
                }
                Err(err) => {
                    warn!("Failed to accept connection: {err}")
                }
            };
        }
    }

    async fn astart(mut self, host_conn: Option<Connecting>) {
        if let Some(host_conn) = host_conn {
            match DirectPeer::connect(self.shared.clone(), host_conn).await {
                Ok(host_conn) => {
                    self.host_conn = Some(host_conn);
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
            info!("Started connection acceptor task");
        }
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

        tokio::spawn(self.astart(host_conn));
        Ok(())
    }
}

fn default_server_config() -> ServerConfig {
    let cert = rcgen::generate_simple_self_signed(vec!["tangled".into()]).unwrap();
    let cert_der = CertificateDer::from(cert.cert);
    let priv_key = PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der());

    let config = ServerConfig::with_single_cert(vec![cert_der.clone()], priv_key.into()).unwrap();
    config
}
