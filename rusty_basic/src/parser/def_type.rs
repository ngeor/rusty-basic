use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::{DefType, Keyword, LetterRange, TypeQualifier};
use rusty_common::*;

// DefType      ::= <DefKeyword><ws+><LetterRanges>
// DefKeyword   ::= DEFSNG|DEFDBL|DEFSTR|DEFINT|DEFLNG
// LetterRanges ::= <LetterRange> | <LetterRange><ws*>,<ws*><LetterRanges>
// LetterRange  ::= <Letter> | <Letter>-<Letter>
// Letter       ::= [a-zA-Z]

pub fn def_type_p() -> impl Parser<Output = DefType> {
    seq3(
        def_keyword_p(),
        whitespace().no_incomplete(),
        letter_ranges(),
        |l, _, r| DefType::new(l, r),
    )
}

fn def_keyword_p() -> impl Parser<Output = TypeQualifier> {
    keyword_map(&[
        (Keyword::DefInt, TypeQualifier::PercentInteger),
        (Keyword::DefLng, TypeQualifier::AmpersandLong),
        (Keyword::DefSng, TypeQualifier::BangSingle),
        (Keyword::DefDbl, TypeQualifier::HashDouble),
        (Keyword::DefStr, TypeQualifier::DollarString),
    ])
}

fn letter_ranges() -> impl Parser<Output = Vec<LetterRange>> + NonOptParser {
    csv_non_opt(letter_range(), "Expected: letter ranges")
}

fn letter_range() -> impl Parser<Output = LetterRange> {
    letter()
        .no_incomplete()
        .and_opt(minus_sign().and(letter()))
        .and_then(|(l, opt_r)| match opt_r {
            Some((_, r)) => {
                if l < r {
                    Ok(LetterRange::Range(l, r))
                } else {
                    Err(QError::syntax_error("Invalid letter range"))
                }
            }
            None => Ok(LetterRange::Single(l)),
        })
}

fn letter() -> impl Parser<Output = char> {
    any_token_of(TokenType::Identifier)
        .filter(|token| token.text.chars().count() == 1)
        .map(token_to_char)
        .map_incomplete_err(QError::expected("Expected: letter"))
}

fn token_to_char(token: Token) -> char {
    token.text.chars().next().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::test_utils::*;
    use crate::parser::types::{Statement, TopLevelToken};
    use crate::{assert_def_type, assert_parser_err};

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
        assert_parser_err!("DEFINT HELLO", QError::syntax_error("Expected: letter"));
        assert_parser_err!("DEFINT HELLO,Z", QError::syntax_error("Expected: letter"));
        assert_parser_err!("DEFINT A,HELLO", QError::syntax_error("Expected: letter"));
    }

    #[test]
    fn test_parse_def_int_reverse_range() {
        assert_parser_err!("DEFINT Z-A", QError::syntax_error("Invalid letter range"));
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
                TopLevelToken::DefType(DefType::new(
                    TypeQualifier::PercentInteger,
                    vec![LetterRange::Range('A', 'Z')]
                ))
                .at_rc(2, 9),
                TopLevelToken::Statement(Statement::Comment(" Improve performance".to_string()))
                    .at_rc(2, 20),
            ]
        );
    }
}
