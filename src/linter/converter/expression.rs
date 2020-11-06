use crate::built_ins::BuiltInFunction;
use crate::common::{
    AtLocation, HasLocation, Locatable, Location, QError, QErrorNode, ToLocatableError,
};
use crate::linter::converter::converter::{ConverterImpl, ConverterWithImplicitVariables};
use crate::linter::type_resolver::TypeResolver;
use crate::linter::{DimType, Expression, ExpressionNode, ExpressionType, HasExpressionType};
use crate::parser;
use crate::parser::{BareName, Name, NameNode, QualifiedName, QualifiedNameNode, TypeQualifier};
use std::convert::TryInto;

// Convert expression into an expression + a collection of implicitly declared variables

impl<'a> ConverterWithImplicitVariables<crate::parser::ExpressionNode, ExpressionNode>
    for ConverterImpl<'a>
{
    fn convert_and_collect_implicit_variables(
        &mut self,
        expression_node: crate::parser::ExpressionNode,
    ) -> Result<(ExpressionNode, Vec<QualifiedNameNode>), QErrorNode> {
        let Locatable { element, pos } = expression_node;

        match element {
            parser::Expression::SingleLiteral(f) => {
                Ok((Expression::SingleLiteral(f).at(pos), vec![]))
            }
            parser::Expression::DoubleLiteral(f) => {
                Ok((Expression::DoubleLiteral(f).at(pos), vec![]))
            }
            parser::Expression::StringLiteral(f) => {
                Ok((Expression::StringLiteral(f).at(pos), vec![]))
            }
            parser::Expression::IntegerLiteral(f) => {
                Ok((Expression::IntegerLiteral(f).at(pos), vec![]))
            }
            parser::Expression::LongLiteral(f) => Ok((Expression::LongLiteral(f).at(pos), vec![])),
            parser::Expression::VariableName(var_name) => self
                .resolve_name_in_expression(&var_name.at(pos))
                .with_err_at(pos),
            parser::Expression::FunctionCall(name_expr, args) => {
                let n = name_expr.clone();
                let (converted_args, implicit_variables) =
                    self.convert_and_collect_implicit_variables(args)?;
                let opt_built_in: Option<BuiltInFunction> = (&n).try_into().with_err_at(pos)?;
                match opt_built_in {
                    Some(b) => Ok((
                        Expression::BuiltInFunctionCall(b, converted_args).at(pos),
                        implicit_variables,
                    )),
                    None => {
                        // is it a function or an array element?
                        if self.context.is_array(&n) {
                            // we can ignore `missing` as we already confirmed we know it is an array
                            let (dim_name, _) = self
                                .context
                                .resolve_name_in_assignment(&n, &self.resolver)
                                .with_err_at(pos)?;
                            let element_type = dim_name.expression_type();
                            let array_name: Name = dim_name.into();
                            Ok((
                                Expression::ArrayElement(array_name, converted_args, element_type)
                                    .at(pos),
                                implicit_variables,
                            ))
                        } else {
                            Ok((
                                Expression::FunctionCall(
                                    self.resolver.resolve_name(&n),
                                    converted_args,
                                )
                                .at(pos),
                                implicit_variables,
                            ))
                        }
                    }
                }
            }
            parser::Expression::Property(box_left_side, property_name) => {
                self.convert_property(*box_left_side, property_name, pos)
            }
            parser::Expression::BinaryExpression(op, l, r) => {
                // unbox them
                let unboxed_left = *l;
                let unboxed_right = *r;
                // convert them
                let (converted_left, mut implicit_variables_left) =
                    self.convert_and_collect_implicit_variables(unboxed_left)?;
                let (converted_right, mut implicit_variables_right) =
                    self.convert_and_collect_implicit_variables(unboxed_right)?;
                implicit_variables_left.append(&mut implicit_variables_right);
                // cast
                Expression::binary(converted_left, converted_right, op)
                    .map(|bin_expr| (bin_expr.at(pos), implicit_variables_left))
            }
            parser::Expression::UnaryExpression(op, c) => {
                let (converted_child, implicit_variables) =
                    self.convert_and_collect_implicit_variables(c)?;
                match converted_child.expression_type() {
                    ExpressionType::BuiltIn(TypeQualifier::DollarString)
                    | ExpressionType::FixedLengthString(_)
                    | ExpressionType::UserDefined(_) => {
                        Err(QError::TypeMismatch).with_err_at(converted_child.pos())
                    }
                    ExpressionType::BuiltIn(_) => Ok((
                        Expression::UnaryExpression(op, converted_child).at(pos),
                        implicit_variables,
                    )),
                }
            }
            parser::Expression::Parenthesis(c) => {
                let (converted_child, implicit_variables) =
                    self.convert_and_collect_implicit_variables(c)?;
                Ok((
                    Expression::Parenthesis(converted_child).at(pos),
                    implicit_variables,
                ))
            }
        }
    }
}

impl<'a> ConverterImpl<'a> {
    fn resolve_name_in_expression(
        &mut self,
        n: &NameNode,
    ) -> Result<(ExpressionNode, Vec<QualifiedNameNode>), QError> {
        let Locatable { element: name, pos } = n;
        match self.context.resolve_expression(name, &self.resolver)? {
            Some(x) => Ok((x.at(pos), vec![])),
            None => match self.resolve_name_as_subprogram(name)? {
                Some(x) => Ok((x.at(pos), vec![])),
                None => {
                    let q_name = self
                        .context
                        .resolve_missing_name_in_assignment(name, &self.resolver)?;
                    Ok((
                        Expression::Variable(q_name.clone().into()).at(pos),
                        vec![q_name.at(pos)],
                    ))
                }
            },
        }
    }

    fn resolve_name_as_subprogram(&mut self, name: &Name) -> Result<Option<Expression>, QError> {
        let bare_name: &BareName = name.as_ref();
        if self.subs.contains_key(bare_name) {
            // using the name of a sub as a variable expression
            Err(QError::DuplicateDefinition)
        } else if self.functions.contains_key(bare_name) {
            // if the function expects arguments, argument count mismatch
            let Locatable {
                element: (f_type, f_args),
                ..
            } = self.functions.get(bare_name).unwrap();
            if !f_args.is_empty() {
                Err(QError::ArgumentCountMismatch)
            } else if self.context.is_function_context(bare_name) {
                // We are inside a function that takes no args, and we're using again
                // the name of that function as an expression.
                // This can only work as a variable, otherwise we'll get infinite recursive call.
                //
                // Example:
                // Function Test
                //     INPUT Test
                // End Function
                //
                // Return None and let the next handler add it as a new variable
                Ok(None)
            } else {
                match name {
                    Name::Bare(b) => Ok(Some(Expression::FunctionCall(
                        QualifiedName::new(b.clone(), *f_type),
                        vec![],
                    ))),
                    Name::Qualified(QualifiedName {
                        bare_name,
                        qualifier,
                    }) => {
                        // if the function is a different type and the name is qualified of a different type, duplication definition
                        if f_type != qualifier {
                            Err(QError::DuplicateDefinition)
                        } else {
                            Ok(Some(Expression::FunctionCall(
                                QualifiedName::new(bare_name.clone(), *f_type),
                                vec![],
                            )))
                        }
                    }
                }
            }
        } else {
            Ok(None)
        }
    }

    pub fn convert_property(
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
                    Some(converted_left_expr) => match converted_left_expr {
                        Expression::Variable(converted_var_name) => {
                            if let DimType::UserDefined(user_defined_type_name) =
                                converted_var_name.dim_type()
                            {
                                self.resolve_property(
                                    Expression::Variable(converted_var_name.clone()),
                                    user_defined_type_name,
                                    property_name,
                                    pos,
                                )
                            } else {
                                todo!()
                            }
                        }
                        _ => todo!(),
                    },
                    None => {
                        // The left_side_name is not known as a variable.
                        // Fold it back and register it as an implicit variable.
                        let folded_name = left_side_name + '.' + property_name;
                        self.context.resolve_expression_or_add_implicit_variable(
                            &folded_name,
                            &self.resolver,
                            pos,
                        )
                    }
                }
            }
            crate::parser::Expression::FunctionCall(_left_side_name, _args) => todo!(),
            crate::parser::Expression::Property(new_boxed_left_side, new_property_name) => {
                let new_left_side = *new_boxed_left_side;
                let (
                    Locatable {
                        element: converted_expr,
                        ..
                    },
                    implicit_variables,
                ) = self.convert_property(new_left_side, new_property_name, pos)?;
                if implicit_variables.is_empty() {
                    // the property was resolved without the need for an implicit variable
                    let resolved_expression_type = converted_expr.expression_type();
                    if let ExpressionType::UserDefined(user_defined_type_name) =
                        resolved_expression_type
                    {
                        self.resolve_property(
                            converted_expr,
                            &user_defined_type_name,
                            property_name,
                            pos,
                        )
                    } else {
                        Err(QError::ElementNotDefined).with_err_at(pos)
                    }
                } else {
                    // implicit variables, probably we had to fold back to a single variable
                    todo!()
                }
            }
            _ => unimplemented!(),
        }
    }

    fn resolve_property(
        &self,
        base_expr: Expression,
        user_defined_type_name: &BareName,
        property_name: Name,
        pos: Location,
    ) -> Result<(ExpressionNode, Vec<QualifiedNameNode>), QErrorNode> {
        match self.user_defined_types.get(user_defined_type_name) {
            Some(user_defined_type) => {
                let element_type = user_defined_type
                    .demand_element_by_name(&property_name)
                    .with_err_at(pos)?;
                Ok((
                    Expression::Property(
                        Box::new(base_expr),
                        property_name.into(),
                        element_type.clone().expression_type(),
                    )
                    .at(pos),
                    vec![],
                ))
            }
            None => todo!(),
        }
    }
}
