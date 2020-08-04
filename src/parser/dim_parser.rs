// parses DIM statement

use crate::common::*;
use crate::lexer::*;
use crate::parser::buf_lexer::*;
use crate::parser::error::*;
use crate::parser::name;
use crate::parser::types::*;
use std::io::BufRead;

pub fn try_read<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Option<StatementNode>, ParserError> {
    // try to read DIM, if it succeeds demand it, else return None
    if !lexer.peek()?.is_keyword(Keyword::Dim) {
        return Ok(None);
    }
    // demand DIM
    let pos = read_keyword(lexer, Keyword::Dim)?;
    // demand whitespace
    read_whitespace(lexer)?;
    // demand variable name
    let var_name_node = demand(lexer, name::try_read, "Expected variable name")?;
    let is_long = in_transaction(lexer, |lexer| {
        read_whitespace(lexer)?;
        read_keyword(lexer, Keyword::As)
    })?
    .is_some();
    if !is_long {
        return Ok(Some(
            Statement::Dim(DimDefinition::Compact(var_name_node.strip_location())).at(pos),
        ));
    }
    // explicit type requires a bare name
    let bare_name = match var_name_node.as_ref() {
        Name::Bare(b) => b.clone(),
        Name::Qualified(_) => {
            return Err(ParserError::SyntaxError(
                "Identifier cannot end with %, &, !, #, or $".to_string(),
                var_name_node.location(),
            ));
        }
    };
    // demand whitespace after AS
    read_whitespace(lexer)?;
    // demand type name
    let next = lexer.read()?;
    let var_type = match next {
        LexemeNode::Keyword(Keyword::Double, _, _) => {
            DimType::BuiltInType(TypeQualifier::HashDouble)
        }
        LexemeNode::Keyword(Keyword::Integer, _, _) => {
            DimType::BuiltInType(TypeQualifier::PercentInteger)
        }
        LexemeNode::Keyword(Keyword::Long, _, _) => {
            DimType::BuiltInType(TypeQualifier::AmpersandLong)
        }
        LexemeNode::Keyword(Keyword::Single, _, _) => {
            DimType::BuiltInType(TypeQualifier::BangSingle)
        }
        LexemeNode::Keyword(Keyword::String_, _, _) => {
            DimType::BuiltInType(TypeQualifier::DollarString)
        }
        LexemeNode::Word(w, _) => DimType::UserDefinedType(w.into()),
        _ => {
            return Err(ParserError::SyntaxError(
                "Expected: INTEGER or LONG or SINGLE or DOUBLE or STRING or identifier".to_string(),
                next.location(),
            ))
        }
    };
    Ok(Statement::Dim(DimDefinition::Extended(bare_name, var_type)).at(pos)).map(|x| Some(x))
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
                vec![
                    TopLevelToken::Statement(Statement::Dim(DimDefinition::Extended(
                        "X".into(),
                        DimType::BuiltInType(TypeQualifier::$qualifier)
                    )))
                    .at_rc(1, 1)
                ]
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
        ($keyword: literal) => {
            let input = format!("DIM X{}", $keyword);
            let p = parse(input.as_ref());
            assert_eq!(
                p,
                vec![
                    TopLevelToken::Statement(Statement::Dim(DimDefinition::Compact(
                        format!("X{}", $keyword).into()
                    )))
                    .at_rc(1, 1)
                ]
            );
        };
    }

    #[test]
    fn test_parse_dim_compact_single() {
        assert_parse_dim_compact!("!");
    }

    #[test]
    fn test_parse_dim_compact_double() {
        assert_parse_dim_compact!("#");
    }

    #[test]
    fn test_parse_dim_compact_string() {
        assert_parse_dim_compact!("$");
    }

    #[test]
    fn test_parse_dim_compact_integer() {
        assert_parse_dim_compact!("%");
    }

    #[test]
    fn test_parse_dim_compact_long() {
        assert_parse_dim_compact!("&");
    }

    #[test]
    fn test_parse_dim_compact_bare() {
        assert_parse_dim_compact!("");
    }
}
