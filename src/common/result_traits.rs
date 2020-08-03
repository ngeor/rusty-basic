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
