use super::post_conversion_linter::PostConversionLinter;
use crate::common::*;
use crate::linter::types::*;
use crate::parser::{BareName, BareNameNode, QualifiedName, QualifiedNameNode};
use crate::variant::Variant;
use std::collections::HashSet;

pub struct DotsLinter<'a> {
    pub names_without_dot: &'a HashSet<CaseInsensitiveString>,
}

trait NoDotNamesCheck<T, E> {
    fn ensure_no_dots(&self, x: &T) -> Result<(), E>;
}

impl<'a> NoDotNamesCheck<FunctionImplementation, QErrorNode> for DotsLinter<'a> {
    fn ensure_no_dots(&self, x: &FunctionImplementation) -> Result<(), QErrorNode> {
        self.ensure_no_dots(&x.name)?;
        self.ensure_no_dots(&x.params)
    }
}

impl<'a> NoDotNamesCheck<SubImplementation, QErrorNode> for DotsLinter<'a> {
    fn ensure_no_dots(&self, x: &SubImplementation) -> Result<(), QErrorNode> {
        self.ensure_no_dots(&x.name)?;
        self.ensure_no_dots(&x.params)
    }
}

impl<'a> NoDotNamesCheck<Vec<Locatable<ResolvedParamName>>, QErrorNode> for DotsLinter<'a> {
    fn ensure_no_dots(&self, x: &Vec<Locatable<ResolvedParamName>>) -> Result<(), QErrorNode> {
        x.into_iter().map(|x| self.ensure_no_dots(x)).collect()
    }
}

impl<'a> NoDotNamesCheck<Locatable<ResolvedParamName>, QErrorNode> for DotsLinter<'a> {
    fn ensure_no_dots(&self, x: &Locatable<ResolvedParamName>) -> Result<(), QErrorNode> {
        let Locatable { element, pos } = x;
        self.ensure_no_dots(element).with_err_at(pos)
    }
}

impl<'a> NoDotNamesCheck<ResolvedParamName, QError> for DotsLinter<'a> {
    fn ensure_no_dots(&self, x: &ResolvedParamName) -> Result<(), QError> {
        let bare_name: &BareName = x.as_ref();
        self.ensure_no_dots(bare_name)
    }
}

impl<'a> NoDotNamesCheck<ResolvedDeclaredNameNode, QErrorNode> for DotsLinter<'a> {
    fn ensure_no_dots(&self, x: &ResolvedDeclaredNameNode) -> Result<(), QErrorNode> {
        let Locatable { element, pos } = x;
        self.ensure_no_dots(element).with_err_at(pos)
    }
}

impl<'a> NoDotNamesCheck<ResolvedDeclaredName, QError> for DotsLinter<'a> {
    fn ensure_no_dots(&self, x: &ResolvedDeclaredName) -> Result<(), QError> {
        let bare_name: &BareName = x.as_ref();
        self.ensure_no_dots(bare_name)
    }
}

impl<'a> NoDotNamesCheck<QualifiedNameNode, QErrorNode> for DotsLinter<'a> {
    fn ensure_no_dots(&self, x: &QualifiedNameNode) -> Result<(), QErrorNode> {
        let Locatable { element, pos } = x;
        self.ensure_no_dots(element).with_err_at(pos)
    }
}

impl<'a> NoDotNamesCheck<BareNameNode, QErrorNode> for DotsLinter<'a> {
    fn ensure_no_dots(&self, x: &BareNameNode) -> Result<(), QErrorNode> {
        let Locatable { element, pos } = x;
        self.ensure_no_dots(element).with_err_at(pos)
    }
}

impl<'a> NoDotNamesCheck<QualifiedName, QError> for DotsLinter<'a> {
    fn ensure_no_dots(&self, x: &QualifiedName) -> Result<(), QError> {
        let QualifiedName { name, .. } = x;
        self.ensure_no_dots(name)
    }
}

impl<'a> NoDotNamesCheck<BareName, QError> for DotsLinter<'a> {
    fn ensure_no_dots(&self, x: &BareName) -> Result<(), QError> {
        match x.prefix('.') {
            Some(first) => {
                if self.names_without_dot.contains(&first) {
                    Err(QError::DotClash)
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }
}

impl<'a> NoDotNamesCheck<Vec<ExpressionNode>, QErrorNode> for DotsLinter<'a> {
    fn ensure_no_dots(&self, x: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        x.into_iter().map(|x| self.ensure_no_dots(x)).collect()
    }
}

impl<'a> NoDotNamesCheck<ExpressionNode, QErrorNode> for DotsLinter<'a> {
    fn ensure_no_dots(&self, x: &ExpressionNode) -> Result<(), QErrorNode> {
        let Locatable { element, pos } = x;
        self.ensure_no_dots(element).patch_err_pos(pos)
    }
}

impl<'a> NoDotNamesCheck<Expression, QErrorNode> for DotsLinter<'a> {
    fn ensure_no_dots(&self, x: &Expression) -> Result<(), QErrorNode> {
        match x {
            Expression::Constant(qualified_name) => {
                self.ensure_no_dots(qualified_name).with_err_no_pos()
            }
            Expression::Variable(resolved_declared_name) => self
                .ensure_no_dots(resolved_declared_name)
                .with_err_no_pos(),
            Expression::FunctionCall(name, args) => {
                self.ensure_no_dots(name).with_err_no_pos()?;
                self.ensure_no_dots(args)
            }
            Expression::BuiltInFunctionCall(_, args) => self.ensure_no_dots(args),
            Expression::BinaryExpression(_, left, right, _) => {
                self.ensure_no_dots(left.as_ref())?;
                self.ensure_no_dots(right.as_ref())
            }
            Expression::UnaryExpression(_, child) | Expression::Parenthesis(child) => {
                self.ensure_no_dots(child.as_ref())
            }
            _ => Ok(()),
        }
    }
}

impl<'a> PostConversionLinter for DotsLinter<'a> {
    fn visit_function_implementation(&self, f: &FunctionImplementation) -> Result<(), QErrorNode> {
        self.ensure_no_dots(f)?;
        self.visit_statement_nodes(&f.body)
    }

    fn visit_sub_implementation(&self, s: &SubImplementation) -> Result<(), QErrorNode> {
        self.ensure_no_dots(s)?;
        self.visit_statement_nodes(&s.body)
    }

    fn visit_dim(&self, d: &ResolvedDeclaredNameNode) -> Result<(), QErrorNode> {
        self.ensure_no_dots(d)
    }

    fn visit_assignment(
        &self,
        name: &ResolvedDeclaredName,
        v: &ExpressionNode,
    ) -> Result<(), QErrorNode> {
        self.ensure_no_dots(name).with_err_no_pos()?;
        self.visit_expression(v)
    }

    fn visit_for_loop(&self, f: &ForLoopNode) -> Result<(), QErrorNode> {
        // TODO verify variable name

        self.visit_expression(&f.lower_bound)?;
        self.visit_expression(&f.upper_bound)?;
        match &f.step {
            Some(s) => self.visit_expression(s)?,
            None => (),
        }
        self.visit_statement_nodes(&f.statements)
    }

    fn visit_const(&self, left: &QualifiedNameNode, _right: &Variant) -> Result<(), QErrorNode> {
        self.ensure_no_dots(left)
    }

    fn visit_expression(&self, e: &ExpressionNode) -> Result<(), QErrorNode> {
        self.ensure_no_dots(e)
    }
}
