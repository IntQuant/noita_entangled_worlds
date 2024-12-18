use bitcode::{Decode, Encode};

pub mod message_socket;

pub mod basic_types;
pub mod des;

pub use basic_types::*;

#[derive(Encode, Decode)]
pub struct ProxyKV {
    pub key: String,
    pub value: String,
}

#[derive(Encode, Decode)]
pub struct ProxyKVBin {
    pub key: u8,
    pub value: Vec<u8>,
}

#[derive(Encode, Decode)]
pub struct ModMessage {
    pub peer: basic_types::PeerId,
    pub value: Vec<u8>,
}

#[derive(Encode, Decode, Clone)]
pub enum RemoteMessage {
    RemoteDes(des::RemoteDes),
}

#[derive(Encode, Decode)]
pub enum NoitaInbound {
    RawMessage(Vec<u8>),
    Ready,
    ProxyToDes(des::ProxyToDes),
    RemoteMessage {
        source: basic_types::PeerId,
        message: RemoteMessage,
    },
}

#[derive(Encode, Decode)]
pub enum NoitaOutbound {
    Raw(Vec<u8>),
    DesToProxy(des::DesToProxy),
    RemoteMessage {
        reliable: bool,
        destination: Destination<PeerId>,
        message: RemoteMessage,
    },
}
