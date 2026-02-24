use rusty_parser::{CaseBlock, CaseExpression, SelectCase};

use crate::converter::common::{Convertible, ConvertibleIn};
use crate::core::{Context, LintErrorPos};

impl Convertible for SelectCase {
    fn convert(self, ctx: &mut Context) -> Result<Self, LintErrorPos> {
        let expr = self.expr.convert_in_default(ctx)?;
        let case_blocks = self.case_blocks.convert(ctx)?;
        let else_block = self.else_block.convert(ctx)?;
        let inline_comments = self.inline_comments;
        Ok(Self {
            expr,
            case_blocks,
            else_block,
            inline_comments,
        })
    }
}

impl Convertible for CaseBlock {
    fn convert(self, ctx: &mut Context) -> Result<Self, LintErrorPos> {
        let (expression_list, statements) = self.into();
        let expression_list = expression_list.convert(ctx)?;
        let statements = statements.convert(ctx)?;
        Ok(Self::new(expression_list, statements))
    }
}

impl Convertible for CaseExpression {
    fn convert(self, ctx: &mut Context) -> Result<Self, LintErrorPos> {
        match self {
            Self::Simple(e) => e.convert_in_default(ctx).map(CaseExpression::Simple),
            Self::Is(op, e) => e.convert_in_default(ctx).map(|e| Self::Is(op, e)),
            Self::Range(from, to) => {
                let from = from.convert_in_default(ctx)?;
                let to = to.convert_in_default(ctx)?;
                Ok(Self::Range(from, to))
            }
        }
    }
}
