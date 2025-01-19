use std::sync::Arc;

use crate::{GameEffectData, PeerId, WorldPos};
use bitcode::{Decode, Encode};

pub const REQUEST_AUTHORITY_RADIUS: i32 = 400;
pub const TRANSFER_RADIUS: f32 = 500.0;
pub const AUTHORITY_RADIUS: f32 = 600.0;
pub const INTEREST_REQUEST_RADIUS: i32 = 900;

/// 64 bit globally unique id. Assigned randomly, should only have 50% chance of collision with 2^32 entities at once.
#[derive(Debug, Encode, Decode, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Gid(pub u64);

// 32 bit locally unique id.
#[derive(Debug, Encode, Decode, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Lid(pub u32);

#[derive(Encode, Decode, Clone, PartialEq)]
pub enum EntitySpawnInfo {
    Filename(String),
    Serialized { serialized_at: i32, data: Vec<u8> },
}

impl Default for EntitySpawnInfo {
    fn default() -> Self {
        Self::Filename(String::new())
    }
}

#[derive(Encode, Decode, Clone)]
pub struct FullEntityData {
    pub gid: Gid,
    pub pos: WorldPos,
    pub data: EntitySpawnInfo,
}

#[derive(Encode, Decode, Clone)]
pub struct UpdatePosition {
    pub gid: Gid,
    pub pos: WorldPos,
}

#[derive(Encode, Decode, Clone)]
pub enum DesToProxy {
    InitOrUpdateEntity(FullEntityData),
    DeleteEntity(Gid),
    ReleaseAuthority(Gid),
    RequestAuthority { pos: WorldPos, radius: i32 },
    UpdatePositions(Vec<UpdatePosition>),
    TransferAuthorityTo(Gid, PeerId),
}

#[derive(Encode, Decode, Clone)]
pub enum ProxyToDes {
    /// Got authority over entity.
    GotAuthority(FullEntityData),
}

#[derive(Encode, Decode, Clone)]
pub struct InterestRequest {
    pub pos: WorldPos,
    pub radius: i32,
}

#[derive(Encode, Decode, Clone, Copy, PartialEq)]
pub struct PhysBodyInfo {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub vx: f32,
    pub vy: f32,
    pub av: f32,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default)]
pub enum EntityKind {
    /// Normal entity, replicated with a filename.
    #[default]
    Normal,
    // Item entity, can be picked up.
    Item,
}

#[derive(Encode, Decode, Clone, PartialEq, Default)]
pub struct EntityInfo {
    pub spawn_info: EntitySpawnInfo,
    pub kind: EntityKind,
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub vx: f32,
    pub vy: f32,
    pub hp: f32,
    pub phys: Vec<Option<PhysBodyInfo>>,
    pub cost: i64,
    pub game_effects: Vec<GameEffectData>,
    pub current_stains: u64,
    pub wand: Option<(Option<Gid>, Vec<u8>)>,
    pub is_global: bool,
    pub drops_gold: bool,
    pub limbs: Vec<(f32, f32)>,
    pub ai_state: i32,
    pub ai_rotation: f32,
    pub is_enabled: bool, //for kolmi/runestones/etc
    pub counter: u8,      //for mom orbs/dragon has death script/etc
                          //pub synced_var: Vec<(String, i32, f32, bool)>, for mod compat maybe
}
//TODO stevari
//TODO authority transfers should serialize entities probably
//TODO kivi/mom/spirit might deal double damage
//TODO sync x scale direction as a bool probably
//TODO shop wands jittery
//TODO host melee damage from clients enemies
//TODO fish rotate
#[derive(Encode, Decode, Clone)]
pub enum EntityUpdate {
    /// Sets the gid that following EntityUpdates will act on.
    CurrentEntity(Lid),
    Init(Box<EntityInfo>, Gid),
    // TODO diffing for position
    SetPosition(f32, f32),
    SetRotation(f32),
    SetVelocity(f32, f32),
    SetHp(f32),
    SetPhysInfo(Vec<Option<PhysBodyInfo>>),
    // TODO...
    RemoveEntity(Lid),
    LocalizeEntity(Lid, PeerId),
    KillEntity {
        lid: Lid,
        responsible_peer: Option<PeerId>,
    },
    SetCost(i64),
    SetStains(u64),
    SetGameEffects(Vec<GameEffectData>),
    SetWand(Option<(Option<Gid>, Vec<u8>)>),
    SetAiRotation(f32),
    SetAiState(i32),
    SetLimbs(Vec<(f32, f32)>),
    SetIsEnabled(bool),
    SetCounter(u8),
}

#[derive(Encode, Decode, Clone)]
pub enum RemoteDes {
    /// Should be sent when client opens the game, to reset in case of restart.
    Reset,
    InterestRequest(InterestRequest),
    EntityUpdate(Vec<EntityUpdate>),
    ExitedInterest,
    Projectiles(Arc<Vec<ProjectileFired>>),
    RequestGrab(Lid),
}

#[derive(Debug, Encode, Decode, Clone)]
pub struct ProjectileFired {
    pub shooter_lid: Lid,
    pub position: (f32, f32),
    pub target: (f32, f32),
    pub serialized: Vec<u8>,
}
