use shared::des::Gid;

/// The `ew_gid_lid` variable name as the engine serializes it: a big-endian
/// u32 length followed by the bytes.
const FRAMED_NAME: &[u8] = b"\x00\x00\x00\x0aew_gid_lid";

/// `u64::MAX` is 20 digits, so a longer value string is not a gid.
const MAX_DIGITS: usize = 20;

/// Pulls the `ew_gid_lid` value out of an entity the engine serialized.
///
/// The engine writes components positionally: every string is a big-endian
/// u32 length followed by its bytes, and a `VariableStorageComponent` is
/// `name`, `value_string`, `value_int`, `value_bool`, `value_float` in that
/// order. So the gid sits directly behind the length-prefixed name:
///
/// ```text
/// 00 00 00 0a "ew_gid_lid"  00 00 00 NN "<NN ascii digits>"
/// ```
///
/// Matching the name together with its own length prefix rejects the same
/// text inside a tag list, and reading exactly `NN` bytes means a binary byte
/// that happens to be an ASCII digit can never leak into the number.
pub(crate) fn gid_from_serialized_entity(data: &[u8]) -> Option<Gid> {
    let start = data
        .windows(FRAMED_NAME.len())
        .position(|w| w == FRAMED_NAME)?
        + FRAMED_NAME.len();
    let len_bytes: [u8; 4] = data.get(start..start + 4)?.try_into().ok()?;
    let len = u32::from_be_bytes(len_bytes) as usize;
    if len == 0 || len > MAX_DIGITS {
        return None;
    }
    let digits = data.get(start + 4..start + 4 + len)?;
    if !digits.iter().all(u8::is_ascii_digit) {
        return None;
    }
    std::str::from_utf8(digits).ok()?.parse().ok().map(Gid)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn string(s: &[u8]) -> Vec<u8> {
        let mut out = (s.len() as u32).to_be_bytes().to_vec();
        out.extend_from_slice(s);
        out
    }

    /// A VariableStorageComponent as the engine lays it out, surrounded by
    /// binary noise that contains ASCII digits.
    fn blob(name: &[u8], value: &[u8]) -> Vec<u8> {
        let mut out = vec![0x00, 0x00, 0x00, 0x02, 0x37, 0x39];
        out.extend(string(b"VariableStorageComponent"));
        out.push(0x01);
        out.extend(string(b""));
        out.extend(string(name));
        out.extend(string(value));
        out.extend_from_slice(&0x3132_3334_i32.to_be_bytes());
        out.push(0x01);
        out.extend_from_slice(&1.0f32.to_be_bytes());
        out
    }

    #[test]
    fn reads_the_framed_value() {
        assert_eq!(
            gid_from_serialized_entity(&blob(b"ew_gid_lid", b"1234567890123")),
            Some(Gid(1234567890123))
        );
    }

    #[test]
    fn reads_exactly_the_declared_length() {
        // The int field behind the string is 0x31323334, i.e. "1234"; the old
        // digit-run scan would have glued it on.
        assert_eq!(
            gid_from_serialized_entity(&blob(b"ew_gid_lid", b"42")),
            Some(Gid(42))
        );
    }

    #[test]
    fn ignores_the_name_inside_a_tag_list() {
        let mut data = string(b"enemy,ew_gid_lid,ew_des");
        data.extend(blob(b"other_var", b"7"));
        assert_eq!(gid_from_serialized_entity(&data), None);
    }

    #[test]
    fn rejects_non_digit_and_oversized_values() {
        assert_eq!(
            gid_from_serialized_entity(&blob(b"ew_gid_lid", b"12a4")),
            None
        );
        assert_eq!(gid_from_serialized_entity(&blob(b"ew_gid_lid", b"")), None);
        assert_eq!(
            gid_from_serialized_entity(&blob(b"ew_gid_lid", &[b'9'; 21])),
            None
        );
    }

    #[test]
    fn tolerates_a_truncated_blob() {
        let full = blob(b"ew_gid_lid", b"12345");
        for cut in 0..full.len() {
            let _ = gid_from_serialized_entity(&full[..cut]);
        }
        assert_eq!(gid_from_serialized_entity(&full), Some(Gid(12345)));
    }
}
