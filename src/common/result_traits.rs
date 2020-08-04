pub trait ChainTryRead: Sized {
    fn or_try_read<F>(self, op: F) -> Self
    where
        F: FnMut() -> Self;
}

impl<T, E> ChainTryRead for Result<Option<T>, E> {
    fn or_try_read<F>(self, mut op: F) -> Self
    where
        F: FnMut() -> Self,
    {
        match &self {
            Ok(opt) => match opt {
                Some(_) => self,
                None => op(),
            },
            Err(_) => self,
        }
    }
}

pub trait TerminalTryRead<T>: Sized {
    fn or_read<F>(self, op: F) -> T
    where
        F: FnMut() -> T;
}

impl<T, E> TerminalTryRead<Result<T, E>> for Result<Option<T>, E> {
    fn or_read<F>(self, mut op: F) -> Result<T, E>
    where
        F: FnMut() -> Result<T, E>,
    {
        match self {
            Ok(opt) => match opt {
                Some(x) => Ok(x),
                None => op(),
            },
            Err(e) => Err(e),
        }
    }
}
