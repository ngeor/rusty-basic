use crate::case_insensitive_utils::{cmp_str, hash_str};
use crate::CaseInsensitiveStr;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

/// A string that is case insensitive when comparing or checking for equality.
#[derive(Clone, Debug)]
pub struct CaseInsensitiveString(String);

impl CaseInsensitiveString {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    /// Checks if the string contains the given character, case insensitively.
    ///
    /// # Examples
    ///
    /// ```
    /// use rusty_common::CaseInsensitiveString;
    /// assert_eq!(CaseInsensitiveString::from("a#").contains('#'), true);
    /// assert_eq!(CaseInsensitiveString::from("a#").contains('A'), true);
    /// assert_eq!(CaseInsensitiveString::from("a#").contains('b'), false);
    /// ```
    pub fn contains(&self, needle: char) -> bool {
        for c in self.chars() {
            if c.eq_ignore_ascii_case(&needle) {
                return true;
            }
        }

        false
    }

    /// If the string contains the given delimiter, splits the string and returns the part
    /// before the delimiter.
    ///
    /// # Examples
    ///
    /// ```
    /// use rusty_common::CaseInsensitiveStr;
    /// use rusty_common::CaseInsensitiveString;
    /// assert_eq!(CaseInsensitiveString::from("a.b").prefix('.'), Some(CaseInsensitiveStr::new("a")));
    /// assert_eq!(CaseInsensitiveString::from("ab").prefix('.'), None);
    /// ```
    pub fn prefix(&self, delimiter: char) -> Option<&CaseInsensitiveStr> {
        if self.contains(delimiter) {
            let parts: Vec<&str> = self.0.split('.').collect();
            debug_assert!(!parts.is_empty());
            let first = parts[0];
            Some(CaseInsensitiveStr::new(first))
        } else {
            None
        }
    }
}

impl Deref for CaseInsensitiveString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CaseInsensitiveString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Display for CaseInsensitiveString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Hash for CaseInsensitiveString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_str(self.0.as_str(), state)
    }
}

impl PartialEq for CaseInsensitiveString {
    fn eq(&self, other: &Self) -> bool {
        cmp_str(self.0.as_str(), other.0.as_str()) == Ordering::Equal
    }
}

impl Eq for CaseInsensitiveString {}

impl Borrow<CaseInsensitiveStr> for CaseInsensitiveString {
    fn borrow(&self) -> &CaseInsensitiveStr {
        CaseInsensitiveStr::new(&self.0)
    }
}

impl From<String> for CaseInsensitiveString {
    fn from(x: String) -> Self {
        Self::new(x)
    }
}

impl From<&str> for CaseInsensitiveString {
    fn from(x: &str) -> Self {
        Self::new(x.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    #[test]
    fn test_case_insensitive_string() {
        let x: CaseInsensitiveString = CaseInsensitiveString::new("abcDEF".into());
        let y: CaseInsensitiveString = CaseInsensitiveString::new("ABCdef".into());
        assert_eq!("abcDEF".to_string(), x.to_string());
        assert_eq!("ABCdef".to_string(), y.to_string());
        assert_eq!(x, y);

        let mut hasher = DefaultHasher::new();
        x.hash(&mut hasher);
        let x_hash = hasher.finish();

        hasher = DefaultHasher::new();
        y.hash(&mut hasher);
        let y_hash = hasher.finish();

        assert_eq!(x_hash, y_hash);
    }
}
