// parses DIM statement

use crate::common::*;
use crate::lexer::*;
use crate::parser::buf_lexer::*;
use crate::parser::declared_name;
use crate::parser::error::*;
use crate::parser::types::*;
use std::io::BufRead;

pub fn try_read<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Option<StatementNode>, ParserError> {
    // try to read DIM, if it succeeds demand it, else return None
    if !lexer.peek()?.as_ref().is_keyword(Keyword::Dim) {
        return Ok(None);
    }
    // demand DIM
    let pos = read_keyword(lexer, Keyword::Dim)?;
    // demand whitespace
    read_whitespace(lexer)?;
    // demand variable name
    let declared_name_node = demand(lexer, declared_name::try_read, "Expected variable name")?;
    Ok(Some(Statement::Dim(declared_name_node).at(pos)))
}

#[cfg(test)]
mod tests {
    use crate::common::*;
    use crate::parser::error::*;
    use crate::parser::test_utils::*;
    use crate::parser::types::*;

    macro_rules! assert_parse_dim_extended_built_in {
        ($keyword: literal, $qualifier: ident) => {
            let input = format!("DIM X AS {}", $keyword);
            let p = parse(input.as_ref());
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
            ParserError::SyntaxError(
                "Expected: INTEGER or LONG or SINGLE or DOUBLE or STRING or identifier".to_string(),
                Location::new(1, 10)
            )
        );
    }

    #[test]
    fn test_parse_dim_extended_with_qualified_name() {
        let input = "DIM A$ AS STRING";
        assert_eq!(
            parse_err(input),
            ParserError::SyntaxError(
                "Identifier cannot end with %, &, !, #, or $".to_string(),
                Location::new(1, 5)
            )
        );
    }

    macro_rules! assert_parse_dim_compact {
        ($keyword: literal, $qualifier: ident) => {
            let input = format!("DIM X{}", $keyword);
            let p = parse(input.as_ref());
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
