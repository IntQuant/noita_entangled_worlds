use bitcode::{Decode, Encode};

use crate::WorldPos;

/// 64 bit globally unique id. Assigned randomly, should only have 50% chance of collision with 2^32 entities at once.
pub type Gid = u64;

// 32 bit locally unique id.
#[derive(Encode, Decode, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Lid(pub u32);

#[derive(Encode, Decode, Clone)]
pub enum EntitySpawnInfo {
    Filename(String),
    // Serialized(Vec<u8>),
}

#[derive(Encode, Decode)]
pub struct FullEntityData {
    pub gid: Gid,
    pub pos: WorldPos,
    pub data: EntitySpawnInfo,
}

#[derive(Encode, Decode)]
pub enum DesToProxy {
    InitOrUpdateEntity(FullEntityData),
    DeleteEntity { gid: Gid },
    ReleaseAuthority(FullEntityData),
}

#[derive(Encode, Decode)]
pub enum ProxyToDes {
    /// Got authority over entity.
    GotAuthority(FullEntityData),
}

#[derive(Encode, Decode, Clone)]
pub struct InterestRequest {
    pub pos: WorldPos,
    pub radius: i32,
}

#[derive(Encode, Decode, Clone)]
pub struct EntityInfo {
    pub entity_data: EntitySpawnInfo,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
}

#[derive(Encode, Decode, Clone)]
pub enum EntityUpdate {
    /// Sets the gid that following EntityUpdates will act on.
    CurrentEntity(Lid),
    Init(EntityInfo),
    // TODO diffing for position
    SetPosition(f32, f32),
    SetVelocity(f32, f32),
    // TODO...
    RemoveEntity(Lid),
}

#[derive(Encode, Decode, Clone)]
pub enum RemoteDes {
    /// Should be sent when client opens the game, to reset in case of restart.
    Reset,
    InterestRequest(InterestRequest),
    EntityUpdate(Vec<EntityUpdate>),
    ExitedInterest,
}
