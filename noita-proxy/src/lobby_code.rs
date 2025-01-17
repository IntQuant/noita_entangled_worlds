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

impl LobbyCode {
    const VERSION: char = '0';

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

        let lobby_hex = &raw[3..3 + 16];
        let code = u64::from_str_radix(lobby_hex, 16).map_err(|_| LobbyError::NotALobbyCode)?;

        Ok(LobbyCode {
            kind,
            code: LobbyId::from_raw(code),
        })
    }

    pub fn serialize(self) -> String {
        let dat = match self.kind {
            LobbyKind::Steam => 's',
            LobbyKind::Gog => 'g',
        };
        format!("e{}{}{:016x}w", Self::VERSION, dat, self.code.raw())
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
            code: LobbyId::from_raw(100),
        };
        assert_eq!(code.serialize(), "e0s0000000000000064w");
    }

    #[test]
    fn test_deserialize() {
        let code = LobbyCode::parse("e0s0000000000000064w");
        assert_eq!(
            code,
            Ok(LobbyCode {
                kind: super::LobbyKind::Steam,
                code: LobbyId::from_raw(100),
            })
        );
    }
}
