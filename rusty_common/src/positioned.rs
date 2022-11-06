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

    /// Converts this instance to a result, given the specified function.
    /// The function acts on the element of this instance and returns
    /// the result. The error of the result is converted to a [Positioned] error.
    pub fn try_map<F, U, E>(&self, op: F) -> Result<U, Positioned<E>>
    where
        F: FnOnce(&T) -> Result<U, E>,
    {
        (op)(&self.element).map_err(|e| e.at_pos(self.pos))
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

    fn at_start(self) -> Positioned<Self> {
        self.at_pos(Position::start())
    }

    fn at_no_pos(self) -> Positioned<Self> {
        self.at_pos(Position::zero())
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

//
// NoPosIter
//

pub struct NoPosIter<'a, T: 'a, I>
where
    I: Iterator<Item = &'a Positioned<T>>,
{
    pos_iter: I,
}

impl<'a, T: 'a, I> Iterator for NoPosIter<'a, T, I>
where
    I: Iterator<Item = &'a Positioned<T>>,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos_iter.next().map(|positioned| &positioned.element)
    }
}

pub trait NoPosIterTrait<'a, T: 'a>: Iterator<Item = &'a Positioned<T>> {
    fn no_pos(self) -> NoPosIter<'a, T, Self>
    where
        Self: Sized,
    {
        NoPosIter { pos_iter: self }
    }
}

impl<'a, T: 'a, I> NoPosIterTrait<'a, T> for I where I: Iterator<Item = &'a Positioned<T>> {}

//
// NoPosIntoIter
//

pub struct NoPosIntoIter<T, I>
where
    I: Iterator<Item = Positioned<T>>,
{
    pos_iter: I,
}

impl<T, I> Iterator for NoPosIntoIter<T, I>
where
    I: Iterator<Item = Positioned<T>>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos_iter.next().map(|positioned| positioned.element)
    }
}

pub trait NoPosIntoIterTrait<T>: Iterator<Item = Positioned<T>> {
    fn no_pos(self) -> NoPosIntoIter<T, Self>
    where
        Self: Sized,
    {
        NoPosIntoIter { pos_iter: self }
    }
}

impl<T, I> NoPosIntoIterTrait<T> for I where I: Iterator<Item = Positioned<T>> {}

//
// NoPosContainer
//

pub trait NoPosContainer {
    type Output;
    fn no_pos(self) -> Self::Output;
}

impl<T> NoPosContainer for Vec<Positioned<T>> {
    type Output = Vec<T>;

    fn no_pos(self) -> Self::Output {
        self.into_iter().no_pos().collect()
    }
}
