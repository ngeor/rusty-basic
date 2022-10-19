use crate::common::QErrorNode;

pub trait SameTypeConverterInContext<T, U> {
    fn convert_in_context(&mut self, item: T, context: U) -> Result<T, QErrorNode>;
}

// blanket for Vec
impl<X, T, U> SameTypeConverterInContext<Vec<T>, U> for X
where
    X: SameTypeConverterInContext<T, U>,
    U: Copy,
{
    fn convert_in_context(&mut self, items: Vec<T>, context: U) -> Result<Vec<T>, QErrorNode> {
        items
            .into_iter()
            .map(|item| self.convert_in_context(item, context))
            .collect()
    }
}
