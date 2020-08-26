// ========================================================
// traits
// ========================================================

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

// ========================================================
// simple parsing functions
// ========================================================

/// Returns a function that gets the next item from a reader.
pub fn read_any<R: Reader + 'static>() -> impl Fn(R) -> (R, Result<R::Item, R::Err>) {
    |reader| reader.read()
}

/// Returns a function that gets the next item from a reader, ensuring that
/// it is not a Not Found result.
pub fn read_some<R: Reader + 'static, FE>(
    err_fn: FE,
) -> Box<dyn Fn(R) -> (R, Result<R::Item, R::Err>)>
where
    FE: Fn() -> R::Err + 'static,
    R::Err: IsNotFoundErr,
{
    demand(read_any(), err_fn)
}

// ========================================================
// simple parsing combinators
// ========================================================

/// Returns a function that ensures that we don't get a Not Found result from
/// the given source.
///
/// Not found results are converted to the error provided from the given function.
pub fn demand<R, S, FE, T, E>(source: S, err_fn: FE) -> Box<dyn Fn(R) -> (R, Result<T, E>)>
where
    R: Reader<Err = E> + 'static,
    S: Fn(R) -> (R, Result<T, E>) + 'static,
    FE: Fn() -> E + 'static,
    E: IsNotFoundErr + 'static,
{
    Box::new(move |reader| {
        let (reader, next) = source(reader);
        match next {
            Ok(x) => (reader, Ok(x)),
            Err(err) => {
                if err.is_not_found_err() {
                    (reader, Err(err_fn()))
                } else {
                    (reader, Err(err))
                }
            }
        }
    })
}

// ========================================================
// when Item : Copy
// ========================================================

// ========================================================
// when Item = char
// ========================================================

mod ch {}

// ========================================================
// when Reader + HasLocation
// ========================================================

mod loc {}
