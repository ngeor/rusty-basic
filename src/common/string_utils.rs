pub trait StringUtils {
    /// Truncates the given string if it exceeds the specified length,
    /// or pads it with spaces if it is not long enough.
    fn fix_length(&self, len: usize) -> String;

    /// Pads the given string with spaces on the left side so that the
    /// resulting string is of the specified length.
    fn pad_left(&self, len: usize) -> String;
}

impl StringUtils for str {
    /// Truncates the given string if it exceeds the specified length,
    /// or pads it with spaces if it is not long enough.
    ///
    /// # Examples
    /// ```
    /// use rusty_basic::common::StringUtils;
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

    /// Pads the given string with spaces on the left side.
    ///
    /// # Examples
    /// ```
    /// use rusty_basic::common::StringUtils;
    /// assert_eq!(String::from("abc").pad_left(0), String::from("abc"));
    /// assert_eq!(String::from("abc").pad_left(3), String::from("abc"));
    /// assert_eq!(String::from("abc").pad_left(4), String::from(" abc"));
    /// ```
    fn pad_left(&self, len: usize) -> String {
        if len <= self.len() {
            self.to_owned()
        } else {
            // prepend spaces
            let mut result: String = " ".repeat(len - self.len());
            result.push_str(self);
            result
        }
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
