use crate::common::QError;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::{DefType, Keyword, LetterRange, TypeQualifier};

// DefType      ::= <DefKeyword><ws+><LetterRanges>
// DefKeyword   ::= DEFSNG|DEFDBL|DEFSTR|DEFINT|DEFLNG
// LetterRanges ::= <LetterRange> | <LetterRange><ws*>,<ws*><LetterRanges>
// LetterRange  ::= <Letter> | <Letter>-<Letter>
// Letter       ::= [a-zA-Z]

pub fn def_type_p() -> impl Parser<Output = DefType> {
    seq3(def_keyword_p(), whitespace(), letter_ranges(), |l, _, r| {
        DefType::new(l, r)
    })
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

fn letter_ranges() -> impl NonOptParser<Output = Vec<LetterRange>> {
    letter_range().csv_demand()
}

fn letter_range() -> impl NonOptParser<Output = LetterRange> {
    letter()
        .and_opt(item_p('-').and(letter()))
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

fn letter() -> impl Parser<Output = char> + NonOptParser<Output = char> {
    LetterToken.parser().map(token_to_char)
}

struct LetterToken;

impl TokenPredicate for LetterToken {
    fn test(&self, token: &Token) -> bool {
        token.kind == TokenType::Identifier as i32
            && token.text.len() == 1
            && is_letter(token_ref_to_char(token))
    }
}

impl ErrorProvider for LetterToken {
    fn provide_error(&self) -> QError {
        QError::syntax_error("Expected: letter")
    }
}

fn token_ref_to_char(token: &Token) -> char {
    token.text.chars().next().unwrap()
}

fn token_to_char(token: Token) -> char {
    token.text.chars().next().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_parser_err;
    use crate::common::*;
    use crate::parser::test_utils::*;
    use crate::parser::types::{Statement, TopLevelToken};

    /// Asserts that the given input program contains a def type top level token.
    macro_rules! assert_def_type {
        ($input:expr, $expected_qualifier:expr, $expected_ranges:expr) => {
            match parse($input).demand_single().strip_location() {
                TopLevelToken::DefType(def_type) => {
                    let def_type_qualifier: &crate::parser::TypeQualifier = def_type.as_ref();
                    assert_eq!(*def_type_qualifier, $expected_qualifier);
                    assert_eq!(def_type.ranges(), &$expected_ranges);
                }
                _ => panic!("{:?}", $input),
            }
        };
    }

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
