use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::declared_name;
use crate::parser::pc::common::*;
use crate::parser::pc::map::map;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::types::*;
use std::io::BufRead;

/// Parses DIM statement
pub fn dim<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
    map(
        crate::parser::pc::ws::seq2(
            keyword(Keyword::Dim),
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
        ($name: literal, $keyword: literal, $qualifier: ident) => {
            let input = format!("DIM {} AS {}", $name, $keyword);
            let p = parse(input);
            assert_eq!(
                p,
                vec![TopLevelToken::Statement(Statement::Dim(
                    DeclaredName::new(
                        $name.into(),
                        TypeDefinition::ExtendedBuiltIn(TypeQualifier::$qualifier)
                    )
                    .at_rc(1, 5)
                ))
                .at_rc(1, 1)]
            );
        };
    }

    #[test]
    fn test_parse_dim_extended_built_in() {
        assert_parse_dim_extended_built_in!("A", "SINGLE", BangSingle);
        assert_parse_dim_extended_built_in!("A.", "SINGLE", BangSingle);
        assert_parse_dim_extended_built_in!("A.B", "SINGLE", BangSingle);
        assert_parse_dim_extended_built_in!("AB", "DOUBLE", HashDouble);
        assert_parse_dim_extended_built_in!("S", "STRING", DollarString);
        assert_parse_dim_extended_built_in!("I", "INTEGER", PercentInteger);
        assert_parse_dim_extended_built_in!("L1", "LONG", AmpersandLong);
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
        ($name: literal) => {
            let input = format!("DIM {}", $name);
            let p = parse(input);
            assert_eq!(
                p,
                vec![TopLevelToken::Statement(Statement::Dim(
                    DeclaredName::bare($name).at_rc(1, 5)
                ))
                .at_rc(1, 1)]
            );
        };

        ($name: literal, $keyword: literal, $qualifier: ident) => {
            let input = format!("DIM {}{}", $name, $keyword);
            let p = parse(input);
            assert_eq!(
                p,
                vec![TopLevelToken::Statement(Statement::Dim(
                    DeclaredName::compact($name, TypeQualifier::$qualifier).at_rc(1, 5)
                ))
                .at_rc(1, 1)]
            );
        };
    }

    #[test]
    fn test_parse_dim_compact() {
        assert_parse_dim_compact!("A");
        assert_parse_dim_compact!("A.");
        assert_parse_dim_compact!("A.B");
        assert_parse_dim_compact!("A", "!", BangSingle);
        assert_parse_dim_compact!("A.", "!", BangSingle);
        assert_parse_dim_compact!("A.B", "!", BangSingle);
        assert_parse_dim_compact!("BC", "#", HashDouble);
        assert_parse_dim_compact!("X", "$", DollarString);
        assert_parse_dim_compact!("Z", "%", PercentInteger);
        assert_parse_dim_compact!("L1", "&", AmpersandLong);
    }
}
