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
    fn convert_same_type_with_implicits(&mut self, a: T) -> R<T>;
}

// blanket for Option
impl<T, A> SameTypeConverterWithImplicits<Option<A>> for T
where
    T: SameTypeConverterWithImplicits<A>,
{
    fn convert_same_type_with_implicits(&mut self, a: Option<A>) -> R<Option<A>> {
        match a {
            Some(a) => self
                .convert_same_type_with_implicits(a)
                .map(|(a, implicit_variables)| (Some(a), implicit_variables)),
            None => Ok((None, vec![])),
        }
    }
}

// blanket for Box
impl<T, A> SameTypeConverterWithImplicits<Box<A>> for T
where
    T: SameTypeConverterWithImplicits<A>,
{
    fn convert_same_type_with_implicits(&mut self, a: Box<A>) -> R<Box<A>> {
        let unboxed: A = *a;
        let (converted, implicit_variables) = self.convert_same_type_with_implicits(unboxed)?;
        Ok((Box::new(converted), implicit_variables))
    }
}

// blanket for Vec
impl<T, A> SameTypeConverterWithImplicits<Vec<A>> for T
where
    T: SameTypeConverterWithImplicits<A>,
{
    fn convert_same_type_with_implicits(&mut self, a: Vec<A>) -> R<Vec<A>> {
        let mut result: Vec<A> = vec![];
        let mut total_implicit: Implicits = vec![];
        for i in a {
            let (b, mut implicit) = self.convert_same_type_with_implicits(i)?;
            result.push(b);
            total_implicit.append(&mut implicit);
        }
        Ok((result, total_implicit))
    }
}
