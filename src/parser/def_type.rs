use crate::common::{HasLocation, QError};
use crate::parser::pc::binary::BinaryParser;
use crate::parser::pc::text::whitespace_p;
use crate::parser::pc::unary_fn::UnaryFnParser;
use crate::parser::pc::{if_p, item_p, Parser};
use crate::parser::pc::{is_letter, Reader};
use crate::parser::pc_specific::{keyword_p, PcSpecific};
use crate::parser::{DefType, Keyword, LetterRange, TypeQualifier};

// DefType      ::= <DefKeyword><ws+><LetterRanges>
// DefKeyword   ::= DEFSNG|DEFDBL|DEFSTR|DEFINT|DEFLNG
// LetterRanges ::= <LetterRange> | <LetterRange><ws*>,<ws*><LetterRanges>
// LetterRange  ::= <Letter> | <Letter>-<Letter>
// Letter       ::= [a-zA-Z]

pub fn def_type_p<R>() -> impl Parser<R, Output = DefType>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    def_keyword_p()
        .and_demand(whitespace_p().or_syntax_error("Expected: whitespace after DEF-type"))
        .and_demand(
            letter_range_p()
                .csv()
                .or_syntax_error("Expected: letter ranges"),
        )
        .map(|((l, _), r)| DefType::new(l, r))
}

fn def_keyword_p<R>() -> impl Parser<R, Output = TypeQualifier>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_p(Keyword::DefDbl)
        .or(keyword_p(Keyword::DefInt))
        .or(keyword_p(Keyword::DefLng))
        .or(keyword_p(Keyword::DefSng))
        .or(keyword_p(Keyword::DefStr))
        .map(|(k, _)| match k {
            Keyword::DefInt => TypeQualifier::PercentInteger,
            Keyword::DefLng => TypeQualifier::AmpersandLong,
            Keyword::DefSng => TypeQualifier::BangSingle,
            Keyword::DefDbl => TypeQualifier::HashDouble,
            Keyword::DefStr => TypeQualifier::DollarString,
            _ => panic!("Should not have parsed keyword {} in def_keyword_p", k),
        })
}

fn letter_range_p<R>() -> impl Parser<R, Output = LetterRange>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    two_letter_range_p().or(single_letter_range_p())
}

fn single_letter_range_p<R>() -> impl Parser<R, Output = LetterRange>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    if_p(is_letter).map(|l| LetterRange::Single(l))
}

fn two_letter_range_p<R>() -> impl Parser<R, Output = LetterRange>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    if_p(is_letter)
        .and(item_p('-'))
        .and_demand(if_p(is_letter).or_syntax_error("Expected: letter after dash"))
        .and_then(|((l, _), r)| {
            if l < r {
                Ok(LetterRange::Range(l, r))
            } else {
                Err(QError::syntax_error("Invalid letter range"))
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
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
                _ => panic!(format!("{:?}", $input)),
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
        assert_eq!(
            parse_err("DEFINT HELLO"),
            QError::syntax_error("No separator: E")
        );
        assert_eq!(
            parse_err("DEFINT HELLO,Z"),
            QError::syntax_error("No separator: E")
        );
        assert_eq!(
            parse_err("DEFINT A,HELLO"),
            QError::syntax_error("No separator: E")
        );
    }

    #[test]
    fn test_parse_def_int_reverse_range() {
        assert_eq!(
            parse_err("DEFINT Z-A"),
            QError::syntax_error("Invalid letter range")
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
