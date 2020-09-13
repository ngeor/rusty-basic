pub trait StringUtils {
    /// Returns a substring of the given string, trimmed at the specified length.
    fn sub_str(self, len: usize) -> Self;
}

impl StringUtils for String {
    /// Returns a substring of the given string, trimmed at the specified length.
    ///
    /// # Examples
    /// ```
    /// use rusty_basic::common::StringUtils;
    /// assert_eq!(String::from("abc").sub_str(2), String::from("ab"));
    /// assert_eq!(String::from("abc").sub_str(5), String::from("abc"));
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
}
