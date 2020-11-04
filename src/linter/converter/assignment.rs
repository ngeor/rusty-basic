use crate::common::{
    AtLocation, CanCastTo, Locatable, Location, QError, QErrorNode, ToLocatableError,
};
use crate::linter::converter::converter::{ConverterImpl, ConverterWithImplicitVariables};
use crate::linter::{DimName, DimType, Expression, ExpressionNode, Statement, StatementNode};
use crate::parser::{BareName, Name, QualifiedName, QualifiedNameNode, TypeQualifier};
use std::convert::TryInto;

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

    fn assignment_name_variable_name(
        &mut self,
        name: Name,
        pos: Location,
    ) -> Result<(ExpressionNode, Vec<QualifiedNameNode>), QErrorNode> {
        let bare_name: &BareName = name.as_ref();
        if self.context.is_function_context(bare_name) {
            self.assign_to_function(name)
                .map(|dim_name| (Expression::Variable(dim_name).at(pos), vec![]))
                .with_err_at(pos)
        } else if self.subs.contains_key(bare_name)
            // it is possible to have a param name shadowing a function name (but not a sub name...)
            || (!self.context.is_param(&name, &self.resolver) && self.functions.contains_key(bare_name))
            || self.context.contains_const(bare_name)
        {
            Err(QError::DuplicateDefinition).with_err_at(pos)
        } else {
            let (dim_name, missing) = self
                .context
                .resolve_name_in_assignment(&name, &self.resolver)
                .with_err_at(pos)?;
            let mut implicit_variables: Vec<QualifiedNameNode> = vec![];
            if missing {
                // dim_name must be resolved BuiltIn
                let qualified_name: QualifiedName = dim_name.clone().try_into().with_err_at(pos)?;
                implicit_variables.push(qualified_name.at(pos));
            }
            Ok((Expression::Variable(dim_name).at(pos), implicit_variables))
        }
    }

    fn assignment_name_property(
        &mut self,
        left_side: crate::parser::Expression,
        property_name: Name,
        pos: Location,
    ) -> Result<(ExpressionNode, Vec<QualifiedNameNode>), QErrorNode> {
        // A.B$
        // A.B.C
        // if A is a known user defined type, proceed
        // if A is known, error
        // if A is unknown, fold into A.B.C and add new implicit variable

        // A(1).Test.Toast -> only allowed if A exists and is array of user defined type

        match left_side {
            crate::parser::Expression::VariableName(left_side_name) => {
                match self
                    .context
                    .resolve_expression(&left_side_name, &self.resolver)
                    .with_err_at(pos)?
                {
                    Some(left_expr) => todo!(),
                    None => {
                        // The left_side_name is not known as a variable.
                        // Fold it back and register it as an implicit variable.
                        let folded_name = left_side_name + '.' + property_name;
                        let qualified_name = self
                            .context
                            .resolve_missing_name_in_assignment(&folded_name, &self.resolver)
                            .with_err_at(pos)?;
                        Ok((
                            Expression::Variable(qualified_name.clone().into()).at(pos),
                            vec![qualified_name.at(pos)],
                        ))
                    }
                }
            }
            crate::parser::Expression::FunctionCall(left_side_name, args) => todo!(),
            crate::parser::Expression::Property(new_left_side, new_property_name) => todo!(),
            _ => unimplemented!(),
        }
    }

    fn assign_to_function(&self, name: Name) -> Result<DimName, QError> {
        let function_type: TypeQualifier = self.demand_function_type(&name);
        if name.is_bare_or_of_type(function_type) {
            Ok(DimName::new(name.into(), DimType::BuiltIn(function_type)))
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
