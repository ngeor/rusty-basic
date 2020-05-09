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

    pub fn consume(self) -> (T, Location) {
        (self.element, self.location)
    }
}

impl<T: std::fmt::Debug + Sized> AsRef<T> for Locatable<T> {
    fn as_ref(&self) -> &T {
        &self.element
    }
}

// AtLocation

pub trait AtLocation<T> {
    fn at(self, pos: Location) -> T;
}

impl<T: std::fmt::Debug + Sized> AtLocation<Locatable<T>> for T {
    fn at(self, pos: Location) -> Locatable<T> {
        Locatable::new(self, pos)
    }
}

pub trait AtRowCol<T> {
    fn at_rc(self, row: u32, col: u32) -> T;
}

impl<T, U> AtRowCol<U> for T
where
    T: AtLocation<U>,
{
    fn at_rc(self, row: u32, col: u32) -> U {
        self.at(Location::new(row, col))
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

// StripLocation

pub trait StripLocationRef<T> {
    fn strip_location(&self) -> &T;
}

pub trait StripLocationVal<T> {
    fn strip_location(self) -> T;
}

impl<T: std::fmt::Debug + Sized> StripLocationRef<T> for Locatable<T> {
    fn strip_location(&self) -> &T {
        &self.element
    }
}

impl<T: std::fmt::Debug + Sized> StripLocationVal<T> for Locatable<T> {
    fn strip_location(self) -> T {
        self.element
    }
}

impl<T: std::fmt::Debug + Sized> StripLocationVal<Vec<T>> for Vec<Locatable<T>> {
    fn strip_location(self) -> Vec<T> {
        self.into_iter().map(|x| x.strip_location()).collect()
    }
}
