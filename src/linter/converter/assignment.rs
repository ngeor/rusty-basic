use crate::common::{
    AtLocation, CanCastTo, HasLocation, Locatable, Location, QError, QErrorNode, ToLocatableError,
};
use crate::linter::converter::converter::{ConverterImpl, ConverterWithImplicitVariables};
use crate::linter::{Expression, ExpressionNode, ExpressionType, Statement, StatementNode};
use crate::parser::{BareName, Name, QualifiedNameNode, TypeQualifier};

impl<'a> ConverterImpl<'a> {
    pub fn assignment(
        &mut self,
        name_expr_node: crate::parser::ExpressionNode,
        expression_node: crate::parser::ExpressionNode,
    ) -> Result<(StatementNode, Vec<QualifiedNameNode>), QErrorNode> {
        let (
            Locatable {
                element: dim_name,
                pos,
            },
            mut implicit_variables_left,
        ) = self.assignment_name(name_expr_node)?;
        let (converted_expr, mut implicit_variables_right) =
            self.convert_and_collect_implicit_variables(expression_node)?;
        if converted_expr.can_cast_to(&dim_name) {
            implicit_variables_left.append(&mut implicit_variables_right);
            Ok((
                Statement::Assignment(dim_name, converted_expr).at(pos),
                implicit_variables_left,
            ))
        } else {
            Err(QError::TypeMismatch).with_err_at(&converted_expr)
        }
    }

    pub fn assignment_name(
        &mut self,
        name_expr_node: crate::parser::ExpressionNode,
    ) -> Result<(ExpressionNode, Vec<QualifiedNameNode>), QErrorNode> {
        match self.assignment_subprogram(&name_expr_node)? {
            Some(func_assignment) => Ok((func_assignment, vec![])),
            _ => {
                let Locatable {
                    element: name_expr,
                    pos,
                } = name_expr_node;
                match name_expr {
                    crate::parser::Expression::VariableName(name) => {
                        self.assignment_name_variable_name(name, pos)
                    }
                    crate::parser::Expression::FunctionCall(_, _) => {
                        // TODO check if name is an array
                        self.convert_and_collect_implicit_variables(name_expr.at(pos))
                    }
                    crate::parser::Expression::Property(left_side, property_name) => {
                        self.assignment_name_property(*left_side, property_name, pos)
                    }
                    _ => unimplemented!(),
                }
            }
        }
    }

    fn assignment_subprogram(
        &mut self,
        name_expr_node: &crate::parser::ExpressionNode,
    ) -> Result<Option<ExpressionNode>, QErrorNode> {
        let pos = name_expr_node.pos();
        match name_expr_node.as_ref().clone().fold_name() {
            Some(fold_name) => {
                let bare_name: &BareName = fold_name.as_ref();
                if self.context.is_function_context(bare_name) {
                    self.assign_to_function(fold_name)
                        .map(|(var_name, func_expression_type)| {
                            Some(Expression::Variable(var_name, func_expression_type).at(pos))
                        })
                        .with_err_at(name_expr_node)
                } else if self.subs.contains_key(bare_name)
                    // it is possible to have a param name shadowing a function name (but not a sub name...)
                    || (!self.context.is_param(&fold_name, &self.resolver) && self.functions.contains_key(bare_name))
                    || self.context.contains_const(bare_name)
                {
                    Err(QError::DuplicateDefinition).with_err_at(pos)
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    fn assignment_name_variable_name(
        &mut self,
        name: Name,
        pos: Location,
    ) -> Result<(ExpressionNode, Vec<QualifiedNameNode>), QErrorNode> {
        let (var_name, expr_type, missing) = self
            .context
            .resolve_name_in_assignment(&name, &self.resolver)
            .with_err_at(pos)?;
        let mut implicit_variables: Vec<QualifiedNameNode> = vec![];
        if missing {
            // var_name must be Qualified because it was missing
            if let Name::Qualified(qualified_name) = var_name.clone() {
                implicit_variables.push(qualified_name.at(pos));
            } else {
                panic!("missing name was not qualified");
            }
        }
        Ok((
            Expression::Variable(var_name, expr_type).at(pos),
            implicit_variables,
        ))
    }

    fn assignment_name_property(
        &mut self,
        left_side: crate::parser::Expression,
        property_name: Name,
        pos: Location,
    ) -> Result<(ExpressionNode, Vec<QualifiedNameNode>), QErrorNode> {
        super::expression::property::into_expr_result(self, left_side, property_name, pos)
    }

    fn assign_to_function(&self, name: Name) -> Result<(Name, ExpressionType), QError> {
        let function_type: TypeQualifier = self.demand_function_type(&name);
        if name.is_bare_or_of_type(function_type) {
            Ok((
                name.qualify(function_type),
                ExpressionType::BuiltIn(function_type),
            ))
        } else {
            // trying to assign to the function with an explicit wrong type
            Err(QError::DuplicateDefinition)
        }
    }

    fn demand_function_type<S: AsRef<BareName>>(&self, function_name: S) -> TypeQualifier {
        let Locatable {
            element: (function_type, _),
            ..
        } = self
            .functions
            .get(function_name.as_ref())
            .expect("Function not found");
        *function_type
    }
}
