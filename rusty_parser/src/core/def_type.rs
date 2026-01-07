use rusty_pc::*;

use crate::error::ParseError;
use crate::input::RcStringView;
use crate::pc_specific::*;
use crate::tokens::{TokenType, any_token_of, minus_sign, whitespace};
use crate::{Keyword, LetterRange, TypeQualifier};

/// Represents a definition of default type, such as DEFINT A-Z.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefType {
    qualifier: TypeQualifier,
    ranges: Vec<LetterRange>,
}

impl DefType {
    pub fn new(qualifier: TypeQualifier, ranges: Vec<LetterRange>) -> Self {
        Self { qualifier, ranges }
    }

    pub fn ranges(&self) -> &Vec<LetterRange> {
        &self.ranges
    }

    pub fn qualifier(&self) -> TypeQualifier {
        self.qualifier
    }
}

// DefType      ::= <DefKeyword><ws+><LetterRanges>
// DefKeyword   ::= DEFSNG|DEFDBL|DEFSTR|DEFINT|DEFLNG
// LetterRanges ::= <LetterRange> | <LetterRange><ws*>,<ws*><LetterRanges>
// LetterRange  ::= <Letter> | <Letter>-<Letter>
// Letter       ::= [a-zA-Z]

pub fn def_type_p() -> impl Parser<RcStringView, Output = DefType, Error = ParseError> {
    seq3(def_keyword_p(), whitespace(), letter_ranges(), |l, _, r| {
        DefType::new(l, r)
    })
}

fn def_keyword_p() -> impl Parser<RcStringView, Output = TypeQualifier, Error = ParseError> {
    keyword_map(&[
        (Keyword::DefInt, TypeQualifier::PercentInteger),
        (Keyword::DefLng, TypeQualifier::AmpersandLong),
        (Keyword::DefSng, TypeQualifier::BangSingle),
        (Keyword::DefDbl, TypeQualifier::HashDouble),
        (Keyword::DefStr, TypeQualifier::DollarString),
    ])
}

fn letter_ranges() -> impl Parser<RcStringView, Output = Vec<LetterRange>, Error = ParseError> {
    csv_non_opt(letter_range(), "Expected: letter ranges")
}

fn letter_range() -> impl Parser<RcStringView, Output = LetterRange, Error = ParseError> {
    letter()
        .no_incomplete()
        .and_opt_tuple(minus_sign().and_tuple(letter()))
        .flat_map(|input, (l, opt_r)| match opt_r {
            Some((_, r)) => {
                if l < r {
                    Ok((input, LetterRange::Range(l, r)))
                } else {
                    Err((
                        true,
                        input,
                        ParseError::syntax_error("Invalid letter range"),
                    ))
                }
            }
            None => Ok((input, LetterRange::Single(l))),
        })
}

fn letter() -> impl Parser<RcStringView, Output = char, Error = ParseError> {
    any_token_of!(TokenType::Identifier)
        .filter_or_err(
            |token| token.as_str().chars().count() == 1,
            ParseError::syntax_error("Expected: letter"),
        )
        .map(token_to_char)
}

fn token_to_char(token: Token) -> char {
    token.as_str().chars().next().unwrap()
}

#[cfg(test)]
mod tests {
    use rusty_common::AtPos;

    use crate::error::ParseError;
    use crate::test_utils::*;
    use crate::{assert_def_type, assert_parser_err, *};

    #[test]
    fn test_parse_def_int_a_z() {
        assert_def_type!(
            "DEFINT A-Z",
            TypeQualifier::PercentInteger,
            vec![LetterRange::Range('A', 'Z')]
        );
    }

    #[test]
    fn test_parse_def_int_a() {
        assert_def_type!(
            "DEFINT A",
            TypeQualifier::PercentInteger,
            vec![LetterRange::Single('A')]
        );
    }

    #[test]
    fn test_parse_def_str_a_b_c_mixed_whitespace() {
        assert_def_type!(
            "DEFSTR A, B,C  ",
            TypeQualifier::DollarString,
            vec![
                LetterRange::Single('A'),
                LetterRange::Single('B'),
                LetterRange::Single('C')
            ]
        );
    }

    #[test]
    fn test_parse_def_lng_a_i_k_w_z() {
        assert_def_type!(
            "DEFLNG A-I, K-W, Z",
            TypeQualifier::AmpersandLong,
            vec![
                LetterRange::Range('A', 'I'),
                LetterRange::Range('K', 'W'),
                LetterRange::Single('Z')
            ]
        );
    }

    #[test]
    fn test_parse_def_int_word_instead_of_letter() {
        assert_parser_err!("DEFINT HELLO", ParseError::syntax_error("Expected: letter"));
        assert_parser_err!(
            "DEFINT HELLO,Z",
            ParseError::syntax_error("Expected: letter")
        );
        assert_parser_err!(
            "DEFINT A,HELLO",
            ParseError::syntax_error("Expected: letter")
        );
    }

    #[test]
    fn test_parse_def_int_reverse_range() {
        assert_parser_err!(
            "DEFINT Z-A",
            ParseError::syntax_error("Invalid letter range")
        );
    }

    #[test]
    fn test_inline_comment() {
        let input = r#"
        DEFINT A-Z ' Improve performance
        "#;
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                GlobalStatement::DefType(DefType::new(
                    TypeQualifier::PercentInteger,
                    vec![LetterRange::Range('A', 'Z')]
                ))
                .at_rc(2, 9),
                GlobalStatement::Statement(Statement::Comment(" Improve performance".to_string()))
                    .at_rc(2, 20),
            ]
        );
    }
}
