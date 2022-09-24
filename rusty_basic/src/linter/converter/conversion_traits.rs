use super::{Implicits, R};

/// Converts an object into another, possibly collecting implicitly defined
/// variables along the way.
///
/// Example: `INPUT N` is a statement that implicitly declares variable `N`.
pub trait SameTypeConverterWithImplicits<T> {
    fn convert_same_type_with_implicits(&mut self, item: T) -> R<T>;
}

// blanket for Vec
impl<X, T> SameTypeConverterWithImplicits<Vec<T>> for X
where
    X: SameTypeConverterWithImplicits<T>,
{
    fn convert_same_type_with_implicits(&mut self, items: Vec<T>) -> R<Vec<T>> {
        let mut result: Vec<T> = vec![];
        let mut total_implicit: Implicits = vec![];
        for item in items {
            let (converted_item, mut implicit) = self.convert_same_type_with_implicits(item)?;
            result.push(converted_item);
            total_implicit.append(&mut implicit);
        }
        Ok((result, total_implicit))
    }
}

pub trait SameTypeConverterWithImplicitsInContext<T, U> {
    fn convert_same_type_with_implicits_in_context(&mut self, item: T, context: U) -> R<T>;
}

// blanket for Vec
impl<X, T, U> SameTypeConverterWithImplicitsInContext<Vec<T>, U> for X
where
    X: SameTypeConverterWithImplicitsInContext<T, U>,
    U: Copy,
{
    fn convert_same_type_with_implicits_in_context(
        &mut self,
        items: Vec<T>,
        context: U,
    ) -> R<Vec<T>> {
        let mut result: Vec<T> = vec![];
        let mut total_implicit: Implicits = vec![];
        for item in items {
            let (converted_item, mut implicit) =
                self.convert_same_type_with_implicits_in_context(item, context)?;
            result.push(converted_item);
            total_implicit.append(&mut implicit);
        }
        Ok((result, total_implicit))
    }
}
