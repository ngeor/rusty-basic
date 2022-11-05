use crate::pc::Token;
use crate::pc_specific::TokenType;
use rusty_common::{CaseInsensitiveStr, QError};
use std::str::FromStr;

// From the internets:
// Doc comments are secretly just attributes,
// so if your macro can match attributes, it can match doc comments.
// Then, you just need to emit the attributes with the item(s)
// you're generating.
//
// In particular, it's the `$(#[$($attrss:tt)*])*` pattern to match attributes,
// and the `$(#[$($attrss)*])*` expression to emit them that you want.

#[macro_export]
macro_rules! keyword_enum {
    ($vis:vis enum $name:ident $all_names:ident $all_names_as_str:ident $all_names_as_case_insensitive_str:ident {
        $($(#[$($attrss:tt)*])*$member:ident),+$(,)?
    }) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        $vis enum $name {
            $($(#[$($attrss)*])*$member),+
        }

        const $all_names : &[$name] = &[
            $($name::$member),+
        ];

        pub const $all_names_as_str : &[&str] = &[
            $(stringify!($member)),+
        ];

        const $all_names_as_case_insensitive_str : &[&CaseInsensitiveStr] = &[
            $( CaseInsensitiveStr::new( stringify!($member) ) ),+
        ];

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.as_ref().to_uppercase().fmt(f)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                let idx = $all_names
                    .binary_search(self)
                    .expect("Should never happen");
                $all_names_as_str[idx]
            }
        }

        impl TryFrom<&CaseInsensitiveStr> for $name {
            type Error = usize;

            fn try_from(s: &CaseInsensitiveStr) -> Result<$name, usize> {
                $all_names_as_case_insensitive_str.binary_search(&s)
                    .map(|idx| $all_names[idx])
            }
        }

    };
}

keyword_enum!(pub enum Keyword SORTED_KEYWORDS SORTED_KEYWORDS_STR SORTED_KEYWORDS_CI_STR {
    /// ACCESS
    Access,
    /// AND
    And,
    /// APPEND
    Append,
    /// AS
    As,
    /// CASE
    Case,
    /// CLOSE
    Close,
    /// COLOR
    Color,
    /// CONST
    Const,
    /// DATA
    Data,
    /// DECLARE
    Declare,
    /// DEF
    Def,
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
    /// DIM
    Dim,
    // DO
    Do,
    /// DOUBLE
    Double,
    /// ELSE
    Else,
    /// ELSEIF
    ElseIf,
    /// END
    End,
    /// ERROR
    Error,
    /// EXIT
    Exit,
    /// FIELD
    Field,
    /// FOR
    For,
    /// FUNCTION
    Function,
    /// GET
    Get,
    /// GOSUB
    GoSub,
    /// GOTO
    GoTo,
    /// IF
    If,
    /// INPUT
    Input,
    /// INTEGER
    Integer,
    /// IS
    Is,
    /// LEN
    Len,
    /// LINE
    Line,
    /// LOCATE
    Locate,
    /// LONG
    Long,
    /// LOOP
    Loop,
    /// LPRINT
    LPrint,
    /// LSET
    LSet,
    /// MOD
    Mod,
    /// NAME
    Name,
    /// NEXT
    Next,
    /// NOT
    Not,
    /// ON
    On,
    /// OPEN
    Open,
    /// OR
    Or,
    /// OUTPUT
    Output,
    /// PRINT
    Print,
    /// PUT
    Put,
    /// RANDOM
    Random,
    /// READ
    Read,
    /// REDIM
    Redim,
    /// RESUME
    Resume,
    /// RETURN
    Return,
    /// SEG
    Seg,
    /// SELECT
    Select,
    /// SHARED
    Shared,
    /// SINGLE
    Single,
    /// STATIC
    Static,
    /// STEP
    Step,
    /// STRING
    String,
    /// SUB
    Sub,
    /// SYSTEM
    System,
    /// THEN
    Then,
    /// TO
    To,
    /// TYPE
    Type,
    /// UNTIL
    Until,
    /// USING
    Using,
    /// VIEW
    View,
    /// WEND
    Wend,
    /// WHILE
    While,
    /// WIDTH
    Width,
});

impl PartialEq<Token> for Keyword {
    fn eq(&self, other: &Token) -> bool {
        TokenType::Keyword.matches(other) && other.text.eq_ignore_ascii_case(self.as_ref())
    }
}

impl FromStr for Keyword {
    type Err = QError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let temp = CaseInsensitiveStr::new(s);
        Self::try_from(temp).map_err(|_| QError::InternalError(format!("Not a keyword: {}", s)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyword_sanity_checks() {
        // equal size of the two arrays
        assert_eq!(SORTED_KEYWORDS.len(), SORTED_KEYWORDS_STR.len());
        for i in 0..SORTED_KEYWORDS.len() {
            // display is uppercase
            assert_eq!(
                SORTED_KEYWORDS[i].to_string(),
                SORTED_KEYWORDS_STR[i].to_uppercase()
            );
            // can parse string to keyword
            assert_eq!(
                SORTED_KEYWORDS_STR[i].parse::<Keyword>().unwrap(),
                SORTED_KEYWORDS[i],
            );
            // can parse lowercase string to keyword
            assert_eq!(
                SORTED_KEYWORDS_STR[i]
                    .to_lowercase()
                    .parse::<Keyword>()
                    .unwrap(),
                SORTED_KEYWORDS[i]
            );
        }
        // sort order is correct
        for i in 1..SORTED_KEYWORDS.len() {
            assert!(
                SORTED_KEYWORDS_STR[i].to_uppercase() > SORTED_KEYWORDS_STR[i - 1].to_uppercase(),
                "{} should be after {}",
                SORTED_KEYWORDS_STR[i],
                SORTED_KEYWORDS_STR[i - 1]
            );
        }
    }

    #[test]
    fn test_from_string_not_a_keyword() {
        assert_eq!(
            QError::InternalError("Not a keyword: foo".to_string()),
            "foo".parse::<Keyword>().unwrap_err()
        );
    }
}
