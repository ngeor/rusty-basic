//
// Location
//

use std::ops::Deref;
use tokenizers::RowCol;

/// The location of a token within a text file, expressed in row and column.
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

    pub fn row(&self) -> u32 {
        self.row
    }

    pub fn col(&self) -> u32 {
        self.col
    }
}

impl From<RowCol> for Location {
    fn from(row_col: RowCol) -> Self {
        Self::new(row_col.row, row_col.col)
    }
}

//
// Locatable
//

/// Bundles an element (typically a parsed token) together with its location
/// within the file it was read from.
#[derive(Clone, Debug, PartialEq)]
pub struct Locatable<T> {
    pub element: T,
    pub pos: Location,
}

impl<T> Locatable<T> {
    pub fn new(element: T, pos: Location) -> Self {
        Locatable { element, pos }
    }

    pub fn map<F, U>(self, op: F) -> Locatable<U>
    where
        F: FnOnce(T) -> U,
    {
        let Locatable { element, pos } = self;
        let mapped: U = op(element);
        Locatable {
            element: mapped,
            pos,
        }
    }
}

impl<T> AsRef<T> for Locatable<T> {
    fn as_ref(&self) -> &T {
        &self.element
    }
}

impl<T: PartialEq<T>> PartialEq<T> for Locatable<T> {
    fn eq(&self, that: &T) -> bool {
        &self.element == that
    }
}

impl<T> Deref for Locatable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.element
    }
}

// AtLocation

pub trait AtLocation<T, TLocation> {
    fn at(self, pos: TLocation) -> T;
}

impl<T> AtLocation<Locatable<T>, Location> for T {
    fn at(self, pos: Location) -> Locatable<T> {
        Locatable::new(self, pos)
    }
}

impl<T, TLocation: HasLocation> AtLocation<Locatable<T>, &TLocation> for T {
    fn at(self, pos: &TLocation) -> Locatable<T> {
        Locatable::new(self, pos.pos())
    }
}

pub trait AtRowCol<T> {
    fn at_rc(self, row: u32, col: u32) -> T;
}

impl<T, U> AtRowCol<U> for T
where
    T: AtLocation<U, Location>,
{
    fn at_rc(self, row: u32, col: u32) -> U {
        self.at(Location::new(row, col))
    }
}

//
// HasLocation
//

pub trait HasLocation {
    fn pos(&self) -> Location;
}

impl<T> HasLocation for Locatable<T> {
    fn pos(&self) -> Location {
        self.pos
    }
}

impl HasLocation for Location {
    fn pos(&self) -> Location {
        *self
    }
}

//
// StripLocation
//

pub trait StripLocation<T> {
    fn strip_location(self) -> T;
}

impl<T> StripLocation<T> for Locatable<T> {
    fn strip_location(self) -> T {
        self.element
    }
}

impl<T> StripLocation<Vec<T>> for Vec<Locatable<T>> {
    fn strip_location(self) -> Vec<T> {
        self.into_iter().map(|x| x.strip_location()).collect()
    }
}
