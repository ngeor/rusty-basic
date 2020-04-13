use crate::common::CmpIgnoreAsciiCase;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Keyword {
    /// DECLARE
    Declare,
    /// DEFDBL
    DefDbl,
    /// DEFINT
    DefInt,
    /// DEFLNG
    DefLng,
    /// DEFSNG
    DefSng,
    /// DEFSTR
    DefStr,
    /// ELSE
    Else,
    /// ELSEIF
    ElseIf,
    /// END
    End,
    /// FOR
    For,
    /// FUNCTION
    Function,
    /// IF
    If,
    /// NEXT
    Next,
    /// STEP
    Step,
    /// THEN
    Then,
    /// TO
    To,
}

const STR_DECLARE: &str = "DECLARE";
const STR_DEFDBL: &str = "DEFDBL";
const STR_DEFINT: &str = "DEFINT";
const STR_DEFLNG: &str = "DEFLNG";
const STR_DEFSNG: &str = "DEFSNG";
const STR_DEFSTR: &str = "DEFSTR";
const STR_ELSE: &str = "ELSE";
const STR_ELSEIF: &str = "ELSEIF";
const STR_END: &str = "END";
const STR_FOR: &str = "FOR";
const STR_FUNCTION: &str = "FUNCTION";
const STR_IF: &str = "IF";
const STR_NEXT: &str = "NEXT";
const STR_STEP: &str = "STEP";
const STR_THEN: &str = "THEN";
const STR_TO: &str = "TO";

const SORTED_KEYWORDS_STR: [&str; 16] = [
    STR_DECLARE,
    STR_DEFDBL,
    STR_DEFINT,
    STR_DEFLNG,
    STR_DEFSNG,
    STR_DEFSTR,
    STR_ELSE,
    STR_ELSEIF,
    STR_END,
    STR_FOR,
    STR_FUNCTION,
    STR_IF,
    STR_NEXT,
    STR_STEP,
    STR_THEN,
    STR_TO,
];

const SORTED_KEYWORDS: [Keyword; 16] = [
    Keyword::Declare,
    Keyword::DefDbl,
    Keyword::DefInt,
    Keyword::DefLng,
    Keyword::DefSng,
    Keyword::DefStr,
    Keyword::Else,
    Keyword::ElseIf,
    Keyword::End,
    Keyword::For,
    Keyword::Function,
    Keyword::If,
    Keyword::Next,
    Keyword::Step,
    Keyword::Then,
    Keyword::To,
];

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Declare => STR_DECLARE.fmt(f),
            Self::DefDbl => STR_DEFDBL.fmt(f),
            Self::DefInt => STR_DEFINT.fmt(f),
            Self::DefLng => STR_DEFLNG.fmt(f),
            Self::DefSng => STR_DEFSNG.fmt(f),
            Self::DefStr => STR_DEFSTR.fmt(f),
            Self::Else => STR_ELSE.fmt(f),
            Self::ElseIf => STR_ELSEIF.fmt(f),
            Self::End => STR_END.fmt(f),
            Self::For => STR_FOR.fmt(f),
            Self::Function => STR_FUNCTION.fmt(f),
            Self::If => STR_IF.fmt(f),
            Self::Next => STR_NEXT.fmt(f),
            Self::Step => STR_STEP.fmt(f),
            Self::Then => STR_THEN.fmt(f),
            Self::To => STR_TO.fmt(f),
        }
    }
}

impl FromStr for Keyword {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        SORTED_KEYWORDS_STR
            .binary_search_by(|p| CmpIgnoreAsciiCase::compare_ignore_ascii_case(*p, s))
            .map(|idx| SORTED_KEYWORDS[idx])
            .map_err(|_| format!("Not a keyword: {}", s))
    }
}

#[cfg(test)]
mod tests {
    use super::Keyword;

    #[test]
    fn test_to_string() {
        assert_eq!(Keyword::Declare.to_string(), "DECLARE");
        assert_eq!(Keyword::DefDbl.to_string(), "DEFDBL");
        assert_eq!(Keyword::DefInt.to_string(), "DEFINT");
        assert_eq!(Keyword::DefLng.to_string(), "DEFLNG");
        assert_eq!(Keyword::DefSng.to_string(), "DEFSNG");
        assert_eq!(Keyword::DefStr.to_string(), "DEFSTR");
        assert_eq!(Keyword::Else.to_string(), "ELSE");
        assert_eq!(Keyword::ElseIf.to_string(), "ELSEIF");
        assert_eq!(Keyword::End.to_string(), "END");
        assert_eq!(Keyword::For.to_string(), "FOR");
        assert_eq!(Keyword::Function.to_string(), "FUNCTION");
        assert_eq!(Keyword::If.to_string(), "IF");
        assert_eq!(Keyword::Next.to_string(), "NEXT");
        assert_eq!(Keyword::Step.to_string(), "STEP");
        assert_eq!(Keyword::Then.to_string(), "THEN");
        assert_eq!(Keyword::To.to_string(), "TO");
    }

    #[test]
    fn test_from_string() {
        assert_eq!(Keyword::Declare, "DECLARE".parse().unwrap());
        assert_eq!(Keyword::DefDbl, "DEFDBL".parse().unwrap());
        assert_eq!(Keyword::DefInt, "DEFINT".parse().unwrap());
        assert_eq!(Keyword::DefLng, "DEFLNG".parse().unwrap());
        assert_eq!(Keyword::DefSng, "DEFSNG".parse().unwrap());
        assert_eq!(Keyword::DefStr, "DEFSTR".parse().unwrap());
        assert_eq!(Keyword::Else, "ELSE".parse().unwrap());
        assert_eq!(Keyword::ElseIf, "ELSEIF".parse().unwrap());
        assert_eq!(Keyword::End, "END".parse().unwrap());
        assert_eq!(Keyword::For, "FOR".parse().unwrap());
        assert_eq!(Keyword::Function, "FUNCTION".parse().unwrap());
        assert_eq!(Keyword::If, "IF".parse().unwrap());
        assert_eq!(Keyword::Next, "NEXT".parse().unwrap());
        assert_eq!(Keyword::Step, "STEP".parse().unwrap());
        assert_eq!(Keyword::Then, "THEN".parse().unwrap());
        assert_eq!(Keyword::To, "TO".parse().unwrap());
    }

    #[test]
    fn test_from_string_lowercase() {
        assert_eq!(Keyword::Declare, "declare".parse().unwrap());
        assert_eq!(Keyword::DefDbl, "defdbl".parse().unwrap());
        assert_eq!(Keyword::DefInt, "defint".parse().unwrap());
        assert_eq!(Keyword::DefLng, "deflng".parse().unwrap());
        assert_eq!(Keyword::DefSng, "defsng".parse().unwrap());
        assert_eq!(Keyword::DefStr, "defstr".parse().unwrap());
        assert_eq!(Keyword::Else, "else".parse().unwrap());
        assert_eq!(Keyword::ElseIf, "elseif".parse().unwrap());
        assert_eq!(Keyword::End, "end".parse().unwrap());
        assert_eq!(Keyword::For, "for".parse().unwrap());
        assert_eq!(Keyword::Function, "function".parse().unwrap());
        assert_eq!(Keyword::If, "if".parse().unwrap());
        assert_eq!(Keyword::Next, "next".parse().unwrap());
        assert_eq!(Keyword::Step, "step".parse().unwrap());
        assert_eq!(Keyword::Then, "then".parse().unwrap());
        assert_eq!(Keyword::To, "to".parse().unwrap());
    }

    #[test]
    fn test_from_string_not_a_keyword() {
        assert_eq!("Not a keyword: foo", "foo".parse::<Keyword>().unwrap_err());
    }
}
