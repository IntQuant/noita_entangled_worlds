use std::num::NonZero;

use bitcode::{Decode, Encode};

use crate::WorldPos;

#[derive(Encode, Decode)]
pub enum EntityData {
    Serialized(Vec<u8>),
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
    CurrentEntity(NonZero<i32>),
    SetPosition(WorldPos),
    // TODO...
}

#[derive(Encode, Decode, Clone)]
pub enum RemoteDes {
    InterestRequest(InterestRequest),
    EnteredInterest,
    ExitedInterest,
    EntityUpdate(Vec<EntityUpdate>),
}
