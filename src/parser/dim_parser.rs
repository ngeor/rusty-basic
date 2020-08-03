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
        let var_type = read_word_or_keyword(lexer)?;
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
            vec![TopLevelToken::Statement(Statement::Dim("X".into(), "STRING".into())).at_rc(1, 1)]
        );
    }

    #[test]
    fn test_type_mismatch() {
        // TODO move to linter
        let input = r#"
        X = 1
        IF X = 0 THEN DIM A AS STRING
        A = 42 <-- type mismatch
        "#;
        unimplemented!();
    }
}
