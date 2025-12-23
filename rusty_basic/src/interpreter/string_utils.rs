/// Truncates the given string if it exceeds the specified length,
/// or pads it with spaces if it is not long enough.
pub fn fix_length(s: &mut String, len: usize) {
    // truncate string that contains null
    if let Some(index) = s.find('\0') {
        while s.len() > index {
            s.pop();
        }
    }

    while s.len() > len {
        s.pop();
    }

    while s.len() < len {
        s.push(' ');
    }
}

pub fn to_ascii_string(bytes: &[u8]) -> String {
    bytes.iter().map(|b| *b as char).collect()
}

pub fn to_ascii_bytes(s: &str) -> Vec<u8> {
    s.chars().map(|c| c as u8).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_length_trim() {
        let mut s = String::from("abc");
        fix_length(&mut s, 2);
        assert_eq!(s, "ab");
    }

    #[test]
    fn test_fix_length_unchanged() {
        let mut s = String::from("abc");
        fix_length(&mut s, 3);
        assert_eq!(s, "abc");
    }

    #[test]
    fn test_fix_length_pad() {
        let mut s = String::from("abc");
        fix_length(&mut s, 4);
        assert_eq!(s, "abc ");
    }

    #[test]
    fn test_fix_length_replace_null_with_space() {
        let mut s = String::from("ab\0");
        fix_length(&mut s, 3);
        assert_eq!(s, "ab ");
    }
}
