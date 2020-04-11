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

    #[cfg(test)]
    pub fn zero() -> Location {
        Location::new(0, 0)
    }

    pub fn start() -> Location {
        Location::new(1, 1)
    }
}

// Locatable

#[derive(Clone, Debug, PartialEq)]
pub struct Locatable<T> {
    element: T,
    location: Location,
}

impl<T: Sized> Locatable<T> {
    pub fn new(element: T, location: Location) -> Locatable<T> {
        Locatable { element, location }
    }

    pub fn element(&self) -> &T {
        &self.element
    }

    pub fn element_into(self) -> T {
        self.element
    }

    pub fn map<U: Sized, F>(&self, f: F) -> Locatable<U>
    where
        F: Fn(&T) -> U,
    {
        Locatable::new(f(&self.element), self.location)
    }

    pub fn map_into<U: Sized, F>(self, f: F) -> Locatable<U>
    where
        F: Fn(T) -> U,
    {
        Locatable::new(f(self.element), self.location)
    }

    pub fn at(self, new_location: Location) -> Self {
        Locatable::new(self.element, new_location)
    }
}

// HasLocation

pub trait HasLocation {
    fn location(&self) -> Location;
}

impl<T> HasLocation for Locatable<T> {
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

// AddLocation

pub trait AddLocation<T> {
    fn add_location(self, pos: Location) -> T;
}

impl<T, TL> AddLocation<Vec<TL>> for Vec<T>
where
    TL: HasLocation,
    T: AddLocation<TL>,
{
    fn add_location(self, pos: Location) -> Vec<TL> {
        self.into_iter().map(|x| x.add_location(pos)).collect()
    }
}

impl<T: Copy> AddLocation<Locatable<T>> for T {
    fn add_location(self, pos: Location) -> Locatable<T> {
        Locatable::new(self, pos)
    }
}

impl<T, TL> AddLocation<Box<TL>> for Box<T>
where
    TL: HasLocation,
    T: AddLocation<TL>,
{
    fn add_location(self, pos: Location) -> Box<TL> {
        let inside_the_box: T = *self;
        Box::new(inside_the_box.add_location(pos))
    }
}

// StripLocation

pub trait StripLocation<T> {
    fn strip_location(self) -> T;
}

impl<T, TL: StripLocation<T>> StripLocation<Vec<T>> for Vec<TL> {
    fn strip_location(self) -> Vec<T> {
        self.into_iter().map(|x| x.strip_location()).collect()
    }
}

impl<T, TL> StripLocation<Box<T>> for Box<TL>
where
    TL: HasLocation,
    TL: StripLocation<T>,
{
    fn strip_location(self) -> Box<T> {
        let inside_the_box: TL = *self;
        Box::new(inside_the_box.strip_location())
    }
}

impl<T, TL> StripLocation<Option<T>> for Option<TL>
where
    TL: StripLocation<T>,
{
    fn strip_location(self) -> Option<T> {
        match self {
            Some(x) => Some(x.strip_location()),
            None => None,
        }
    }
}

impl<T> StripLocation<T> for Locatable<T>
where
    T: Clone,
{
    fn strip_location(self) -> T {
        self.element().clone()
    }
}

// CaseInsensitiveString

#[derive(Clone, Debug)]
pub struct CaseInsensitiveString {
    inner: String,
    upper: String,
}

impl CaseInsensitiveString {
    pub fn new(value: String) -> CaseInsensitiveString {
        let upper = value.to_uppercase();
        CaseInsensitiveString {
            inner: value,
            upper,
        }
    }

    pub fn eq(&self, other: &str) -> bool {
        self.upper == other.to_uppercase()
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
        self.upper == other.upper
    }
}

impl PartialEq<&str> for CaseInsensitiveString {
    fn eq(&self, other: &&str) -> bool {
        self.upper == other.to_uppercase()
    }
}

impl PartialEq<&str> for &CaseInsensitiveString {
    fn eq(&self, other: &&str) -> bool {
        self.upper == other.to_uppercase()
    }
}

impl Eq for CaseInsensitiveString {}

impl std::hash::Hash for CaseInsensitiveString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.upper.hash(state);
    }
}

impl AsRef<str> for CaseInsensitiveString {
    fn as_ref(&self) -> &str {
        self.inner.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_insensitive_string() {
        let x: CaseInsensitiveString = "abcDEF".into();
        let y: CaseInsensitiveString = "ABCdef".into();
        assert_eq!("abcDEF".to_string(), x.to_string());
        assert_eq!("ABCdef".to_string(), y.to_string());
        assert_eq!(x, y);
    }
}
