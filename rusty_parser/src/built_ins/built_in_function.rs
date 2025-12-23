use crate::keyword_enum;
use crate::specific::{QualifiedName, TypeQualifier};
use rusty_common::*;
use std::borrow::Borrow;
use std::convert::TryFrom;

keyword_enum!(
pub enum BuiltInFunction SORTED_BUILT_IN_FUNCTIONS SORTED_BUILT_IN_FUNCTION_NAMES SORTED_BUILT_IN_FUNCTIONS_CI_STR {
    /// `CHR$(ascii-code%)` returns the text representation of the given ascii code
    Chr,

    /// `CVD(8 byte string-expression)`
    ///
    /// Converts a string previously created by `MKD$` into a double.
    Cvd,

    /// `ENVIRON$ (env-variable$)` -> returns the variable
    ///
    /// `ENVIRON$ (n%)` -> returns the nth variable (TODO support this)
    Environ,

    /// `EOF(file-number%)` -> checks if the end of file has been reached
    Eof,

    /// `ERR`
    Err,

    /// `INKEY$`
    ///
    /// Reads a character from the keyboard.
    InKey,

    /// `INSTR([start%,] hay$, needle$)`
    /// if start% is omitted, INSTR starts at position 1
    /// returns the first occurrence of needle$ inside hay$
    InStr,

    /// LBOUND
    LBound,

    /// LCASE$
    LCase,

    /// `LEFT$(str_expr$, count%)`
    Left,

    /// `LEN(str_expr$)` -> number of characters in string
    ///
    /// `LEN(variable)` -> number of bytes required to store a variable
    Len,

    /// `LTRIM$`
    LTrim,

    /// MID$ function returns part of a string
    ///
    /// MID$ statement replaces part of a string (TODO support this)
    ///
    /// MID$(str_expr$, start%[, length%])
    ///
    /// MID$(str_var$, start%[, length%]) = str_expr$
    ///
    /// if the length is omitted, returns or replaces all remaining characters
    Mid,

    /// `MKD$(double-expression#)`
    ///
    /// Converts a double precision number into an 8 byte string that can be
    /// used in `FIELD` statements.
    Mkd,

    /// `PEEK`
    Peek,

    /// `RIGHT$(str_expr$, count%)`
    Right,

    /// `RTRIM$`
    RTrim,

    /// `SPACE$(number-of-spaces)`
    Space,

    /// `STR$(numeric-expression)` returns a string representation of a number
    Str,

    /// Returns a string of a specified length made up of a repeating character.
    ///
    /// `STRING$(length%, { ascii-code% | string-expression$ })`
    ///
    /// - `length%` The length of the string
    /// - `ascii-code%` The ASCII code of the repeating character
    /// - `string-expression$` Any string expression. Only the first character will be used.
    String,

    /// UBOUND
    UBound,

    /// UCASE$
    UCase,

    /// `VAL(str-expr$)` converts a string representation of a number to a number.
    Val,

    /// `VARPTR`
    VarPtr,

    /// `VARSEG`
    VarSeg,
});

// BuiltInFunction -> CaseInsensitiveString

impl From<BuiltInFunction> for CaseInsensitiveString {
    fn from(x: BuiltInFunction) -> Self {
        Self::from(x.as_ref())
    }
}

// BuiltInFunction -> TypeQualifier

impl From<&BuiltInFunction> for TypeQualifier {
    fn from(x: &BuiltInFunction) -> TypeQualifier {
        match x {
            BuiltInFunction::Chr => TypeQualifier::DollarString,
            BuiltInFunction::Cvd => TypeQualifier::HashDouble,
            BuiltInFunction::Environ => TypeQualifier::DollarString,
            BuiltInFunction::Eof => TypeQualifier::PercentInteger,
            BuiltInFunction::Err => TypeQualifier::PercentInteger,
            BuiltInFunction::InKey => TypeQualifier::DollarString,
            BuiltInFunction::InStr => TypeQualifier::PercentInteger,
            BuiltInFunction::LBound => TypeQualifier::PercentInteger,
            BuiltInFunction::LCase => TypeQualifier::DollarString,
            BuiltInFunction::Left => TypeQualifier::DollarString,
            BuiltInFunction::Len => TypeQualifier::PercentInteger,
            BuiltInFunction::LTrim => TypeQualifier::DollarString,
            BuiltInFunction::Mid => TypeQualifier::DollarString,
            BuiltInFunction::Mkd => TypeQualifier::DollarString,
            BuiltInFunction::Peek => TypeQualifier::PercentInteger,
            BuiltInFunction::Right => TypeQualifier::DollarString,
            BuiltInFunction::RTrim => TypeQualifier::DollarString,
            BuiltInFunction::Space => TypeQualifier::DollarString,
            BuiltInFunction::Str => TypeQualifier::DollarString,
            BuiltInFunction::String => TypeQualifier::DollarString,
            BuiltInFunction::UBound => TypeQualifier::PercentInteger,
            BuiltInFunction::UCase => TypeQualifier::DollarString,
            BuiltInFunction::Val => TypeQualifier::BangSingle,
            BuiltInFunction::VarPtr => TypeQualifier::PercentInteger,
            BuiltInFunction::VarSeg => TypeQualifier::PercentInteger,
        }
    }
}

// BuiltInFunction -> QualifiedName

impl From<BuiltInFunction> for QualifiedName {
    fn from(built_in_function: BuiltInFunction) -> Self {
        let qualifier: TypeQualifier = (&built_in_function).into();
        Self::new(built_in_function.into(), qualifier)
    }
}

// CaseInsensitiveString -> BuiltInFunction

impl BuiltInFunction {
    pub fn try_parse<S>(s: &S) -> Option<BuiltInFunction>
    where
        S: Borrow<CaseInsensitiveStr>,
    {
        Self::try_from(s.borrow()).ok()
    }
}
