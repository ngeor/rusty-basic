pub trait Convertible {
    type Context;
    type Output;
    type Error;

    fn convert(&self, context: &Self::Context) -> Result<Self::Output, Self::Error>;
}

impl<T> Convertible for Vec<T>
where
    T: Convertible,
{
    type Context = T::Context;
    type Output = Vec<T::Output>;
    type Error = T::Error;

    fn convert(&self, context: &Self::Context) -> Result<Self::Output, Self::Error> {
        self.iter().map(|item| item.convert(context)).collect()
    }
}
