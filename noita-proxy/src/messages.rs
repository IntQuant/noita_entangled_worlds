use bitcode::{Decode, Encode};

use crate::GameSettings;

#[derive(Decode, Encode)]
pub enum NetMsg {
    StartGame { settings: GameSettings },
    ModRaw { data: Vec<u8> },
}
