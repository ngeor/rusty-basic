use crate::parser::pc::Reader;
use crate::parser::pc2::{read_one_if_try_from_p, ReadOneIfTryFrom};
use crate::parser::TypeQualifier;

/// Returns a parser that can parse a `TypeQualifier`.
pub fn type_qualifier_p<R>() -> ReadOneIfTryFrom<R, TypeQualifier>
where
    R: Reader<Item = char>,
{
    read_one_if_try_from_p::<R, TypeQualifier>()
}
