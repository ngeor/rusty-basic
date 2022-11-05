use crate::Position;

/// Bundles an element (typically a parsed token) together with its position
/// within the file it was read from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Positioned<T> {
    // TODO make fields private
    pub element: T,
    pub pos: Position,
}

impl<T> Positioned<T> {
    pub fn new(element: T, pos: Position) -> Self {
        Positioned { element, pos }
    }

    pub fn element(self) -> T {
        self.element
    }

    pub fn map<F, U>(self, op: F) -> Positioned<U>
    where
        F: FnOnce(T) -> U,
    {
        let Positioned { element, pos } = self;
        let mapped: U = op(element);
        Positioned {
            element: mapped,
            pos,
        }
    }

    // TODO: #[cfg(test)]
    pub fn no_pos(items: Vec<Self>) -> Vec<T> {
        items.into_iter().map(Self::element).collect()
    }
}

// AtPos

pub trait AtPos: Sized {
    fn at_pos(self, pos: Position) -> Positioned<Self> {
        Positioned::new(self, pos)
    }

    fn at<T: HasPos>(self, pos: &T) -> Positioned<Self> {
        self.at_pos(pos.pos())
    }

    fn at_rc(self, row: u32, col: u32) -> Positioned<Self> {
        self.at_pos(Position::new(row, col))
    }
}

impl<T> AtPos for T {}

//
// HasPos
//

pub trait HasPos {
    fn pos(&self) -> Position;
}

impl<T> HasPos for Positioned<T> {
    fn pos(&self) -> Position {
        self.pos
    }
}

impl HasPos for Position {
    fn pos(&self) -> Position {
        *self
    }
}

impl<T> HasPos for Box<Positioned<T>> {
    fn pos(&self) -> Position {
        self.as_ref().pos()
    }
}
