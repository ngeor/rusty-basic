use crate::parser::base::parsers::Parser;
use crate::parser::TypeQualifier;

/// Returns a parser that can parse a `TypeQualifier`.
pub fn type_qualifier_p<R>() -> impl Parser<Output = TypeQualifier> {
    any_p::<R>().try_from::<TypeQualifier>()
}
