use std::collections::HashSet;

use rusty_common::*;
use rusty_parser::*;

use super::post_conversion_linter::PostConversionLinter;
use crate::core::*;

/// In QBasic, if you have anywhere in the program a variable of user defined type,
/// e.g. `DIM A AS Person`,
/// then you can't have anywhere else in the program a variable that starts with
/// that name followed by a dot,
/// e.g. `DIM A.B$` is illegal.
///
/// The QBasic behavior is probably done at the parsing level,
/// but here we catch it at the linter.
/// QBasic's behavior is to throw the error:
/// "Expected: , or end-of-statement"
/// and the error location is reported at the dot.
#[derive(Default)]
pub struct DotsLinter {
    user_defined_names: HashSet<CaseInsensitiveString>,
}

trait NoDotNamesCheck<T, E> {
    fn ensure_no_dots(&self, x: &T) -> Result<(), E>;
}

// Blanket for Positioned

impl<T, E> NoDotNamesCheck<Positioned<T>, Positioned<E>> for DotsLinter
where
    DotsLinter: NoDotNamesCheck<T, E>,
{
    fn ensure_no_dots(&self, element: &Positioned<T>) -> Result<(), Positioned<E>> {
        let Positioned { element, pos } = element;
        self.ensure_no_dots(element).map_err(|e| e.at_pos(*pos))
    }
}

// Blanket for Vec

impl<T, E> NoDotNamesCheck<Vec<T>, E> for DotsLinter
where
    DotsLinter: NoDotNamesCheck<T, E>,
{
    fn ensure_no_dots(&self, x: &Vec<T>) -> Result<(), E> {
        x.iter().try_for_each(|x| self.ensure_no_dots(x))
    }
}

// FunctionImplementation and SubImplementation

impl<T> NoDotNamesCheck<SubprogramImplementation<T>, LintErrorPos> for DotsLinter
where
    DotsLinter:
        NoDotNamesCheck<Positioned<T>, LintErrorPos> + NoDotNamesCheck<Parameters, LintErrorPos>,
{
    fn ensure_no_dots(&self, x: &SubprogramImplementation<T>) -> Result<(), LintErrorPos> {
        self.ensure_no_dots(&x.name)?;
        self.ensure_no_dots(&x.params)
    }
}

// TODO the next 4 can be merged into one with a trait like AsRef<BareName>

impl NoDotNamesCheck<Parameter, LintError> for DotsLinter {
    fn ensure_no_dots(&self, x: &Parameter) -> Result<(), LintError> {
        self.ensure_no_dots(x.as_bare_name())
    }
}

impl NoDotNamesCheck<DimVar, LintError> for DotsLinter {
    fn ensure_no_dots(&self, x: &DimVar) -> Result<(), LintError> {
        self.ensure_no_dots(x.as_bare_name())
    }
}

impl NoDotNamesCheck<Name, LintError> for DotsLinter {
    fn ensure_no_dots(&self, name: &Name) -> Result<(), LintError> {
        self.ensure_no_dots(name.as_bare_name())
    }
}

impl NoDotNamesCheck<BareName, LintError> for DotsLinter {
    fn ensure_no_dots(&self, x: &BareName) -> Result<(), LintError> {
        match x.prefix('.') {
            Some(part_before_dot) => {
                if self.user_defined_names.contains(&part_before_dot) {
                    Err(LintError::DotClash)
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }
}

impl NoDotNamesCheck<Positioned<&Expression>, LintErrorPos> for DotsLinter {
    fn ensure_no_dots(&self, x: &Positioned<&Expression>) -> Result<(), LintErrorPos> {
        let Positioned { element, pos } = x;
        self.ensure_no_dots(&(*element, *pos))
    }
}

impl NoDotNamesCheck<ExpressionPos, LintErrorPos> for DotsLinter {
    fn ensure_no_dots(&self, x: &ExpressionPos) -> Result<(), LintErrorPos> {
        let Positioned { element, pos } = x;
        self.ensure_no_dots(&(element, *pos))
    }
}

impl NoDotNamesCheck<(&Expression, Position), LintErrorPos> for DotsLinter {
    fn ensure_no_dots(&self, x: &(&Expression, Position)) -> Result<(), LintErrorPos> {
        let (element, pos) = x;
        match element {
            Expression::Variable(var_name, _) => self.ensure_no_dots(var_name).with_err_at(pos),
            Expression::ArrayElement(var_name, indices, _) => {
                self.ensure_no_dots(var_name).with_err_at(pos)?;
                self.ensure_no_dots(indices)
            }
            Expression::FunctionCall(name, args) => {
                self.ensure_no_dots(name).with_err_at(pos)?;
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
    fn visit_program(&mut self, p: &Program) -> Result<(), LintErrorPos> {
        let mut collector = UserDefinedNamesCollector::visitor();
        collector.visit(p)?;
        self.user_defined_names = collector.delegate().user_defined_names;
        self.visit_global_statements(p)
    }

    fn visit_function_implementation(
        &mut self,
        f: &FunctionImplementation,
    ) -> Result<(), LintErrorPos> {
        self.ensure_no_dots(f)?;
        self.visit_statements(&f.body)
    }

    fn visit_sub_implementation(&mut self, s: &SubImplementation) -> Result<(), LintErrorPos> {
        self.ensure_no_dots(s)?;
        self.visit_statements(&s.body)
    }

    fn visit_dim(&mut self, dim_list: &DimList) -> Result<(), LintErrorPos> {
        dim_list
            .variables
            .iter()
            .try_for_each(|dim_var_pos| self.ensure_no_dots(dim_var_pos))
    }

    fn visit_assignment(&mut self, a: &Assignment, name_pos: Position) -> Result<(), LintErrorPos> {
        let (name, v) = a.into();
        self.ensure_no_dots(&Positioned::new(name, name_pos))?;
        self.visit_expression(v)
    }

    fn visit_for_loop(&mut self, f: &ForLoop) -> Result<(), LintErrorPos> {
        // no need to test f.next_counter, as it is the same as variable_name if it exists
        self.ensure_no_dots(&f.variable_name)?;
        self.visit_expression(&f.lower_bound)?;
        self.visit_expression(&f.upper_bound)?;
        if let Some(s) = &f.step {
            self.visit_expression(s)?;
        }
        self.visit_statements(&f.statements)
    }

    fn visit_expression(&mut self, e: &ExpressionPos) -> Result<(), LintErrorPos> {
        self.ensure_no_dots(e)
    }
}

/// Collects names of parameters or variables that are of a user defined type.
#[derive(Default)]
struct UserDefinedNamesCollector {
    user_defined_names: HashSet<CaseInsensitiveString>,
}

no_op_visitor!(UserDefinedNamesCollector: DefType, FunctionDeclaration, SubDeclaration, UserDefinedType);
no_pos_visitor!(UserDefinedNamesCollector);

impl UserDefinedNamesCollector {
    pub fn visitor() -> impl Visitor<Program> + DelegateVisitor<Self> + SetPosition {
        DeepStatementVisitor::new(Self::default())
    }
}

impl<T> Visitor<SubprogramImplementation<T>> for UserDefinedNamesCollector {
    fn visit(&mut self, element: &SubprogramImplementation<T>) -> VisitResult {
        self.visit(&element.params)
    }
}

impl Visitor<Statement> for UserDefinedNamesCollector {
    fn visit(&mut self, element: &Statement) -> VisitResult {
        match element {
            Statement::Dim(dim_list) | Statement::Redim(dim_list) => {
                self.visit(&dim_list.variables)
            }
            _ => Ok(()),
        }
    }
}

impl<T> Visitor<TypedName<T>> for UserDefinedNamesCollector
where
    T: VarType,
{
    fn visit(&mut self, element: &TypedName<T>) -> VisitResult {
        if element.var_type().as_user_defined_recursively().is_some() {
            self.user_defined_names
                .insert(element.as_bare_name().clone());
        }
        Ok(())
    }
}
