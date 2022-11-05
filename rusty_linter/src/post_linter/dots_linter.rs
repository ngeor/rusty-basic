use super::post_conversion_linter::PostConversionLinter;
use rusty_common::*;
use rusty_parser::*;

use std::collections::HashSet;

#[derive(Default)]
pub struct DotsLinter {
    user_defined_names: HashSet<CaseInsensitiveString>,
}

trait NoDotNamesCheck<T, E> {
    fn ensure_no_dots(&self, x: &T) -> Result<(), E>;
}

impl NoDotNamesCheck<FunctionImplementation, QErrorPos> for DotsLinter {
    fn ensure_no_dots(&self, x: &FunctionImplementation) -> Result<(), QErrorPos> {
        self.ensure_no_dots(&x.name)?;
        self.ensure_no_dots(&x.params)
    }
}

impl NoDotNamesCheck<SubImplementation, QErrorPos> for DotsLinter {
    fn ensure_no_dots(&self, x: &SubImplementation) -> Result<(), QErrorPos> {
        self.ensure_no_dots(&x.name)?;
        self.ensure_no_dots(&x.params)
    }
}

impl NoDotNamesCheck<Vec<Positioned<Parameter>>, QErrorPos> for DotsLinter {
    fn ensure_no_dots(&self, x: &Vec<Positioned<Parameter>>) -> Result<(), QErrorPos> {
        x.iter().try_for_each(|x| self.ensure_no_dots(x))
    }
}

impl NoDotNamesCheck<Positioned<Parameter>, QErrorPos> for DotsLinter {
    fn ensure_no_dots(&self, x: &Positioned<Parameter>) -> Result<(), QErrorPos> {
        let Positioned { element, pos } = x;
        self.ensure_no_dots(element).with_err_at(pos)
    }
}

impl NoDotNamesCheck<Parameter, QError> for DotsLinter {
    fn ensure_no_dots(&self, x: &Parameter) -> Result<(), QError> {
        self.ensure_no_dots(&x.bare_name)
    }
}

impl NoDotNamesCheck<DimVarPos, QErrorPos> for DotsLinter {
    fn ensure_no_dots(&self, x: &DimVarPos) -> Result<(), QErrorPos> {
        let Positioned { element, pos } = x;
        self.ensure_no_dots(element).with_err_at(pos)
    }
}

impl NoDotNamesCheck<DimVar, QError> for DotsLinter {
    fn ensure_no_dots(&self, x: &DimVar) -> Result<(), QError> {
        self.ensure_no_dots(&x.bare_name)
    }
}

impl NoDotNamesCheck<NamePos, QErrorPos> for DotsLinter {
    fn ensure_no_dots(&self, x: &NamePos) -> Result<(), QErrorPos> {
        let name = &x.element;
        self.ensure_no_dots(name).with_err_at(x)
    }
}

impl NoDotNamesCheck<Name, QError> for DotsLinter {
    fn ensure_no_dots(&self, name: &Name) -> Result<(), QError> {
        self.ensure_no_dots(name.bare_name())
    }
}

impl NoDotNamesCheck<BareNamePos, QErrorPos> for DotsLinter {
    fn ensure_no_dots(&self, x: &BareNamePos) -> Result<(), QErrorPos> {
        let Positioned { element, pos } = x;
        self.ensure_no_dots(element).with_err_at(pos)
    }
}

impl NoDotNamesCheck<BareName, QError> for DotsLinter {
    fn ensure_no_dots(&self, x: &BareName) -> Result<(), QError> {
        match x.prefix('.') {
            Some(part_before_dot) => {
                if self.user_defined_names.contains(part_before_dot) {
                    Err(QError::DotClash)
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }
}

impl NoDotNamesCheck<Expressions, QErrorPos> for DotsLinter {
    fn ensure_no_dots(&self, x: &Expressions) -> Result<(), QErrorPos> {
        x.iter().try_for_each(|x| self.ensure_no_dots(x))
    }
}

impl NoDotNamesCheck<ExpressionPos, QErrorPos> for DotsLinter {
    fn ensure_no_dots(&self, x: &ExpressionPos) -> Result<(), QErrorPos> {
        let Positioned { element, pos } = x;
        self.ensure_no_dots(element).patch_err_pos(pos)
    }
}

impl NoDotNamesCheck<Expression, QErrorPos> for DotsLinter {
    fn ensure_no_dots(&self, x: &Expression) -> Result<(), QErrorPos> {
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
    fn visit_program(&mut self, p: &Program) -> Result<(), QErrorPos> {
        let mut collector = UserDefinedNamesCollector::default();
        collector.visit_program(p)?;
        self.user_defined_names = collector.user_defined_names;
        self.visit_global_statements(p)
    }

    fn visit_function_implementation(
        &mut self,
        f: &FunctionImplementation,
    ) -> Result<(), QErrorPos> {
        self.ensure_no_dots(f)?;
        self.visit_statements(&f.body)
    }

    fn visit_sub_implementation(&mut self, s: &SubImplementation) -> Result<(), QErrorPos> {
        self.ensure_no_dots(s)?;
        self.visit_statements(&s.body)
    }

    fn visit_dim(&mut self, dim_list: &DimList) -> Result<(), QErrorPos> {
        dim_list
            .variables
            .iter()
            .try_for_each(|dim_var_pos| self.ensure_no_dots(dim_var_pos))
    }

    fn visit_assignment(&mut self, name: &Expression, v: &ExpressionPos) -> Result<(), QErrorPos> {
        self.ensure_no_dots(name)?;
        self.visit_expression(v)
    }

    fn visit_for_loop(&mut self, f: &ForLoop) -> Result<(), QErrorPos> {
        // no need to test f.next_counter, as it is the same as variable_name if it exists
        self.ensure_no_dots(&f.variable_name)?;
        self.visit_expression(&f.lower_bound)?;
        self.visit_expression(&f.upper_bound)?;
        match &f.step {
            Some(s) => self.visit_expression(s)?,
            None => (),
        }
        self.visit_statements(&f.statements)
    }

    fn visit_expression(&mut self, e: &ExpressionPos) -> Result<(), QErrorPos> {
        self.ensure_no_dots(e)
    }
}

#[derive(Default)]
struct UserDefinedNamesCollector {
    user_defined_names: HashSet<CaseInsensitiveString>,
}

impl UserDefinedNamesCollector {
    fn visit_names<T>(&mut self, params: &Vec<Positioned<TypedName<T>>>)
    where
        T: VarTypeToUserDefinedRecursively,
    {
        self.user_defined_names.extend(
            params
                .iter()
                .map(|dim_var_pos| &dim_var_pos.element)
                .filter(|dim_name| dim_name.var_type.as_user_defined_recursively().is_some())
                .map(|dim_name| &dim_name.bare_name)
                .cloned(),
        );
    }
}

impl PostConversionLinter for UserDefinedNamesCollector {
    fn visit_function_implementation(
        &mut self,
        f: &FunctionImplementation,
    ) -> Result<(), QErrorPos> {
        self.visit_names(&f.params);
        self.visit_statements(&f.body)
    }

    fn visit_sub_implementation(&mut self, s: &SubImplementation) -> Result<(), QErrorPos> {
        self.visit_names(&s.params);
        self.visit_statements(&s.body)
    }

    fn visit_dim(&mut self, dim_list: &DimList) -> Result<(), QErrorPos> {
        self.visit_names(&dim_list.variables);
        Ok(())
    }
}
