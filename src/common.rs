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

impl<T> Locatable<T> {
    pub fn new(element: T, location: Location) -> Locatable<T> {
        Locatable { element, location }
    }

    pub fn element(&self) -> &T {
        &self.element
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

pub trait AddLocation<T: HasLocation> {
    fn add_location(&self, pos: Location) -> T;
}

impl<T: Clone> AddLocation<Locatable<T>> for T {
    fn add_location(&self, pos: Location) -> Locatable<T> {
        Locatable::new(self.clone(), pos)
    }
}

impl<T, TL> AddLocation<Box<TL>> for Box<T>
where
    TL: HasLocation,
    T: AddLocation<TL>,
{
    fn add_location(&self, pos: Location) -> Box<TL> {
        let inside_the_box: &T = self;
        Box::new(inside_the_box.add_location(pos))
    }
}

// StripLocationRef

pub trait StripLocationRef<T> {
    fn strip_location_ref(&self) -> &T;
}

impl<T> StripLocationRef<T> for Locatable<T> {
    fn strip_location_ref(&self) -> &T {
        self.element()
    }
}

// StripLocation

pub trait StripLocation<T> {
    fn strip_location(&self) -> T;
}

impl<T, TL: StripLocation<T>> StripLocation<Vec<T>> for Vec<TL> {
    fn strip_location(&self) -> Vec<T> {
        self.iter().map(|x| x.strip_location()).collect()
    }
}

impl<T, TL> StripLocation<Box<T>> for Box<TL>
where
    TL: HasLocation,
    TL: StripLocation<T>,
{
    fn strip_location(&self) -> Box<T> {
        let inside_the_box: &TL = self;
        Box::new(inside_the_box.strip_location())
    }
}

impl<T, TL> StripLocation<Option<T>> for Option<TL>
where
    TL: StripLocation<T>,
{
    fn strip_location(&self) -> Option<T> {
        match self {
            Some(x) => Some(x.strip_location()),
            None => None,
        }
    }
}
