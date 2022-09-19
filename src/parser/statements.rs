use crate::common::*;
use crate::parser::base::delimited_pc::DelimitedTrait;
use crate::parser::base::guard_pc::GuardTrait;
use crate::parser::base::parsers::{
    AndOptFactoryTrait, HasOutput, KeepLeftTrait, ManyTrait, NonOptParser, Parser,
};
use crate::parser::base::tokenizers::Tokenizer;
use crate::parser::base::undo_pc::Undo;
use crate::parser::specific::keyword_choice::keyword_choice_p;
use crate::parser::specific::whitespace::WhitespaceTrait;
use crate::parser::specific::with_pos::WithPosTrait;
use crate::parser::specific::{item_p, OrSyntaxErrorTrait};
use crate::parser::statement;
use crate::parser::statement_separator::Separator;
use crate::parser::types::*;

pub fn single_line_non_comment_statements_p() -> impl Parser<Output = StatementNodes> {
    statement::single_line_non_comment_statement_p()
        .with_pos()
        .one_or_more_delimited_by(
            item_p(':').surrounded_by_opt_ws(),
            QError::syntax_error("Error: trailing colon"),
        )
        .preceded_by_req_ws()
}

pub fn single_line_statements_p() -> impl Parser<Output = StatementNodes> {
    statement::single_line_statement_p()
        .with_pos()
        .one_or_more_delimited_by(
            item_p(':').surrounded_by_opt_ws(),
            QError::syntax_error("Error: trailing colon"),
        )
        .preceded_by_req_ws()
}

pub fn zero_or_more_statements_opt_lazy(
    keywords: &[Keyword],
) -> impl Parser<Output = StatementNodes> + '_ {
    LazyZeroOrMoreStatements { keywords }
}

struct LazyZeroOrMoreStatements<'a> {
    keywords: &'a [Keyword],
}

impl<'a> HasOutput for LazyZeroOrMoreStatements<'a> {
    type Output = StatementNodes;
}

impl<'a> Parser for LazyZeroOrMoreStatements<'a> {
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        zero_or_more_statements_p(keyword_choice_p(self.keywords)).parse(tokenizer)
    }
}

// When `zero_or_more_statements_p` is called, it must always read first the first statement separator.
// `top_level_token` handles the case where the first statement does not start with
// a separator.
pub fn zero_or_more_statements_p<S>(exit_source: S) -> impl Parser<Output = StatementNodes>
where
    S: Parser,
    S::Output: Undo,
{
    // first separator
    // loop
    //      negate exit source
    //      statement node and separator
    Separator::NonComment.then_use(guarded_statement(exit_source).zero_or_more())
}

pub fn zero_or_more_statements_non_opt<S>(
    exit_source: S,
) -> impl NonOptParser<Output = StatementNodes>
where
    S: Parser,
    S::Output: Undo,
{
    // first separator
    // loop
    //      negate exit source
    //      statement node and separator
    Separator::NonComment
        .or_syntax_error("Expected: end-of-statement")
        .then_use(guarded_statement(exit_source).zero_or_more())
}

fn guarded_statement<S>(exit_source: S) -> impl Parser<Output = StatementNode>
where
    S: Parser,
    S::Output: Undo,
{
    NegateParser(exit_source).then_use(statement_and_separator())
}

fn statement_and_separator() -> impl Parser<Output = StatementNode> {
    statement::statement_p()
        .with_pos()
        .and_opt_factory(|statement_node| {
            if let Statement::Comment(_) = statement_node.as_ref() {
                Separator::Comment
            } else {
                Separator::NonComment
            }
        })
        .keep_left()
}

struct NegateParser<P>(P);

impl<P> HasOutput for NegateParser<P> {
    type Output = ();
}

impl<P> Parser for NegateParser<P>
where
    P: Parser,
    P::Output: Undo,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.0.parse(tokenizer)? {
            Some(value) => {
                value.undo(tokenizer);
                Ok(None)
            }
            None => Ok(Some(())),
        }
    }
}
