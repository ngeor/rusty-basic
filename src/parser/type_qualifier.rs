use crate::parser::pc::Reader;
use crate::parser::pc2::unary::UnaryParser;
use crate::parser::pc2::{any_p, Parser};
use crate::parser::TypeQualifier;

/// Returns a parser that can parse a `TypeQualifier`.
pub fn type_qualifier_p<R>() -> impl Parser<R, Output = TypeQualifier>
where
    R: Reader<Item = char>,
{
    any_p::<R>().try_from::<TypeQualifier>()
}
