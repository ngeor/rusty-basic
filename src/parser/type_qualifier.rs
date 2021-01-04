use crate::parser::pc::unary::UnaryParser;
use crate::parser::pc::{any_p, Parser, Reader, Undo};
use crate::parser::TypeQualifier;

/// Returns a parser that can parse a `TypeQualifier`.
pub fn type_qualifier_p<R>() -> impl Parser<R, Output = TypeQualifier>
where
    R: Reader<Item = char>,
{
    any_p::<R>().try_from::<TypeQualifier>()
}

impl<R: Reader<Item = char>> Undo<TypeQualifier> for R {
    fn undo(self, type_qualifier: TypeQualifier) -> Self {
        let ch = char::from(type_qualifier);
        self.undo_item(ch)
    }
}
