pub trait StringUtils {
    /// Truncates the given string if it exceeds the specified length,
    /// or pads it with spaces if it is not long enough.
    fn fix_length(&self, len: usize) -> String;
}

impl StringUtils for str {
    /// Truncates the given string if it exceeds the specified length,
    /// or pads it with spaces if it is not long enough.
    ///
    /// # Examples
    /// ```
    /// use rusty_common::StringUtils;
    /// assert_eq!(String::from("abc").fix_length(2), String::from("ab"));
    /// assert_eq!(String::from("abc").fix_length(3), String::from("abc"));
    /// assert_eq!(String::from("abc").fix_length(4), String::from("abc "));
    /// assert_eq!(String::from("ab\0").fix_length(3), String::from("ab "));
    /// ```
    fn fix_length(&self, len: usize) -> String {
        let mut result: String = String::new();
        for ch in self.chars() {
            if ch == '\0' || result.len() >= len {
                break;
            }
            result.push(ch);
        }
        while result.len() < len {
            result.push(' ');
        }
        result
    }
}

pub trait ToAsciiString {
    fn to_ascii_string(&self) -> String;
}

impl ToAsciiString for [u8] {
    fn to_ascii_string(&self) -> String {
        self.iter().map(|b| *b as char).collect()
    }
}

pub trait ToAsciiBytes {
    fn to_ascii_bytes(&self) -> Vec<u8>;
}

impl ToAsciiBytes for str {
    fn to_ascii_bytes(&self) -> Vec<u8> {
        self.chars().map(|c| c as u8).collect()
    }
}
