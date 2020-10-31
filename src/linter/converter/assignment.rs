use crate::common::{AtLocation, CanCastTo, Locatable, QError, QErrorNode, ToLocatableError};
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
                        let qualified_name: QualifiedName =
                            dim_name.clone().try_into().with_err_at(pos)?;
                        implicit_variables.push(qualified_name.at(pos));
                    }
                    Ok((Expression::Variable(dim_name).at(pos), implicit_variables))
                }
            }
            crate::parser::Expression::FunctionCall(_, _) => {
                // TODO check if name is an array
                self.convert_and_collect_implicit_variables(name_expr.at(pos))
            }
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
