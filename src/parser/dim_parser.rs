// parses DIM statement

use crate::common::*;
use crate::lexer::*;
use crate::parser::buf_lexer::*;
use crate::parser::error::*;
use crate::parser::types::*;
use std::io::BufRead;

// TODO get rid also of the <T: BufRead> from here

pub fn try_read<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Option<StatementNode>, ParserError> {
    // try to read DIM, if it succeeds demand it, else return None
    if peek_keyword(lexer, Keyword::Dim)? {
        // demand DIM
        let pos = read_keyword(lexer, Keyword::Dim)?;
        // demand whitespace
        read_whitespace(lexer)?;
        // demand variable name
        let var_name = read_bare_name(lexer)?;
        // demand whitespace
        read_whitespace(lexer)?;
        // demand keyword AS
        read_keyword(lexer, Keyword::As)?;
        // demand whitespace
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
            _ => return Err(ParserError::SyntaxError(next)),
        };
        Ok(Statement::Dim(var_name, var_type).at(pos)).map(|x| Some(x))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use crate::common::*;
    use crate::parser::test_utils::*;
    use crate::parser::types::*;

    #[test]
    fn test_parse_dim() {
        let input = "DIM X AS STRING";
        let p = parse(input);
        assert_eq!(
            p,
            vec![TopLevelToken::Statement(Statement::Dim(
                "X".into(),
                DimType::BuiltInType(TypeQualifier::DollarString)
            ))
            .at_rc(1, 1)]
        );
    }
}
