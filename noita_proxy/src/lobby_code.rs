use std::fmt::{self};

use steamworks::LobbyId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LobbyKind {
    Steam,
    Gog,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LobbyCode {
    pub kind: LobbyKind,
    pub code: LobbyId,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LobbyError {
    NotALobbyCode,
    CodeVersionMismatch,
}

impl fmt::Display for LobbyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LobbyError::NotALobbyCode => write!(f, "Not a lobby code"),
            LobbyError::CodeVersionMismatch => write!(f, "Code version mismatch"),
        }
    }
}

impl LobbyCode {
    const VERSION: char = '0';
    const BASE: u64 = 0x0186000000000000;

    pub fn parse(raw: &str) -> Result<Self, LobbyError> {
        let raw = raw.trim();
        if !(raw.starts_with('e') && raw.ends_with('w')) {
            return Err(LobbyError::NotALobbyCode);
        }
        let mut chars = raw.chars();
        chars.next();
        let version_char = chars.next().ok_or(LobbyError::NotALobbyCode)?;
        if version_char != Self::VERSION {
            return Err(LobbyError::CodeVersionMismatch);
        }
        let kind = match chars.next().ok_or(LobbyError::NotALobbyCode)? {
            's' => LobbyKind::Steam,
            'g' => LobbyKind::Gog,
            _ => {
                return Err(LobbyError::NotALobbyCode);
            }
        };

        let lobby_hex = &raw[3..raw.len() - 1];
        let code = u64::from_str_radix(lobby_hex, 16).map_err(|_| LobbyError::NotALobbyCode)?;

        Ok(LobbyCode {
            kind,
            code: LobbyId::from_raw(code.wrapping_add(Self::BASE)),
        })
    }

    pub fn serialize(self) -> String {
        let dat = match self.kind {
            LobbyKind::Steam => 's',
            LobbyKind::Gog => 'g',
        };
        format!(
            "e{}{}{:x}w",
            Self::VERSION,
            dat,
            self.code.raw().wrapping_sub(Self::BASE)
        )
    }
}

#[cfg(test)]
mod test {
    use steamworks::LobbyId;

    use super::LobbyCode;

    #[test]
    fn test_serialize() {
        let code = LobbyCode {
            kind: super::LobbyKind::Steam,
            code: LobbyId::from_raw(LobbyCode::BASE + 100),
        };
        assert_eq!(code.serialize(), "e0s64w");
    }

    #[test]
    fn test_deserialize() {
        let code = LobbyCode::parse("e0s64w");
        assert_eq!(
            code,
            Ok(LobbyCode {
                kind: super::LobbyKind::Steam,
                code: LobbyId::from_raw(LobbyCode::BASE + 100),
            })
        );
    }
}
