use crate::common::QError;
use crate::parser::expression::guarded_expression_node_p;
use crate::parser::pc::{Parser, Tokenizer};
use crate::parser::pc_specific::{keyword, whitespace_boundary};
use crate::parser::types::Keyword;
use crate::parser::ExpressionNode;
use crate::parser_declaration;

parser_declaration!(
    pub struct OptSecondExpressionParser {
        keyword: Keyword,
    }
);

pub trait ExtractExpression {
    fn to_expression(&self) -> &ExpressionNode;
}

impl ExtractExpression for ExpressionNode {
    fn to_expression(&self) -> &ExpressionNode {
        self
    }
}

impl<P> Parser for OptSecondExpressionParser<P>
where
    P: Parser,
    P::Output: ExtractExpression,
{
    type Output = (P::Output, Option<ExpressionNode>);

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let first = self.parser.parse(tokenizer)?;
        let first_expr = first.to_expression();
        let is_paren = first_expr.as_ref().is_parenthesis();
        let opt_right = whitespace_boundary(is_paren)
            .and(keyword(self.keyword))
            .then_demand(
                guarded_expression_node_p().or_fail(QError::SyntaxError(format!(
                    "Expected: expression after {}",
                    self.keyword
                ))),
            )
            .allow_none()
            .parse(tokenizer)?;
        Ok((first, opt_right))
    }
}
