use crate::common::*;
use crate::linter::converter::conversion_traits::{
    OneToManyConverter, SameTypeConverterWithImplicits,
};
use crate::linter::converter::{ConverterImpl, ExprContext, R};
use crate::linter::{DimContext, NameContext};
use crate::parser::{
    BareName, DimName, ExitObject, Expression, Name, Statement, StatementNode, StatementNodes,
};

// A statement can be expanded into multiple statements to convert implicitly
// declared variables into explicit.
// Example:
//      A = B + C
// becomes
//      DIM B
//      DIM C
//      DIM A
//      A = B + C

impl<'a> OneToManyConverter<StatementNode> for ConverterImpl<'a> {
    fn convert_to_many(
        &mut self,
        statement_node: StatementNode,
    ) -> Result<StatementNodes, QErrorNode> {
        let mut result: StatementNodes = vec![];
        let (converted_statement_node, implicit_vars) =
            self.convert_same_type_with_implicits(statement_node)?;
        if let Statement::Const(_, _) = converted_statement_node.as_ref() {
            // filter out CONST statements, they've been registered into context as values
        } else {
            for implicit_var in implicit_vars {
                let Locatable {
                    element: q_name,
                    pos,
                } = implicit_var;
                result.push(Statement::Dim(DimName::from(q_name).into_list(pos)).at(pos));
            }
            result.push(converted_statement_node);
        }
        Ok(result)
    }
}

impl<'a> SameTypeConverterWithImplicits<StatementNode> for ConverterImpl<'a> {
    fn convert_same_type_with_implicits(
        &mut self,
        statement_node: StatementNode,
    ) -> R<StatementNode> {
        let Locatable {
            element: statement,
            pos,
        } = statement_node;
        match statement {
            Statement::Assignment(n, e) => self.assignment(n.at(pos), e),
            // CONST will be filtered out in the StatementNodes processor
            Statement::Const(n, e) => self.context.on_const(n, e).map(|_| (dummy_const(), vec![])),
            Statement::SubCall(n, args) => self.sub_call(n.at(pos), args),
            Statement::BuiltInSubCall(built_in_sub, args) => {
                let (converted_args, implicits) =
                    self.context.on_expressions(args, ExprContext::Parameter)?;
                Ok((
                    Statement::BuiltInSubCall(built_in_sub, converted_args).at(pos),
                    implicits,
                ))
            }
            Statement::IfBlock(i) => self
                .convert_same_type_with_implicits(i)
                .map(|(i, implicit_vars)| (Statement::IfBlock(i).at(pos), implicit_vars)),
            Statement::SelectCase(s) => self
                .convert_same_type_with_implicits(s)
                .map(|(s, implicit_vars)| (Statement::SelectCase(s).at(pos), implicit_vars)),
            Statement::ForLoop(f) => self
                .convert_same_type_with_implicits(f)
                .map(|(f, implicit_vars)| (Statement::ForLoop(f).at(pos), implicit_vars)),
            Statement::While(c) => self
                .convert_same_type_with_implicits(c)
                .map(|(c, implicit_vars)| (Statement::While(c).at(pos), implicit_vars)),
            Statement::DoLoop(do_loop_node) => self
                .convert_same_type_with_implicits(do_loop_node)
                .map(|(x, implicit_vars)| (Statement::DoLoop(x).at(pos), implicit_vars)),
            Statement::Return(opt_label) => {
                if opt_label.is_some() && self.context.is_in_subprogram() {
                    // cannot have RETURN with explicit label inside subprogram
                    Err(QError::IllegalInSubFunction).with_err_at(pos)
                } else {
                    Ok((Statement::Return(opt_label).at(pos), vec![]))
                }
            }
            Statement::Resume(resume_option) => {
                if self.context.is_in_subprogram() {
                    Err(QError::IllegalInSubFunction).with_err_at(pos)
                } else {
                    Ok((Statement::Resume(resume_option).at(pos), vec![]))
                }
            }
            Statement::Exit(exit_object) => match self.context.get_name_context() {
                NameContext::Global => {
                    Err(QError::syntax_error("Illegal outside of subprogram")).with_err_at(pos)
                }
                NameContext::Sub => {
                    if exit_object == ExitObject::Sub {
                        Ok((Statement::Exit(exit_object).at(pos), vec![]))
                    } else {
                        Err(QError::syntax_error("Illegal inside sub")).with_err_at(pos)
                    }
                }
                NameContext::Function => {
                    if exit_object == ExitObject::Function {
                        Ok((Statement::Exit(exit_object).at(pos), vec![]))
                    } else {
                        Err(QError::syntax_error("Illegal inside function")).with_err_at(pos)
                    }
                }
            },
            Statement::Dim(dim_list) => self.context.on_dim(dim_list, DimContext::Default).map(
                |(dim_list, implicit_vars_in_array_dimensions)| {
                    (
                        Statement::Dim(dim_list).at(pos),
                        implicit_vars_in_array_dimensions,
                    )
                },
            ),
            Statement::Redim(dim_list) => self.context.on_dim(dim_list, DimContext::Redim).map(
                |(dim_list, implicit_vars_in_array_dimensions)| {
                    (
                        Statement::Redim(dim_list).at(pos),
                        implicit_vars_in_array_dimensions,
                    )
                },
            ),
            Statement::Print(print_node) => self
                .convert_same_type_with_implicits(print_node)
                .map(|(p, implicit_vars)| (Statement::Print(p).at(pos), implicit_vars)),
            Statement::OnError(_)
            | Statement::Label(_)
            | Statement::GoTo(_)
            | Statement::GoSub(_)
            | Statement::Comment(_)
            | Statement::End
            | Statement::System => Ok((statement.at(pos), vec![])),
        }
    }
}

fn dummy_const() -> StatementNode {
    let pos = Location::start();
    Statement::Const(
        Name::Bare(BareName::new(String::new())).at(pos),
        Expression::IntegerLiteral(0).at(pos),
    )
    .at(pos)
}
