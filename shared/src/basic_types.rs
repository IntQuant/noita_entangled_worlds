use std::ops::Sub;

use bitcode::{Decode, Encode};
use eyre::Context;

#[derive(Debug, Encode, Decode, Default, Clone, Copy)]
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

    pub fn dist(&self, other: &WorldPos) -> (u64, f32) {
        let dx = self.x.abs_diff(other.x) as u64;
        let dy = self.y.abs_diff(other.y) as u64;
        //(dx as f64).hypot(dy as f64) as u64
        (
            dx.saturating_mul(dx).saturating_add(dy.saturating_mul(dy)),
            (dy as f32).atan2(dx as f32),
        )
    }

    pub fn as_array(&self) -> [i64; 2] {
        [self.x as i64, self.y as i64]
    }

    pub fn contains(self, x: f64, y: f64, dist: u32) -> bool {
        let dx = self.x.abs_diff(x as i32);
        let dy = self.y.abs_diff(y as i32);
        dx.saturating_mul(dx).saturating_add(dy.saturating_mul(dy)) < dist.saturating_mul(dist)
    }
}

impl From<(i32, i32)> for WorldPos {
    fn from(value: (i32, i32)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

#[derive(Debug, Encode, Decode, Default, Clone, Copy)]
pub struct WorldVec {
    pub x: i32,
    pub y: i32,
}

impl Sub<WorldPos> for WorldPos {
    type Output = WorldVec;

    fn sub(self, rhs: WorldPos) -> Self::Output {
        WorldVec {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Sub<WorldVec> for WorldVec {
    type Output = WorldVec;

    fn sub(self, rhs: WorldVec) -> Self::Output {
        WorldVec {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl WorldVec {
    pub fn dot(self, other: WorldVec) -> f32 {
        (self.x as f32) * (other.y as f32) + (self.y as f32) * (other.x as f32)
    }
    pub fn hypot(self) -> f32 {
        f32::hypot(self.x as f32, self.y as f32)
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
