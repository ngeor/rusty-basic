use crate::common::*;
use crate::parser::base::parsers::{AndOptTrait, FnMapTrait, KeepLeftTrait, KeepRightTrait, ManyTrait, OptAndPC, Parser, TokenPredicate};
use crate::parser::base::tokenizers::{Token, TokenList};
use crate::parser::specific::with_pos::WithPosTrait;
use crate::parser::specific::{item_p, TokenType};
use crate::parser::types::*;

/// Tries to read a comment.
pub fn comment_p() -> impl Parser<Output = Statement> {
    item_p('\'')
        .and_opt(non_eol())
        .keep_right()
        .fn_map(|x| Statement::Comment(x.unwrap_or_default()))
}

/// Reads multiple comments and the surrounding whitespace.
pub fn comments_and_whitespace_p() -> impl Parser<Output = Vec<Locatable<String>>> {
    eol_or_whitespace()
        .map_none_to_default()
        .and_opt(
            item_p('\'')
                .with_pos()
                .and_opt(non_eol())
                .and_opt(eol_or_whitespace())
                .keep_left()
                .fn_map(|(Locatable { pos, .. }, opt_s)| opt_s.unwrap_or_default().at(pos))
                .one_or_more(),
        )
        .and_opt(eol_or_whitespace())
        .keep_middle()
        .fn_map(|x| x.unwrap_or_default())
}

fn non_eol() -> impl Parser<Output=TokenList> {
    NonEol.parser().one_or_more()
}

fn eol_or_whitespace() -> impl Parser<Output=TokenList> {
    EolOrWhitespace.parser().one_or_more()
}

struct NonEol;

impl TokenPredicate for NonEol {
    fn test(&self, token: &Token) -> bool {
        token.kind != TokenType::Eol as i32
    }
}

struct EolOrWhitespace;

impl TokenPredicate for EolOrWhitespace {
    fn test(&self, token: &Token) -> bool {
        token.kind == TokenType::Eol as i32 || token.kind == TokenType::Whitespace as i32
    }
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
