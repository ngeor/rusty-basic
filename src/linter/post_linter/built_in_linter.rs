use super::post_conversion_linter::PostConversionLinter;
use crate::built_ins::{linter, BuiltInSub};
use crate::common::*;
use crate::linter::NameContext;
use crate::parser::{
    Expression, ExpressionNode, ExpressionNodes, FunctionImplementation, SubImplementation,
};

/// Lints built-in functions and subs.
pub struct BuiltInLinter {
    name_context: NameContext,
}

impl BuiltInLinter {
    pub fn new() -> Self {
        Self {
            name_context: NameContext::Global,
        }
    }
}

impl PostConversionLinter for BuiltInLinter {
    fn visit_function_implementation(
        &mut self,
        f: &FunctionImplementation,
    ) -> Result<(), QErrorNode> {
        self.name_context = NameContext::Function;
        let result = self.visit_statement_nodes(&f.body);
        self.name_context = NameContext::Global;
        result
    }

    fn visit_sub_implementation(&mut self, s: &SubImplementation) -> Result<(), QErrorNode> {
        self.name_context = NameContext::Sub;
        let result = self.visit_statement_nodes(&s.body);
        self.name_context = NameContext::Global;
        result
    }

    fn visit_built_in_sub_call(
        &mut self,
        built_in_sub: &BuiltInSub,
        args: &ExpressionNodes,
    ) -> Result<(), QErrorNode> {
        self.visit_expressions(args)?;
        linter::lint_sub_call(built_in_sub, args, self.name_context)
    }

    fn visit_expression(&mut self, expr_node: &ExpressionNode) -> Result<(), QErrorNode> {
        let pos = expr_node.pos();
        let e = expr_node.as_ref();
        match e {
            Expression::BuiltInFunctionCall(built_in_function, args) => {
                for x in args {
                    self.visit_expression(x)?;
                }
                linter::lint_function_call(built_in_function, args).patch_err_pos(pos)
            }
            Expression::BinaryExpression(_, left, right, _) => {
                self.visit_expression(left)?;
                self.visit_expression(right)
            }
            Expression::UnaryExpression(_, child) => self.visit_expression(child),
            _ => Ok(()),
        }
    }
}
