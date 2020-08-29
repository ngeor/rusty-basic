use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::pc::common::*;
use crate::parser::pc::copy::*;
use crate::parser::pc::traits::*;
use crate::parser::types::*;
use std::io::BufRead;

// DefType      ::= <DefKeyword><ws+><LetterRanges>
// DefKeyword   ::= DEFSNG|DEFDBL|DEFSTR|DEFINT|DEFLNG
// LetterRanges ::= <LetterRange> | <LetterRange><ws*>,<ws*><LetterRanges>
// LetterRange  ::= <Letter> | <Letter>-<Letter>
// Letter       ::= [a-zA-Z]

pub fn def_type<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<DefType, QError>)> {
    map(
        seq3(
            def_keyword(),
            demand(
                crate::parser::pc::ws::one_or_more(),
                QError::syntax_error_fn("Expected whitespace"),
            ),
            demand(
                letter_ranges(),
                QError::syntax_error_fn("Expected letter ranges"),
            ),
        ),
        |(l, _, r)| DefType::new(l, r),
    )
}

fn def_keyword<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<TypeQualifier, QError>)> {
    map_fully_ok(read_any_keyword(), |reader: EolReader<T>, (k, s)| match k {
        Keyword::DefDbl => (reader, Ok(TypeQualifier::HashDouble)),
        Keyword::DefInt => (reader, Ok(TypeQualifier::PercentInteger)),
        Keyword::DefLng => (reader, Ok(TypeQualifier::AmpersandLong)),
        Keyword::DefSng => (reader, Ok(TypeQualifier::BangSingle)),
        Keyword::DefStr => (reader, Ok(TypeQualifier::DollarString)),
        _ => (reader.undo(s), Err(QError::not_found_err())),
    })
}

fn letter_ranges<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Vec<LetterRange>, QError>)> {
    map_default_to_not_found(csv_zero_or_more(letter_range()))
}

fn letter_range<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<LetterRange, QError>)> {
    or(
        two_letter_range(), // needs to be first because the second will match too
        single_letter_range(),
    )
}

fn single_letter_range<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<LetterRange, QError>)> {
    map(read_any_letter(), |l| LetterRange::Single(l))
}

fn two_letter_range<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<LetterRange, QError>)> {
    and_then(
        and(
            read_any_letter(),
            seq2(
                try_read('-'),
                demand(
                    read_any_letter(),
                    QError::syntax_error_fn("Expected letter after dash"),
                ),
            ),
        ),
        |(l, (_, r))| {
            if l < r {
                Ok(LetterRange::Range(l, r))
            } else {
                Err(QError::SyntaxError("Invalid letter range".to_string()))
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use super::*;
    use crate::parser::{HasQualifier, Statement};

    /// Asserts that the given input program contains a def type top level token.
    macro_rules! assert_def_type {
        ($input:expr, $expected_qualifier:expr, $expected_ranges:expr) => {
            match parse($input).demand_single().as_ref() {
                TopLevelToken::DefType(x) => {
                    assert_eq!(x.qualifier(), $expected_qualifier);
                    assert_eq!(x.ranges(), &$expected_ranges);
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
            QError::SyntaxError("No separator: E".to_string(),)
        );
        assert_eq!(
            parse_err("DEFINT HELLO,Z"),
            QError::SyntaxError("No separator: E".to_string(),)
        );
        assert_eq!(
            parse_err("DEFINT A,HELLO"),
            QError::SyntaxError("No separator: E".to_string(),)
        );
    }

    #[test]
    fn test_parse_def_int_reverse_range() {
        assert_eq!(
            parse_err("DEFINT Z-A"),
            QError::SyntaxError("Invalid letter range".to_string(),)
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
