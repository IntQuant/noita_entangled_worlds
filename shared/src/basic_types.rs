use bitcode::{Decode, Encode};

#[derive(Encode, Decode)]
pub struct WorldPos {
    pub x: i32,
    pub y: i32,
}

#[derive(Encode, Decode)]
pub struct PeerId(pub u64);

#[derive(Encode, Decode, Debug, PartialEq, Eq)]
pub enum Destination<PeerType> {
    Peer(PeerType),
    Host,
    Broadcast,
}
