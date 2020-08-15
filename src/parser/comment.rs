use crate::common::*;
use crate::lexer::*;

use crate::parser::types::*;
use std::io::BufRead;

/// Tries to read a comment
pub fn next<T: BufRead>(lexer: &mut BufLexer<T>) -> Option<Result<StatementNode, QErrorNode>> {
    let mut opt_pos: Option<Location> = None;
    lexer
        // read if we have a ' symbol
        .take_if(|lexeme_node| lexeme_node.is_symbol('\''))
        // capture pos, to be able to tell if `fold` returned an empty string because there was an empty comment or if because there was no comment at all
        .tap_next(|lexeme_node| opt_pos = Some(lexeme_node.pos()))
        // keep reading until we hit eol
        .take_while(|x| !x.is_eol())
        // fold all remaining lexemes into a string. This creates an empty string even if we never hit a comment,
        // which is why we use the `opt_pos` to map it back to None if needed
        .fold(String::new(), |acc, Locatable { element, .. }| {
            format!("{}{}", acc, element) // concatenate strings
        })
        .map_ok(|text| Statement::Comment(text))
        .with_ok_pos(opt_pos)
}

#[deprecated]
pub fn try_read<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Option<StatementNode>, QErrorNode> {
    next(lexer).transpose()
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
            vec![TopLevelToken::Statement(Statement::Comment(String::new())).at_rc(1, 1)]
        );
    }
}
