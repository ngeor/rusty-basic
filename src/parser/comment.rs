// parses comments

use crate::common::*;
use crate::lexer::*;

use crate::parser::types::*;
use std::io::BufRead;

pub fn try_read<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Option<StatementNode>, QErrorNode> {
    if !lexer.peek_ng().is_symbol('\'') {
        return Ok(None);
    }
    let pos = lexer.read()?.pos();
    let mut buf = String::new();
    loop {
        if lexer.peek_ng().is_eol_or_eof() {
            break;
        }
        let Locatable { element: n, .. } = lexer.read()?;
        buf.push_str(n.to_string().as_ref());
    }
    Ok(Statement::Comment(buf).at(pos)).map(|x| Some(x))
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use super::*;

    #[test]
    fn test_comment_until_eof() {
        let input = "' just a comment . 123 AS";
        let program = parse(input);
        assert_eq!(
            program,
            vec![TopLevelToken::Statement(Statement::Comment(
                " just a comment . 123 AS".to_string()
            ))
            .at_rc(1, 1)]
        );
    }
}
