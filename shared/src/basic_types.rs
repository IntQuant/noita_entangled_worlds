use bitcode::{Decode, Encode};
use eyre::Context;

#[derive(Debug, Encode, Decode, Clone, Copy)]
pub struct WorldPos {
    pub x: i32,
    pub y: i32,
}

impl WorldPos {
    pub fn from_f32(x: f32, y: f32) -> Self {
        Self {
            x: x as i32,
            y: y as i32,
        }
    }

    pub fn from_f64(x: f64, y: f64) -> Self {
        Self {
            x: x as i32,
            y: y as i32,
        }
    }

    pub fn as_array(&self) -> [i64; 2] {
        [self.x as i64, self.y as i64]
    }
}

#[derive(Debug, Encode, Decode, Hash, PartialEq, Eq, Clone, Copy, Default)]
pub struct PeerId(pub u64);

impl PeerId {
    pub fn from_hex(hex_str: &str) -> eyre::Result<Self> {
        Ok(Self(
            u64::from_str_radix(hex_str, 16).wrap_err("Failed to parse PeerId")?,
        ))
    }
}

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
