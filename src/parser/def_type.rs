use crate::common::QError;
use crate::parser::base::and_pc::AndDemandTrait;
use crate::parser::base::and_then_pc::AndThenTrait;
use crate::parser::base::parsers::{
    ErrorProvider, FnMapTrait, NonOptParser, OrTrait, Parser, TokenPredicate,
};
use crate::parser::base::recognizers::is_letter;
use crate::parser::base::tokenizers::Token;
use crate::parser::specific::csv::csv_one_or_more;
use crate::parser::specific::keyword_choice::keyword_choice_p;
use crate::parser::specific::whitespace::WhitespaceTrait;
use crate::parser::specific::{item_p, OrSyntaxErrorTrait, TokenType};
use crate::parser::{DefType, Keyword, LetterRange, TypeQualifier};

// DefType      ::= <DefKeyword><ws+><LetterRanges>
// DefKeyword   ::= DEFSNG|DEFDBL|DEFSTR|DEFINT|DEFLNG
// LetterRanges ::= <LetterRange> | <LetterRange><ws*>,<ws*><LetterRanges>
// LetterRange  ::= <Letter> | <Letter>-<Letter>
// Letter       ::= [a-zA-Z]

pub fn def_type_p() -> impl Parser<Output = DefType> {
    def_keyword_p()
        .followed_by_req_ws()
        .and_demand(csv_one_or_more(letter_range_p()).or_syntax_error("Expected: letter ranges"))
        .fn_map(|(l, r)| DefType::new(l, r))
}

fn def_keyword_p() -> impl Parser<Output = TypeQualifier> {
    keyword_choice_p(&[
        Keyword::DefDbl,
        Keyword::DefInt,
        Keyword::DefLng,
        Keyword::DefSng,
        Keyword::DefStr,
    ])
    .fn_map(|(k, _)| match k {
        Keyword::DefInt => TypeQualifier::PercentInteger,
        Keyword::DefLng => TypeQualifier::AmpersandLong,
        Keyword::DefSng => TypeQualifier::BangSingle,
        Keyword::DefDbl => TypeQualifier::HashDouble,
        Keyword::DefStr => TypeQualifier::DollarString,
        _ => panic!("Should not have parsed keyword {} in def_keyword_p", k),
    })
}

fn letter_range_p() -> impl Parser<Output = LetterRange> {
    two_letter_range_p().or(single_letter_range_p())
}

fn single_letter_range_p() -> impl Parser<Output = LetterRange> {
    letter_opt().fn_map(LetterRange::Single)
}

fn two_letter_range_p() -> impl Parser<Output = LetterRange> {
    letter_opt()
        .and_demand(item_p('-'))
        .and_demand(letter())
        .and_then(|((l, _), r)| {
            if l < r {
                Ok(LetterRange::Range(l, r))
            } else {
                Err(QError::syntax_error("Invalid letter range"))
            }
        })
}

fn letter_opt() -> impl Parser<Output = char> {
    LetterToken
        .parser()
        .fn_map(|token| token.text.chars().next().unwrap())
}

fn letter() -> impl NonOptParser<Output = char> {
    LetterToken
        .parser()
        .fn_map(|token| token.text.chars().next().unwrap())
}

struct LetterToken;

impl TokenPredicate for LetterToken {
    fn test(&self, token: &Token) -> bool {
        token.kind == TokenType::Identifier as i32
            && token.text.len() == 1
            && is_letter(token.text.chars().next().unwrap())
    }
}

impl ErrorProvider for LetterToken {
    fn provide_error(&self) -> QError {
        QError::syntax_error("Expected letter")
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_parser_err;
    use crate::common::*;
    use crate::parser::test_utils::*;
    use crate::parser::types::{Statement, TopLevelToken};

    use super::*;

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
        assert_parser_err!("DEFINT HELLO", QError::syntax_error("No separator: E"));
        assert_parser_err!("DEFINT HELLO,Z", QError::syntax_error("No separator: E"));
        assert_parser_err!("DEFINT A,HELLO", QError::syntax_error("No separator: E"));
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
