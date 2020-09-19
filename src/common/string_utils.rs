pub trait StringUtils {
    /// Returns a substring of the given string, trimmed at the specified length.
    fn sub_str(self, len: usize) -> Self;

    /// Truncates the given string if it exceeds the specified length,
    /// or pads it with spaces if it is not long enough.
    fn fix_length(self, len: usize) -> Self;
}

impl StringUtils for String {
    /// Returns a substring of the given string, trimmed at the specified length.
    ///
    /// # Examples
    /// ```
    /// use rusty_basic::common::StringUtils;
    /// assert_eq!(String::from("abc").sub_str(2), String::from("ab"));
    /// assert_eq!(String::from("abc").sub_str(3), String::from("abc"));
    /// assert_eq!(String::from("abc").sub_str(4), String::from("abc"));
    /// ```
    fn sub_str(self, len: usize) -> Self {
        if len < self.len() {
            let chars: Vec<char> = self.chars().collect();
            let c = &chars[0..len];
            let s2: String = c.iter().collect();
            s2
        } else {
            self
        }
    }

    /// Truncates the given string if it exceeds the specified length,
    /// or pads it with spaces if it is not long enough.
    ///
    /// # Examples
    /// ```
    /// use rusty_basic::common::StringUtils;
    /// assert_eq!(String::from("abc").fix_length(2), String::from("ab"));
    /// assert_eq!(String::from("abc").fix_length(3), String::from("abc"));
    /// assert_eq!(String::from("abc").fix_length(4), String::from("abc "));
    /// ```
    fn fix_length(self, len: usize) -> Self {
        if len < self.len() {
            // truncate
            let chars: Vec<char> = self.chars().collect();
            let c = &chars[0..len];
            let s2: String = c.iter().collect();
            s2
        } else if len > self.len() {
            // extend with spaces
            let mut result = self.clone();
            result.extend(std::iter::repeat(' ').take(len - self.len()));
            result
        } else {
            self
        }
    }
}
