use super::post_conversion_linter::PostConversionLinter;
use crate::common::*;
use crate::parser::*;

use std::collections::HashSet;

#[derive(Default)]
pub struct DotsLinter {
    user_defined_names: HashSet<CaseInsensitiveString>,
}

trait NoDotNamesCheck<T, E> {
    fn ensure_no_dots(&self, x: &T) -> Result<(), E>;
}

impl NoDotNamesCheck<FunctionImplementation, QErrorNode> for DotsLinter {
    fn ensure_no_dots(&self, x: &FunctionImplementation) -> Result<(), QErrorNode> {
        self.ensure_no_dots(&x.name)?;
        self.ensure_no_dots(&x.params)
    }
}

impl NoDotNamesCheck<SubImplementation, QErrorNode> for DotsLinter {
    fn ensure_no_dots(&self, x: &SubImplementation) -> Result<(), QErrorNode> {
        self.ensure_no_dots(&x.name)?;
        self.ensure_no_dots(&x.params)
    }
}

impl NoDotNamesCheck<Vec<Locatable<ParamName>>, QErrorNode> for DotsLinter {
    fn ensure_no_dots(&self, x: &Vec<Locatable<ParamName>>) -> Result<(), QErrorNode> {
        x.iter().try_for_each(|x| self.ensure_no_dots(x))
    }
}

impl NoDotNamesCheck<Locatable<ParamName>, QErrorNode> for DotsLinter {
    fn ensure_no_dots(&self, x: &Locatable<ParamName>) -> Result<(), QErrorNode> {
        let Locatable { element, pos } = x;
        self.ensure_no_dots(element).with_err_at(pos)
    }
}

impl NoDotNamesCheck<ParamName, QError> for DotsLinter {
    fn ensure_no_dots(&self, x: &ParamName) -> Result<(), QError> {
        let bare_name: &BareName = x.bare_name();
        self.ensure_no_dots(bare_name)
    }
}

impl NoDotNamesCheck<DimNameNode, QErrorNode> for DotsLinter {
    fn ensure_no_dots(&self, x: &DimNameNode) -> Result<(), QErrorNode> {
        let Locatable { element, pos } = x;
        self.ensure_no_dots(element).with_err_at(pos)
    }
}

impl NoDotNamesCheck<DimName, QError> for DotsLinter {
    fn ensure_no_dots(&self, x: &DimName) -> Result<(), QError> {
        self.ensure_no_dots(x.bare_name())
    }
}

impl NoDotNamesCheck<NameNode, QErrorNode> for DotsLinter {
    fn ensure_no_dots(&self, x: &NameNode) -> Result<(), QErrorNode> {
        let name = x.as_ref();
        self.ensure_no_dots(name).with_err_at(x)
    }
}

impl NoDotNamesCheck<Name, QError> for DotsLinter {
    fn ensure_no_dots(&self, name: &Name) -> Result<(), QError> {
        self.ensure_no_dots(name.bare_name())
    }
}

impl NoDotNamesCheck<BareNameNode, QErrorNode> for DotsLinter {
    fn ensure_no_dots(&self, x: &BareNameNode) -> Result<(), QErrorNode> {
        let Locatable { element, pos } = x;
        self.ensure_no_dots(element).with_err_at(pos)
    }
}

impl NoDotNamesCheck<BareName, QError> for DotsLinter {
    fn ensure_no_dots(&self, x: &BareName) -> Result<(), QError> {
        match x.prefix('.') {
            Some(first) => {
                if self.user_defined_names.contains(&first) {
                    Err(QError::DotClash)
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }
}

impl NoDotNamesCheck<ExpressionNodes, QErrorNode> for DotsLinter {
    fn ensure_no_dots(&self, x: &ExpressionNodes) -> Result<(), QErrorNode> {
        x.iter().try_for_each(|x| self.ensure_no_dots(x))
    }
}

impl NoDotNamesCheck<ExpressionNode, QErrorNode> for DotsLinter {
    fn ensure_no_dots(&self, x: &ExpressionNode) -> Result<(), QErrorNode> {
        let Locatable { element, pos } = x;
        self.ensure_no_dots(element).patch_err_pos(pos)
    }
}

impl NoDotNamesCheck<Expression, QErrorNode> for DotsLinter {
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

impl PostConversionLinter for DotsLinter {
    fn visit_program(&mut self, p: &ProgramNode) -> Result<(), QErrorNode> {
        let mut collector = UserDefinedNamesCollector::default();
        collector.visit_program(p)?;
        self.user_defined_names = collector.user_defined_names;
        self.visit_top_level_token_nodes(p)
    }

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
            .try_for_each(|dim_name_node| self.ensure_no_dots(dim_name_node))
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

#[derive(Default)]
struct UserDefinedNamesCollector {
    user_defined_names: HashSet<CaseInsensitiveString>,
}

impl UserDefinedNamesCollector {
    fn visit_names<T>(&mut self, params: &Vec<Locatable<VarName<T>>>)
    where
        T: VarTypeToUserDefinedRecursively,
    {
        self.user_defined_names.extend(
            params
                .iter()
                .map(|dim_name_node| dim_name_node.as_ref())
                .filter(|dim_name| dim_name.var_type().as_user_defined_recursively().is_some())
                .map(|dim_name| dim_name.bare_name())
                .cloned(),
        );
    }
}

impl PostConversionLinter for UserDefinedNamesCollector {
    fn visit_function_implementation(
        &mut self,
        f: &FunctionImplementation,
    ) -> Result<(), QErrorNode> {
        self.visit_names(&f.params);
        self.visit_statement_nodes(&f.body)
    }

    fn visit_sub_implementation(&mut self, s: &SubImplementation) -> Result<(), QErrorNode> {
        self.visit_names(&s.params);
        self.visit_statement_nodes(&s.body)
    }

    fn visit_dim(&mut self, dim_list: &DimList) -> Result<(), QErrorNode> {
        self.visit_names(&dim_list.variables);
        Ok(())
    }
}
