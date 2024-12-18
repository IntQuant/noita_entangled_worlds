use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Clone, Copy)]
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

impl<T> Destination<T> {
    pub fn convert<A>(self) -> Destination<A>
    where
        A: From<T>,
    {
        match self {
            Destination::Peer(p) => Destination::Peer(p.into()),
            Destination::Host => Destination::Host,
            Destination::Broadcast => Destination::Broadcast,
        }
    }
}
