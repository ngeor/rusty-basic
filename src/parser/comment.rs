use crate::common::*;
use crate::lexer::*;

use crate::parser::types::*;
use std::io::BufRead;

/// Tries to read a comment
pub fn try_read<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Option<StatementNode>, QErrorNode> {
    // TODO try to avoid collect
    // TODO deprecate try_read and switch to Option Result signature
    let mut lexeme_nodes = lexer
        // read if we have a ' symbol
        .take_if(|x| x.is_symbol('\''))
        // keep reading until we hit eol
        .take_while(|x| !x.is_eol())
        // collect all lexemes including the ' (we need its pos)
        .collect()?;
    // pop the ' lexeme to get its position later
    Ok(lexeme_nodes.pop_front().map(|first| {
        Statement::Comment(lexeme_nodes.into_iter().fold(
            String::new(),
            |acc, Locatable { element, .. }| {
                // fold all remaining lexemes into a string
                format!("{}{}", acc, element) // concatenate strings
            },
        ))
        .at(first.pos()) // with the pos of the ' symbol
    }))
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

    #[test]
    fn test_comment_at_eof() {
        let input = "'";
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                TopLevelToken::Statement(Statement::Comment(String::new())).at_rc(1, 1)
            ]
        );
    }
}
