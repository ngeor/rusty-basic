pub trait StringUtils {
    /// Truncates the given string if it exceeds the specified length,
    /// or pads it with spaces if it is not long enough.
    fn fix_length(self, len: usize) -> Self;

    /// Pads the given string with spaces on the left side so that the
    /// resulting string is of the specified length.
    fn pad_left(self, len: usize) -> Self;
}

impl StringUtils for String {
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

    /// Pads the given string with spaces on the left side.
    ///
    /// # Examples
    /// ```
    /// use rusty_basic::common::StringUtils;
    /// assert_eq!(String::from("abc").pad_left(0), String::from("abc"));
    /// assert_eq!(String::from("abc").pad_left(3), String::from("abc"));
    /// assert_eq!(String::from("abc").pad_left(4), String::from(" abc"));
    /// ```
    fn pad_left(self, len: usize) -> Self {
        if len <= self.len() {
            self
        } else {
            // prepend spaces
            let mut result: String = std::iter::repeat(' ').take(len - self.len()).collect();
            result.push_str(self.as_str());
            result
        }
    }
}
