use super::post_conversion_linter::PostConversionLinter;
use crate::common::*;
use crate::parser::{
    BareName, BareNameNode, DimList, DimName, DimNameNode, Expression, ExpressionNode, ForLoopNode,
    FunctionImplementation, Name, NameNode, ParamName, QualifiedName, QualifiedNameNode,
    SubImplementation,
};
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

impl<'a> NoDotNamesCheck<Vec<Locatable<ParamName>>, QErrorNode> for DotsLinter<'a> {
    fn ensure_no_dots(&self, x: &Vec<Locatable<ParamName>>) -> Result<(), QErrorNode> {
        x.into_iter().map(|x| self.ensure_no_dots(x)).collect()
    }
}

impl<'a> NoDotNamesCheck<Locatable<ParamName>, QErrorNode> for DotsLinter<'a> {
    fn ensure_no_dots(&self, x: &Locatable<ParamName>) -> Result<(), QErrorNode> {
        let Locatable { element, pos } = x;
        self.ensure_no_dots(element).with_err_at(pos)
    }
}

impl<'a> NoDotNamesCheck<ParamName, QError> for DotsLinter<'a> {
    fn ensure_no_dots(&self, x: &ParamName) -> Result<(), QError> {
        let bare_name: &BareName = x.as_ref();
        self.ensure_no_dots(bare_name)
    }
}

impl<'a> NoDotNamesCheck<DimNameNode, QErrorNode> for DotsLinter<'a> {
    fn ensure_no_dots(&self, x: &DimNameNode) -> Result<(), QErrorNode> {
        let Locatable { element, pos } = x;
        self.ensure_no_dots(element).with_err_at(pos)
    }
}

impl<'a> NoDotNamesCheck<DimName, QError> for DotsLinter<'a> {
    fn ensure_no_dots(&self, x: &DimName) -> Result<(), QError> {
        self.ensure_no_dots(x.bare_name())
    }
}

impl<'a> NoDotNamesCheck<NameNode, QErrorNode> for DotsLinter<'a> {
    fn ensure_no_dots(&self, x: &NameNode) -> Result<(), QErrorNode> {
        let name = x.as_ref();
        self.ensure_no_dots(name).with_err_at(x)
    }
}

impl<'a> NoDotNamesCheck<Name, QError> for DotsLinter<'a> {
    fn ensure_no_dots(&self, x: &Name) -> Result<(), QError> {
        match x {
            Name::Bare(bare_name) | Name::Qualified(QualifiedName { bare_name, .. }) => {
                self.ensure_no_dots(bare_name)
            }
        }
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
        let QualifiedName {
            bare_name: name, ..
        } = x;
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
            Expression::Variable(var_name, _) => self.ensure_no_dots(var_name).with_err_no_pos(),
            Expression::ArrayElement(var_name, indices, _) => {
                self.ensure_no_dots(var_name).with_err_no_pos()?;
                self.ensure_no_dots(indices)
            }
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
    fn visit_function_implementation(
        &mut self,
        f: &FunctionImplementation,
    ) -> Result<(), QErrorNode> {
        self.ensure_no_dots(f)?;
        self.visit_statement_nodes(&f.body)
    }

    fn visit_sub_implementation(&mut self, s: &SubImplementation) -> Result<(), QErrorNode> {
        self.ensure_no_dots(s)?;
        self.visit_statement_nodes(&s.body)
    }

    fn visit_dim(&mut self, dim_list: &DimList) -> Result<(), QErrorNode> {
        dim_list
            .variables
            .iter()
            .map(|dim_name_node| self.ensure_no_dots(dim_name_node))
            .collect()
    }

    fn visit_assignment(
        &mut self,
        name: &Expression,
        v: &ExpressionNode,
    ) -> Result<(), QErrorNode> {
        self.ensure_no_dots(name)?;
        self.visit_expression(v)
    }

    fn visit_for_loop(&mut self, f: &ForLoopNode) -> Result<(), QErrorNode> {
        // no need to test f.next_counter, as it is the same as variable_name if it exists
        self.ensure_no_dots(&f.variable_name)?;
        self.visit_expression(&f.lower_bound)?;
        self.visit_expression(&f.upper_bound)?;
        match &f.step {
            Some(s) => self.visit_expression(s)?,
            None => (),
        }
        self.visit_statement_nodes(&f.statements)
    }

    fn visit_expression(&mut self, e: &ExpressionNode) -> Result<(), QErrorNode> {
        self.ensure_no_dots(e)
    }
}
