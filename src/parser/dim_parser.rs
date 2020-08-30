use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::declared_name;
use crate::parser::pc::common::*;
use crate::parser::types::*;
use std::io::BufRead;

/// Parses DIM statement
pub fn dim<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Statement, QError>)> {
    map(
        crate::parser::pc::ws::seq2(
            try_read_keyword(Keyword::Dim),
            demand(
                declared_name::declared_name_node(),
                QError::syntax_error_fn("Expected: name after DIM"),
            ),
            QError::syntax_error_fn("Expected: whitespace after DIM"),
        ),
        |(_, r)| Statement::Dim(r),
    )
}

#[cfg(test)]
mod tests {
    use crate::common::*;

    use crate::parser::test_utils::*;
    use crate::parser::types::*;

    macro_rules! assert_parse_dim_extended_built_in {
        ($keyword: literal, $qualifier: ident) => {
            let input = format!("DIM X AS {}", $keyword);
            let p = parse(input);
            assert_eq!(
                p,
                vec![TopLevelToken::Statement(Statement::Dim(
                    DeclaredName::new(
                        "X".into(),
                        TypeDefinition::ExtendedBuiltIn(TypeQualifier::$qualifier)
                    )
                    .at_rc(1, 5)
                ))
                .at_rc(1, 1)]
            );
        };
    }

    #[test]
    fn test_parse_dim_extended_single() {
        assert_parse_dim_extended_built_in!("SINGLE", BangSingle);
    }

    #[test]
    fn test_parse_dim_extended_double() {
        assert_parse_dim_extended_built_in!("DOUBLE", HashDouble);
    }

    #[test]
    fn test_parse_dim_extended_string() {
        assert_parse_dim_extended_built_in!("STRING", DollarString);
    }

    #[test]
    fn test_parse_dim_extended_integer() {
        assert_parse_dim_extended_built_in!("INTEGER", PercentInteger);
    }

    #[test]
    fn test_parse_dim_extended_long() {
        assert_parse_dim_extended_built_in!("LONG", AmpersandLong);
    }

    #[test]
    fn test_parse_dim_extended_wrong_keyword() {
        let input = "DIM X AS AS";
        assert_eq!(
            parse_err(input),
            QError::SyntaxError(
                "Expected: INTEGER or LONG or SINGLE or DOUBLE or STRING or identifier".to_string()
            )
        );
    }

    #[test]
    fn test_parse_dim_extended_with_qualified_name() {
        let input = "DIM A$ AS STRING";
        assert_eq!(
            parse_err(input),
            QError::SyntaxError("Identifier cannot end with %, &, !, #, or $".to_string())
        );
    }

    macro_rules! assert_parse_dim_compact {
        ($keyword: literal, $qualifier: ident) => {
            let input = format!("DIM X{}", $keyword);
            let p = parse(input);
            assert_eq!(
                p,
                vec![TopLevelToken::Statement(Statement::Dim(
                    DeclaredName::compact("X", TypeQualifier::$qualifier).at_rc(1, 5)
                ))
                .at_rc(1, 1)]
            );
        };
    }

    #[test]
    fn test_parse_dim_compact_single() {
        assert_parse_dim_compact!("!", BangSingle);
    }

    #[test]
    fn test_parse_dim_compact_double() {
        assert_parse_dim_compact!("#", HashDouble);
    }

    #[test]
    fn test_parse_dim_compact_string() {
        assert_parse_dim_compact!("$", DollarString);
    }

    #[test]
    fn test_parse_dim_compact_integer() {
        assert_parse_dim_compact!("%", PercentInteger);
    }

    #[test]
    fn test_parse_dim_compact_long() {
        assert_parse_dim_compact!("&", AmpersandLong);
    }

    #[test]
    fn test_parse_dim_compact_bare() {
        let p = parse("DIM X");
        assert_eq!(
            p,
            vec![
                TopLevelToken::Statement(Statement::Dim(DeclaredName::bare("X").at_rc(1, 5)))
                    .at_rc(1, 1)
            ]
        );
    }
}
