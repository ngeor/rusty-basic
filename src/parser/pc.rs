pub trait IsNotFoundErr {
    fn is_not_found_err(&self) -> bool;
}

pub trait NotFoundErr: IsNotFoundErr {
    fn not_found_err() -> Self;
}

pub trait Undo<T> {
    fn undo(self, item: T) -> Self;
}

pub trait Reader: Sized {
    type Item;
    type Err;
    fn read(self) -> (Self, Result<Self::Item, Self::Err>);
    fn undo_item(self, item: Self::Item) -> Self;
}

mod loc {
    // for Reader + HasLocation
}
