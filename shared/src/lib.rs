use bitcode::{Decode, Encode};

#[derive(Encode, Decode)]
pub struct PeerId(pub u64);

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
    pub peer: PeerId,
    pub value: Vec<u8>,
}

#[derive(Encode, Decode)]
pub enum NoitaInbound {
    // ProxyKV(ProxyKV),
    // ProxyKVBin(ProxyKVBin),
    // ModMessage(ModMessage),
    RawMessage(Vec<u8>),
    Ready,
}

#[derive(Encode, Decode)]
pub enum NoitaOutbound {
    // ModMessage(ModMessage),
    Raw(Vec<u8>),
}
