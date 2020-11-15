use super::converter::{Converter, ConverterImpl};
use crate::common::*;
use crate::linter::converter::converter::ConverterWithImplicitVariables;
use crate::parser::{DimName, QualifiedNameNode, Statement, StatementNode, StatementNodes};

// A statement can be expanded into multiple statements to convert implicitly
// declared variables into explicit.
// Example:
//      A = B + C
// becomes
//      DIM B
//      DIM C
//      DIM A
//      A = B + C

impl<'a> Converter<StatementNode, StatementNodes> for ConverterImpl<'a> {
    fn convert(&mut self, statement_node: StatementNode) -> Result<StatementNodes, QErrorNode> {
        let mut result: StatementNodes = vec![];
        let (converted_statement_node, implicit_vars) =
            self.convert_and_collect_implicit_variables(statement_node)?;
        for implicit_var in implicit_vars {
            let Locatable {
                element: q_name,
                pos,
            } = implicit_var;
            result.push(Statement::Dim(DimName::from(q_name).at(pos)).at(pos));
        }
        result.push(converted_statement_node);
        Ok(result)
    }
}

impl<'a> Converter<StatementNodes, StatementNodes> for ConverterImpl<'a> {
    fn convert(&mut self, statement_nodes: StatementNodes) -> Result<StatementNodes, QErrorNode> {
        let mut result: StatementNodes = vec![];
        for statement_node in statement_nodes {
            let (converted_statement_node, implicit_vars) =
                self.convert_and_collect_implicit_variables(statement_node)?;
            for implicit_var in implicit_vars {
                let Locatable {
                    element: q_name,
                    pos,
                } = implicit_var;
                result.push(Statement::Dim(DimName::from(q_name).at(pos)).at(pos));
            }
            result.push(converted_statement_node);
        }
        Ok(result)
    }
}

impl<'a> ConverterWithImplicitVariables<StatementNode, StatementNode> for ConverterImpl<'a> {
    fn convert_and_collect_implicit_variables(
        &mut self,
        statement_node: StatementNode,
    ) -> Result<(StatementNode, Vec<QualifiedNameNode>), QErrorNode> {
        let Locatable {
            element: statement,
            pos,
        } = statement_node;
        match statement {
            Statement::Comment(c) => Ok((Statement::Comment(c).at(pos), vec![])),
            Statement::Assignment(n, e) => self.assignment(n.at(pos), e),
            Statement::Const(n, e, _) => self
                .constant(n, e)
                .map(|statement| (statement.at(pos), vec![])),
            Statement::SubCall(n, args) => self.sub_call(n.at(pos), args),
            Statement::IfBlock(i) => self
                .convert_and_collect_implicit_variables(i)
                .map(|(i, implicit_vars)| (Statement::IfBlock(i).at(pos), implicit_vars)),
            Statement::SelectCase(s) => self
                .convert_and_collect_implicit_variables(s)
                .map(|(s, implicit_vars)| (Statement::SelectCase(s).at(pos), implicit_vars)),
            Statement::ForLoop(f) => self
                .convert_and_collect_implicit_variables(f)
                .map(|(f, implicit_vars)| (Statement::ForLoop(f).at(pos), implicit_vars)),
            Statement::While(c) => self
                .convert_and_collect_implicit_variables(c)
                .map(|(c, implicit_vars)| (Statement::While(c).at(pos), implicit_vars)),
            Statement::ErrorHandler(l) => Ok((Statement::ErrorHandler(l).at(pos), vec![])),
            Statement::Label(l) => Ok((Statement::Label(l).at(pos), vec![])),
            Statement::GoTo(l) => Ok((Statement::GoTo(l).at(pos), vec![])),
            Statement::Dim(dim_name_node) => self
                .convert_and_collect_implicit_variables(dim_name_node)
                .map(|(dim_name_node, implicit_vars_in_array_dimensions)| {
                    (
                        Statement::Dim(dim_name_node).at(pos),
                        implicit_vars_in_array_dimensions,
                    )
                }),
            Statement::Print(print_node) => self
                .convert_and_collect_implicit_variables(print_node)
                .map(|(p, implicit_vars)| (Statement::Print(p).at(pos), implicit_vars)),
            Statement::BuiltInSubCall(_, _) => panic!("parser should not have created this"),
        }
    }
}
