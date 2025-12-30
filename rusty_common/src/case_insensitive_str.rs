use std::cmp::Ordering;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};

use crate::CaseInsensitiveString;
use crate::case_insensitive_utils::{cmp_str, hash_str};

#[derive(Debug)]
pub struct CaseInsensitiveStr(str);

impl CaseInsensitiveStr {
    pub const fn new(value: &str) -> &Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl ToOwned for CaseInsensitiveStr {
    type Owned = CaseInsensitiveString;

    fn to_owned(&self) -> Self::Owned {
        CaseInsensitiveString::new(self.0.to_owned())
    }
}

impl std::fmt::Display for CaseInsensitiveStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Hash for CaseInsensitiveStr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_str(&self.0, state)
    }
}

impl PartialEq for CaseInsensitiveStr {
    fn eq(&self, other: &Self) -> bool {
        self.eq(&other.0)
    }
}

impl Eq for CaseInsensitiveStr {}

impl PartialOrd for CaseInsensitiveStr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.partial_cmp(&other.0)
    }
}

impl Ord for CaseInsensitiveStr {
    fn cmp(&self, other: &Self) -> Ordering {
        cmp_str(&self.0, &other.0)
    }
}

impl PartialEq<str> for CaseInsensitiveStr {
    fn eq(&self, other: &str) -> bool {
        matches!(self.partial_cmp(other), Some(Ordering::Equal))
    }
}

impl PartialOrd<str> for CaseInsensitiveStr {
    fn partial_cmp(&self, other: &str) -> Option<Ordering> {
        Some(cmp_str(&self.0, other))
    }
}

impl AsRef<str> for CaseInsensitiveStr {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use crate::CaseInsensitiveStr;

    #[test]
    fn test_cmp_ignore_ascii_case() {
        assert_eq!(
            CaseInsensitiveStr::new("abc").partial_cmp("abc"),
            Some(Ordering::Equal)
        );
        assert_eq!(
            CaseInsensitiveStr::new("abc").partial_cmp("ABC"),
            Some(Ordering::Equal)
        );
        assert_eq!(
            CaseInsensitiveStr::new("ABC").partial_cmp("abc"),
            Some(Ordering::Equal)
        );
        assert_eq!(
            CaseInsensitiveStr::new("ABC").partial_cmp("ABC"),
            Some(Ordering::Equal)
        );

        assert_eq!(
            CaseInsensitiveStr::new("abc").partial_cmp("def"),
            Some(Ordering::Less)
        );
        assert_eq!(
            CaseInsensitiveStr::new("abc").partial_cmp("DEF"),
            Some(Ordering::Less)
        );
        assert_eq!(
            CaseInsensitiveStr::new("ABC").partial_cmp("def"),
            Some(Ordering::Less)
        );
        assert_eq!(
            CaseInsensitiveStr::new("ABC").partial_cmp("DEF"),
            Some(Ordering::Less)
        );

        assert_eq!(
            CaseInsensitiveStr::new("xyz").partial_cmp("def"),
            Some(Ordering::Greater)
        );
        assert_eq!(
            CaseInsensitiveStr::new("xyz").partial_cmp("DEF"),
            Some(Ordering::Greater)
        );
        assert_eq!(
            CaseInsensitiveStr::new("XYZ").partial_cmp("def"),
            Some(Ordering::Greater)
        );
        assert_eq!(
            CaseInsensitiveStr::new("XYZ").partial_cmp("DEF"),
            Some(Ordering::Greater)
        );

        assert_eq!(
            CaseInsensitiveStr::new("abc").partial_cmp("abca"),
            Some(Ordering::Less)
        );
        assert_eq!(
            CaseInsensitiveStr::new("abca").partial_cmp("abc"),
            Some(Ordering::Greater)
        );
    }
}
