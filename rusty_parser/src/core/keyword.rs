use rusty_common::cmp_str;
use rusty_pc::Token;

use crate::tokens::{TokenMatcher, TokenType};

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
    ($vis:vis enum $name:ident $all_names:ident $all_names_as_str:ident {
        $($(#[$($attrss:tt)*])*$member:ident),+$(,)?
    }) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        $vis enum $name {
            $($(#[$($attrss)*])*$member),+
        }

        /// Stores all keywords.
        const $all_names : &[$name] = &[
            $($name::$member),+
        ];

        /// Stores all keywords as [str].
        /// Can't store in uppercase, which would have been the preferred option,
        /// due to `const`.
        const $all_names_as_str : &[&str] = &[
            $(stringify!($member)),+
        ];

        impl $name {
            /// Returns an [str] reference for this keyword.
            /// Note that this will not be in full uppercase!
            pub fn as_str(&self) -> &str {
                let index = $all_names
                    .binary_search(self)
                    .expect("Should never happen");
                $all_names_as_str[index]
            }
        }

        impl TryFrom<&str> for $name {
            type Error = usize;

            /// Tries to find a Keyword named as the given [str], case insensitive.
            fn try_from(s: &str) -> Result<$name, usize> {
                $all_names_as_str.binary_search_by(|probe| cmp_str(probe, s))
                    .map(|index| $all_names[index])
            }
        }
    };
}

keyword_enum!(pub enum Keyword SORTED_KEYWORDS SORTED_KEYWORDS_STR {
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

impl std::fmt::Display for Keyword {
    /// Formats the keyword (in uppercase).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().to_uppercase().fmt(f)
    }
}

impl TokenMatcher for Keyword {
    /// Matches the given token if it's of Keyword token type and contains this keyword, case insensitive.
    fn matches_token(&self, token: &Token) -> bool {
        TokenType::Keyword.matches_token(token)
            && token.as_str().eq_ignore_ascii_case(self.as_str())
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
            let text = SORTED_KEYWORDS_STR[i];
            assert_eq!(Keyword::try_from(text).unwrap(), SORTED_KEYWORDS[i],);
            // can parse lowercase string to keyword
            let text = SORTED_KEYWORDS_STR[i].to_lowercase();
            assert_eq!(
                Keyword::try_from(text.as_str()).unwrap(),
                SORTED_KEYWORDS[i],
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
}
