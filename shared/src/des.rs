use std::num::NonZero;

use bitcode::{Decode, Encode};

use crate::WorldPos;

/// 64 bit globally unique id. Assigned randomly, should only have 50% chance of collision with 2^32 entities at once.
pub type Gid = u64;

#[derive(Encode, Decode, Clone)]
pub enum EntityData {
    Filename(String),
    // Serialized(Vec<u8>),
}

#[derive(Encode, Decode)]
pub struct FullEntityData {
    pub gid: u64,
    pub pos: WorldPos,
    pub data: EntityData,
}

#[derive(Encode, Decode)]
pub enum DesToProxy {
    InitOrUpdateEntity(FullEntityData),
    DeleteEntity {
        gid: u64,
    },
    ReleaseAuthority(FullEntityData),
    /// GotEntity is a Response for this.
    RequestEntity {
        gid: u64,
    },
}

#[derive(Encode, Decode)]
pub enum ProxyToDes {
    /// RequestEntity is a Request for this.
    GotEntity(FullEntityData),
    /// Got authority over entity.
    GotAuthority(FullEntityData),
}

#[derive(Encode, Decode, Clone)]
pub struct InterestRequest {
    pub pos: WorldPos,
    pub radius: i32,
}

#[derive(Encode, Decode, Clone)]
pub enum EntityUpdate {
    /// Sets the gid that following EntityUpdates will act on.
    CurrentEntity(Gid),
    EntityData(EntityData),
    SetPosition(f32, f32),
    // TODO...
    RemoveEntity(Gid),
}

#[derive(Encode, Decode, Clone)]
pub enum RemoteDes {
    InterestRequest(InterestRequest),
    EntityUpdate(Vec<EntityUpdate>),
    ExitedInterest,
}
