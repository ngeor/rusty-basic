use super::post_conversion_linter::PostConversionLinter;
use crate::built_ins::{lint_function_call, lint_sub_call};
use crate::core::{LintErrorPos, NameContext};
use rusty_common::*;
use rusty_parser::*;

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
    ) -> Result<(), LintErrorPos> {
        self.name_context = NameContext::Function;
        let result = self.visit_statements(&f.body);
        self.name_context = NameContext::Global;
        result
    }

    fn visit_sub_implementation(&mut self, s: &SubImplementation) -> Result<(), LintErrorPos> {
        self.name_context = NameContext::Sub;
        let result = self.visit_statements(&s.body);
        self.name_context = NameContext::Global;
        result
    }

    fn visit_built_in_sub_call(
        &mut self,
        built_in_sub_call: &BuiltInSubCall,
        pos: Position,
    ) -> Result<(), LintErrorPos> {
        let (built_in_sub, args) = built_in_sub_call.into();
        self.visit_expressions(args)?;
        lint_sub_call(built_in_sub, pos, args, self.name_context)
    }

    fn visit_expression(&mut self, expr_pos: &ExpressionPos) -> Result<(), LintErrorPos> {
        let pos = expr_pos.pos();
        match &expr_pos.element {
            Expression::BuiltInFunctionCall(built_in_function, args) => {
                for x in args {
                    self.visit_expression(x)?;
                }
                lint_function_call(built_in_function, pos, args)
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
