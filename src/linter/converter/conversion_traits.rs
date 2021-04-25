use super::{Implicits, R};
use crate::common::QErrorNode;

/// Converts an item to a different item of the same type.
pub trait SameTypeConverter<T> {
    fn convert_same_type(&mut self, item: T) -> Result<T, QErrorNode>;
}

// blanket for Option
impl<X, T> SameTypeConverter<Option<T>> for X
where
    X: SameTypeConverter<T>,
{
    fn convert_same_type(&mut self, item: Option<T>) -> Result<Option<T>, QErrorNode> {
        match item {
            Some(t) => self.convert_same_type(t).map(Some),
            _ => Ok(None),
        }
    }
}

/// Converts an item to multiple items of the same type.
pub trait OneToManyConverter<T> {
    fn convert_to_many(&mut self, item: T) -> Result<Vec<T>, QErrorNode>;
}

// blanket for Vec<T> -> Vec<T>, if T -> Vec<T> exists
impl<X, T> SameTypeConverter<Vec<T>> for X
where
    X: OneToManyConverter<T>,
{
    fn convert_same_type(&mut self, item: Vec<T>) -> Result<Vec<T>, QErrorNode> {
        let mut items: Vec<T> = vec![];
        for t in item {
            let mut expanded = self.convert_to_many(t)?;
            items.append(&mut expanded);
        }
        Ok(items)
    }
}

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
