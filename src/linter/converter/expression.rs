use crate::common::{
    AtLocation, HasLocation, Locatable, Location, QError, QErrorNode, ToLocatableError,
};
use crate::linter::converter::converter::{ConverterImpl, ConverterWithImplicitVariables};
use crate::linter::{Expression, ExpressionNode, ExpressionType, HasExpressionType};
use crate::parser;
use crate::parser::{QualifiedNameNode, TypeQualifier};

// Convert expression into an expression + a collection of implicitly declared variables

type ExprResult = Result<(ExpressionNode, Vec<QualifiedNameNode>), QErrorNode>;

impl<'a> ConverterWithImplicitVariables<crate::parser::ExpressionNode, ExpressionNode>
    for ConverterImpl<'a>
{
    fn convert_and_collect_implicit_variables(
        &mut self,
        expression_node: crate::parser::ExpressionNode,
    ) -> ExprResult {
        let Locatable { element, pos } = expression_node;
        match element {
            parser::Expression::SingleLiteral(f) => f.into_expr_result(pos),
            parser::Expression::DoubleLiteral(d) => d.into_expr_result(pos),
            parser::Expression::StringLiteral(s) => s.into_expr_result(pos),
            parser::Expression::IntegerLiteral(i) => i.into_expr_result(pos),
            parser::Expression::LongLiteral(l) => l.into_expr_result(pos),
            parser::Expression::Variable(var_name, _) => {
                var_name::into_expr_result(self, var_name, pos)
            }
            parser::Expression::Constant(_) => panic!("Constant is only a linter thing"),
            parser::Expression::FunctionCall(name_expr, args) => {
                function_call::into_expr_result(self, name_expr, args, pos)
            }
            parser::Expression::Property(box_left_side, property_name, _) => {
                property::into_expr_result(self, *box_left_side, property_name, pos)
            }
            parser::Expression::BinaryExpression(op, l, r, _) => {
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
                    | ExpressionType::Array(_)
                    | ExpressionType::FixedLengthString(_)
                    | ExpressionType::UserDefined(_)
                    | ExpressionType::Unresolved => {
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
            parser::Expression::ArrayElement(_, _, _) => unimplemented!(),
            parser::Expression::BuiltInFunctionCall(_, _) => unimplemented!(),
        }
    }
}

trait ToExprResult {
    fn into_expr_result(self, pos: Location) -> ExprResult;
}

impl ToExprResult for f32 {
    fn into_expr_result(self, pos: Location) -> ExprResult {
        Ok((Expression::SingleLiteral(self).at(pos), vec![]))
    }
}

impl ToExprResult for f64 {
    fn into_expr_result(self, pos: Location) -> ExprResult {
        Ok((Expression::DoubleLiteral(self).at(pos), vec![]))
    }
}

impl ToExprResult for String {
    fn into_expr_result(self, pos: Location) -> ExprResult {
        Ok((Expression::StringLiteral(self).at(pos), vec![]))
    }
}

impl ToExprResult for i32 {
    fn into_expr_result(self, pos: Location) -> ExprResult {
        Ok((Expression::IntegerLiteral(self).at(pos), vec![]))
    }
}

impl ToExprResult for i64 {
    fn into_expr_result(self, pos: Location) -> ExprResult {
        Ok((Expression::LongLiteral(self).at(pos), vec![]))
    }
}

/// Handles the conversion of `VariableName` expression
pub mod var_name {
    use crate::common::{AtLocation, Locatable, Location, QError, ToLocatableError};
    use crate::linter::converter::converter::ConverterImpl;
    use crate::linter::converter::expression::ExprResult;
    use crate::linter::Expression;
    use crate::parser::{BareName, Name, QualifiedName};

    pub fn into_expr_result(
        converter: &mut ConverterImpl,
        name: Name,
        pos: Location,
    ) -> ExprResult {
        match converter
            .context
            .resolve_expression(&name, &converter.resolver)
            .with_err_at(pos)?
        {
            Some(x) => Ok((x.at(pos), vec![])),
            None => match resolve_name_as_subprogram(converter, &name).with_err_at(pos)? {
                Some(x) => Ok((x.at(pos), vec![])),
                None => {
                    let q_name = converter
                        .context
                        .resolve_missing_name_in_assignment(&name, &converter.resolver)
                        .with_err_at(pos)?;
                    Ok((
                        Expression::from(q_name.clone()).at(pos),
                        vec![q_name.at(pos)],
                    ))
                }
            },
        }
    }

    fn resolve_name_as_subprogram(
        converter: &mut ConverterImpl,
        name: &Name,
    ) -> Result<Option<Expression>, QError> {
        let bare_name: &BareName = name.as_ref();
        if converter.subs.contains_key(bare_name) {
            // using the name of a sub as a variable expression
            Err(QError::DuplicateDefinition)
        } else if converter.functions.contains_key(bare_name) {
            // if the function expects arguments, argument count mismatch
            let Locatable {
                element: (f_type, f_args),
                ..
            } = converter.functions.get(bare_name).unwrap();
            if !f_args.is_empty() {
                Err(QError::ArgumentCountMismatch)
            } else if converter.context.is_function_context(bare_name) {
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
                        Name::new(b.clone(), Some(*f_type)),
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
                                Name::new(bare_name.clone(), Some(*f_type)),
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
}

/// Handles the conversion of `FunctionCall` expression
pub mod function_call {
    use crate::built_ins::BuiltInFunction;
    use crate::common::{AtLocation, Location, QError, ToLocatableError};
    use crate::linter::converter::converter::{ConverterImpl, ConverterWithImplicitVariables};
    use crate::linter::converter::expression::ExprResult;
    use crate::linter::type_resolver::TypeResolver;
    use crate::linter::Expression;
    use crate::parser;
    use crate::parser::Name;
    use std::convert::TryInto;

    pub fn into_expr_result(
        converter: &mut ConverterImpl,
        name_expr: Name,
        args: parser::ExpressionNodes,
        pos: Location,
    ) -> ExprResult {
        let n = name_expr.clone();
        let (converted_args, implicit_variables) =
            converter.convert_and_collect_implicit_variables(args)?;
        let opt_built_in: Option<BuiltInFunction> = (&n).try_into().with_err_at(pos)?;
        match opt_built_in {
            Some(b) => Ok((
                Expression::BuiltInFunctionCall(b, converted_args).at(pos),
                implicit_variables,
            )),
            None => {
                // is it a function or an array element?
                if converter.context.is_array(&n) {
                    // we can ignore `missing` as we already confirmed we know it is an array
                    let (var_name, expression_type, _) = converter
                        .context
                        .resolve_name_in_assignment(&n, &converter.resolver)
                        .with_err_at(pos)?;
                    if converted_args.is_empty() {
                        // entire array
                        Ok((
                            Expression::Variable(var_name, expression_type.new_array()).at(pos),
                            implicit_variables,
                        ))
                    } else {
                        // array element
                        Ok((
                            Expression::ArrayElement(var_name, converted_args, expression_type)
                                .at(pos),
                            implicit_variables,
                        ))
                    }
                } else {
                    // function
                    if converted_args.is_empty() {
                        Err(QError::syntax_error(
                            "Cannot have function call without arguments",
                        ))
                        .with_err_at(pos)
                    } else {
                        Ok((
                            Expression::FunctionCall(
                                converter.resolver.resolve_name(&n).into(),
                                converted_args,
                            )
                            .at(pos),
                            implicit_variables,
                        ))
                    }
                }
            }
        }
    }
}

/// Handles the conversion of `Property` expression
pub mod property {
    use super::var_name;
    use crate::common::{AtLocation, Locatable, Location, QError, ToLocatableError};
    use crate::linter::converter::converter::ConverterImpl;
    use crate::linter::converter::expression::ExprResult;
    use crate::linter::{Expression, ExpressionType, HasExpressionType};
    use crate::parser::{BareName, Name};

    pub fn into_expr_result(
        converter: &mut ConverterImpl,
        left_side: crate::parser::Expression,
        property_name: Name,
        pos: Location,
    ) -> ExprResult {
        // A.B$
        // A.B.C
        // if A is a known user defined type, proceed
        // if A is known, error
        // if A is unknown, fold into A.B.C and add new implicit variable

        // A(1).Test.Toast -> only allowed if A exists and is array of user defined type
        match left_side {
            crate::parser::Expression::Variable(left_side_name, _) => {
                match converter
                    .context
                    .resolve_expression(&left_side_name, &converter.resolver)
                    .with_err_at(pos)?
                {
                    Some(converted_left_expr) => match converted_left_expr {
                        Expression::Variable(converted_var_name, left_type) => {
                            if let ExpressionType::UserDefined(user_defined_type_name) =
                                left_type.clone()
                            {
                                resolve_property(
                                    converter,
                                    Expression::Variable(converted_var_name.clone(), left_type),
                                    &user_defined_type_name,
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
                        let folded_name = left_side_name
                            .try_concat_name(property_name)
                            .expect("Should be able to fold name");
                        var_name::into_expr_result(converter, folded_name, pos)
                    }
                }
            }
            crate::parser::Expression::FunctionCall(_left_side_name, _args) => todo!(),
            crate::parser::Expression::Property(new_boxed_left_side, new_property_name, _) => {
                let new_left_side = *new_boxed_left_side;
                let (
                    Locatable {
                        element: converted_expr,
                        ..
                    },
                    implicit_variables,
                ) = into_expr_result(converter, new_left_side, new_property_name, pos)?;
                if implicit_variables.is_empty() {
                    // the property was resolved without the need for an implicit variable
                    let resolved_expression_type = converted_expr.expression_type();
                    if let ExpressionType::UserDefined(user_defined_type_name) =
                        resolved_expression_type
                    {
                        resolve_property(
                            converter,
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
        converter: &mut ConverterImpl,
        base_expr: Expression,
        user_defined_type_name: &BareName,
        property_name: Name,
        pos: Location,
    ) -> ExprResult {
        match converter.user_defined_types.get(user_defined_type_name) {
            Some(user_defined_type) => {
                let element_type = user_defined_type
                    .demand_element_by_name(&property_name)
                    .with_err_at(pos)?;
                Ok((
                    Expression::Property(
                        Box::new(base_expr),
                        property_name.un_qualify(),
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
