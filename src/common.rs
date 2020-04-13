use std::cmp::Ordering;

// Location

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Location {
    row: u32,
    col: u32,
}

impl Location {
    pub fn new(row: u32, col: u32) -> Location {
        Location { row, col }
    }

    pub fn inc_col(&mut self) {
        self.col += 1
    }

    pub fn inc_row(&mut self) {
        self.row += 1;
        self.col = 1;
    }

    pub fn start() -> Location {
        Location::new(1, 1)
    }
}

// Locatable

#[derive(Clone, Debug, PartialEq)]
pub struct Locatable<T: std::fmt::Debug + Sized> {
    element: T,
    location: Location,
}

impl<T: std::fmt::Debug + Sized> Locatable<T> {
    pub fn new(element: T, location: Location) -> Locatable<T> {
        Locatable { element, location }
    }

    pub fn element(&self) -> &T {
        &self.element
    }

    pub fn element_into(self) -> T {
        self.element
    }

    pub fn map<U: std::fmt::Debug + Sized, F>(&self, f: F) -> Locatable<U>
    where
        F: Fn(&T) -> U,
    {
        Locatable::new(f(&self.element), self.location)
    }

    pub fn map_into<U: std::fmt::Debug + Sized, F>(self, f: F) -> Locatable<U>
    where
        F: Fn(T) -> U,
    {
        Locatable::new(f(self.element), self.location)
    }
}

// HasLocation

pub trait HasLocation {
    fn location(&self) -> Location;
}

impl<T: std::fmt::Debug + Sized> HasLocation for Locatable<T> {
    fn location(&self) -> Location {
        self.location
    }
}

impl<T: HasLocation> HasLocation for Box<T> {
    fn location(&self) -> Location {
        let inside_the_box: &T = self;
        inside_the_box.location()
    }
}

// CaseInsensitiveString

#[derive(Clone, Debug)]
pub struct CaseInsensitiveString {
    inner: String,
}

impl CaseInsensitiveString {
    pub fn new(value: String) -> CaseInsensitiveString {
        CaseInsensitiveString { inner: value }
    }

    /// Gets the first character of the string.
    /// Panics if the string is empty.
    pub fn first_char(&self) -> char {
        self.inner.chars().next().unwrap()
    }
}

impl From<&str> for CaseInsensitiveString {
    fn from(x: &str) -> CaseInsensitiveString {
        CaseInsensitiveString::new(x.to_owned())
    }
}

impl std::fmt::Display for CaseInsensitiveString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl PartialEq for CaseInsensitiveString {
    fn eq(&self, other: &Self) -> bool {
        CmpIgnoreAsciiCase::compare_ignore_ascii_case(self, other) == Ordering::Equal
    }
}

impl PartialEq<str> for CaseInsensitiveString {
    fn eq(&self, other: &str) -> bool {
        CmpIgnoreAsciiCase::compare_ignore_ascii_case(self.inner.as_bytes(), other.as_bytes())
            == Ordering::Equal
    }
}

impl Eq for CaseInsensitiveString {}

impl std::hash::Hash for CaseInsensitiveString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for b in self.inner.as_bytes() {
            b.to_ascii_uppercase().hash(state);
        }
    }
}

// CmpIgnoreAsciiCase

pub trait CmpIgnoreAsciiCase {
    fn compare_ignore_ascii_case(left: Self, right: Self) -> Ordering;
}

impl CmpIgnoreAsciiCase for u8 {
    fn compare_ignore_ascii_case(left: u8, right: u8) -> Ordering {
        let l_upper: u8 = left.to_ascii_uppercase();
        let r_upper: u8 = right.to_ascii_uppercase();
        l_upper.cmp(&r_upper)
    }
}

impl CmpIgnoreAsciiCase for &[u8] {
    fn compare_ignore_ascii_case(left: &[u8], right: &[u8]) -> Ordering {
        let mut i = 0;
        while i < left.len() && i < right.len() {
            let ord = CmpIgnoreAsciiCase::compare_ignore_ascii_case(left[i], right[i]);
            if ord != Ordering::Equal {
                return ord;
            }
            i += 1;
        }

        if i < right.len() {
            Ordering::Less
        } else if i < left.len() {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl CmpIgnoreAsciiCase for &str {
    fn compare_ignore_ascii_case(left: &str, right: &str) -> Ordering {
        let l_bytes: &[u8] = left.as_bytes();
        let r_bytes: &[u8] = right.as_bytes();
        CmpIgnoreAsciiCase::compare_ignore_ascii_case(l_bytes, r_bytes)
    }
}

impl CmpIgnoreAsciiCase for &String {
    fn compare_ignore_ascii_case(left: &String, right: &String) -> Ordering {
        let l_bytes: &[u8] = left.as_bytes();
        let r_bytes: &[u8] = right.as_bytes();
        CmpIgnoreAsciiCase::compare_ignore_ascii_case(l_bytes, r_bytes)
    }
}

impl CmpIgnoreAsciiCase for &CaseInsensitiveString {
    fn compare_ignore_ascii_case(
        left: &CaseInsensitiveString,
        right: &CaseInsensitiveString,
    ) -> Ordering {
        let l_inner: &String = &left.inner;
        let r_inner: &String = &right.inner;
        CmpIgnoreAsciiCase::compare_ignore_ascii_case(l_inner, r_inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    #[test]
    fn test_case_insensitive_string() {
        let x: CaseInsensitiveString = "abcDEF".into();
        let y: CaseInsensitiveString = "ABCdef".into();
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

    #[test]
    fn test_cmp_ignore_ascii_case() {
        assert_eq!(
            CmpIgnoreAsciiCase::compare_ignore_ascii_case("abc", "abc"),
            Ordering::Equal
        );
        assert_eq!(
            CmpIgnoreAsciiCase::compare_ignore_ascii_case("abc", "ABC"),
            Ordering::Equal
        );
        assert_eq!(
            CmpIgnoreAsciiCase::compare_ignore_ascii_case("ABC", "abc"),
            Ordering::Equal
        );
        assert_eq!(
            CmpIgnoreAsciiCase::compare_ignore_ascii_case("ABC", "ABC"),
            Ordering::Equal
        );

        assert_eq!(
            CmpIgnoreAsciiCase::compare_ignore_ascii_case("abc", "def"),
            Ordering::Less
        );
        assert_eq!(
            CmpIgnoreAsciiCase::compare_ignore_ascii_case("abc", "DEF"),
            Ordering::Less
        );
        assert_eq!(
            CmpIgnoreAsciiCase::compare_ignore_ascii_case("ABC", "def"),
            Ordering::Less
        );
        assert_eq!(
            CmpIgnoreAsciiCase::compare_ignore_ascii_case("ABC", "DEF"),
            Ordering::Less
        );

        assert_eq!(
            CmpIgnoreAsciiCase::compare_ignore_ascii_case("xyz", "def"),
            Ordering::Greater
        );
        assert_eq!(
            CmpIgnoreAsciiCase::compare_ignore_ascii_case("xyz", "DEF"),
            Ordering::Greater
        );
        assert_eq!(
            CmpIgnoreAsciiCase::compare_ignore_ascii_case("XYZ", "def"),
            Ordering::Greater
        );
        assert_eq!(
            CmpIgnoreAsciiCase::compare_ignore_ascii_case("XYZ", "DEF"),
            Ordering::Greater
        );

        assert_eq!(
            CmpIgnoreAsciiCase::compare_ignore_ascii_case("abc", "abca"),
            Ordering::Less
        );
        assert_eq!(
            CmpIgnoreAsciiCase::compare_ignore_ascii_case("abca", "abc"),
            Ordering::Greater
        );
    }
}
