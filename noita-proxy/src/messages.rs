use bitcode::{Decode, Encode};

use crate::GameSettings;

#[derive(Debug, Decode, Encode)]
pub enum NetMsg {
    StartGame { settings: GameSettings },
    ModRaw { data: Vec<u8> },
    ModCompressed { data: Vec<u8> },
}
