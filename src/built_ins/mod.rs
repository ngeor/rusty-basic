use crate::common::*;
use crate::parser::{Name, QualifiedName, TypeQualifier};
use std::convert::TryFrom;

// ========================================================
// BuiltInFunction
// ========================================================

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum BuiltInFunction {
    /// CHR$
    Chr,
    /// ENVIRON$
    Environ,
    /// EOF
    Eof,
    /// INSTR
    InStr,
    /// LBOUND
    LBound,
    /// LEN
    Len,
    /// MID$
    Mid,
    /// STR$
    Str,
    /// UBOUND
    UBound,
    /// VAL
    Val,
}

const SORTED_BUILT_IN_FUNCTIONS: [BuiltInFunction; 10] = [
    BuiltInFunction::Chr,
    BuiltInFunction::Environ,
    BuiltInFunction::Eof,
    BuiltInFunction::InStr,
    BuiltInFunction::LBound,
    BuiltInFunction::Len,
    BuiltInFunction::Mid,
    BuiltInFunction::Str,
    BuiltInFunction::UBound,
    BuiltInFunction::Val,
];

const SORTED_BUILT_IN_FUNCTION_NAMES: [&str; 10] = [
    "Chr", "Environ", "Eof", "InStr", "LBound", "Len", "Mid", "Str", "UBound", "Val",
];

// BuiltInFunction -> &str

impl AsRef<str> for BuiltInFunction {
    fn as_ref(&self) -> &str {
        let idx = SORTED_BUILT_IN_FUNCTIONS
            .binary_search(self)
            .expect("Missing built-in function!");
        SORTED_BUILT_IN_FUNCTION_NAMES[idx]
    }
}

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
            BuiltInFunction::Environ => TypeQualifier::DollarString,
            BuiltInFunction::Eof => TypeQualifier::PercentInteger,
            BuiltInFunction::InStr => TypeQualifier::PercentInteger,
            BuiltInFunction::LBound => TypeQualifier::PercentInteger,
            BuiltInFunction::Len => TypeQualifier::PercentInteger,
            BuiltInFunction::Mid => TypeQualifier::DollarString,
            BuiltInFunction::Str => TypeQualifier::DollarString,
            BuiltInFunction::UBound => TypeQualifier::PercentInteger,
            BuiltInFunction::Val => TypeQualifier::BangSingle,
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

impl From<&CaseInsensitiveString> for Option<BuiltInFunction> {
    fn from(s: &CaseInsensitiveString) -> Option<BuiltInFunction> {
        match SORTED_BUILT_IN_FUNCTION_NAMES
            .binary_search_by(|p| CmpIgnoreAsciiCase::compare_ignore_ascii_case(*p, s.as_ref()))
        {
            Ok(idx) => Some(SORTED_BUILT_IN_FUNCTIONS[idx]),
            Err(_) => None,
        }
    }
}

// Name -> BuiltInFunction

impl TryFrom<&Name> for Option<BuiltInFunction> {
    type Error = QError;
    fn try_from(n: &Name) -> Result<Option<BuiltInFunction>, Self::Error> {
        let opt_built_in: Option<BuiltInFunction> = n.bare_name().into();
        match opt_built_in {
            Some(b) => match b {
                BuiltInFunction::Eof
                | BuiltInFunction::InStr
                | BuiltInFunction::Len
                | BuiltInFunction::LBound
                | BuiltInFunction::UBound
                | BuiltInFunction::Val => demand_unqualified(b, n),
                BuiltInFunction::Environ | BuiltInFunction::Mid => {
                    // ENVIRON$ must be qualified
                    match n {
                        Name::Bare(_) => Err(QError::SyntaxError(format!(
                            "Function {:?} must be qualified",
                            n
                        ))),
                        Name::Qualified(QualifiedName { qualifier, .. }) => {
                            if *qualifier == TypeQualifier::DollarString {
                                Ok(Some(b))
                            } else {
                                Err(QError::TypeMismatch)
                            }
                        }
                    }
                }
                BuiltInFunction::Chr | BuiltInFunction::Str => {
                    // STR$ or otherwise it's undefined
                    match n {
                        // confirmed that even with DEFSTR A-Z it won't work as unqualified
                        Name::Bare(_) => Ok(None),
                        Name::Qualified(QualifiedName { qualifier, .. }) => {
                            if *qualifier == TypeQualifier::DollarString {
                                Ok(Some(b))
                            } else {
                                Ok(None)
                            }
                        }
                    }
                }
            },
            None => Ok(None),
        }
    }
}

fn demand_unqualified(
    built_in: BuiltInFunction,
    n: &Name,
) -> Result<Option<BuiltInFunction>, QError> {
    match n {
        Name::Bare(_) => Ok(Some(built_in)),
        _ => Err(QError::SyntaxError(format!(
            "Function {:?} must be unqualified",
            built_in
        ))),
    }
}

// ========================================================
// BuiltInSub
// ========================================================

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuiltInSub {
    Close,
    Environ,
    Field,
    Get,
    Input,
    Kill,
    LineInput,
    LSet,
    Name,
    Open,
    Put,
    ViewPrint,
}

impl BuiltInSub {
    /// Parses a built-in sub name which isn't implemented with a keyword.
    /// This sub would appear as a user defined SUB on the parser layer.
    ///
    /// Some statements are implemented a built-in subs (e.g. `CLOSE`, `OPEN`), but
    /// they can't hit this function, as they are represented by keywords and are
    /// parsed by custom parsers.
    pub fn parse_non_keyword_sub(s: &str) -> Option<BuiltInSub> {
        if s.eq_ignore_ascii_case("Environ") {
            Some(BuiltInSub::Environ)
        } else if s.eq_ignore_ascii_case("Kill") {
            Some(BuiltInSub::Kill)
        } else {
            None
        }
    }
}

pub mod parser {
    use crate::common::*;
    use crate::parser::pc::*;
    use crate::parser::Statement;

    /// Parses built-in subs which have a special syntax.
    pub fn parse<R>() -> impl Parser<R, Output = Statement>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        crate::built_ins::close::parser::parse()
            .or(crate::built_ins::input::parser::parse())
            .or(crate::built_ins::line_input::parser::parse())
            .or(crate::built_ins::name::parser::parse())
            .or(crate::built_ins::open::parser::parse())
            .or(crate::built_ins::field::parser::parse())
            .or(crate::built_ins::lset::parser::parse())
            .or(crate::built_ins::get::parser::parse())
            .or(crate::built_ins::put::parser::parse())
            .or(crate::built_ins::view_print::parser::parse())
    }
}

pub mod linter {
    use crate::built_ins::BuiltInSub;
    use crate::common::QErrorNode;
    use crate::parser::ExpressionNodes;

    pub fn lint_sub_call(
        built_in_sub: &BuiltInSub,
        args: &ExpressionNodes,
    ) -> Result<(), QErrorNode> {
        match built_in_sub {
            BuiltInSub::Close => crate::built_ins::close::linter::lint(args),
            BuiltInSub::Environ => crate::built_ins::environ_sub::linter::lint(args),
            BuiltInSub::Field => crate::built_ins::field::linter::lint(args),
            BuiltInSub::Get => crate::built_ins::get::linter::lint(args),
            BuiltInSub::Input => crate::built_ins::input::linter::lint(args),
            BuiltInSub::Kill => crate::built_ins::kill::linter::lint(args),
            BuiltInSub::LineInput => crate::built_ins::line_input::linter::lint(args),
            BuiltInSub::LSet => crate::built_ins::lset::linter::lint(args),
            BuiltInSub::Name => crate::built_ins::name::linter::lint(args),
            BuiltInSub::Open => crate::built_ins::open::linter::lint(args),
            BuiltInSub::Put => crate::built_ins::put::linter::lint(args),
            BuiltInSub::ViewPrint => crate::built_ins::view_print::linter::lint(args),
        }
    }
}

pub mod close;
pub mod environ_sub;
pub mod field;
pub mod get;
pub mod input;
pub mod kill;
pub mod line_input;
pub mod lset;
pub mod name;
pub mod open;
pub mod put;
pub mod view_print;
