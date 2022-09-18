use crate::common::*;
use crate::parser::base::parsers::{
    AndOptTrait, FnMapTrait, KeepLeftTrait, KeepRightTrait, ManyTrait, Parser, TokenPredicate,
};
use crate::parser::base::surrounded_by::SurroundedBy;
use crate::parser::base::tokenizers::{token_list_to_string, Token, TokenList};
use crate::parser::specific::with_pos::WithPosTrait;
use crate::parser::specific::{eol_or_whitespace, item_p, TokenType};
use crate::parser::types::*;

/// Tries to read a comment.
pub fn comment_p() -> impl Parser<Output = Statement> {
    comment_as_string().fn_map(Statement::Comment)
}

/// Reads multiple comments and the surrounding whitespace.
pub fn comments_and_whitespace_p() -> impl Parser<Output = Vec<Locatable<String>>> {
    SurroundedBy::new(
        eol_or_whitespace(),
        comment_as_string()
            .with_pos()
            .and_opt(eol_or_whitespace())
            .keep_left()
            .one_or_more(),
        eol_or_whitespace(),
    )
}

fn comment_as_string() -> impl Parser<Output = String> {
    // TODO prevent unwrap_or_default with NonOptParser
    item_p('\'')
        .and_opt(non_eol())
        .keep_right()
        .fn_map(|x| token_list_to_string(&x.unwrap_or_default()))
}

fn non_eol() -> impl Parser<Output = TokenList> {
    NonEol.parser().one_or_more()
}

struct NonEol;

impl TokenPredicate for NonEol {
    fn test(&self, token: &Token) -> bool {
        token.kind != TokenType::Eol as i32
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
