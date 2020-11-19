use crate::common::{AtLocation, Locatable, QErrorNode};
use crate::linter::converter::converter::{Converter, ConverterImpl};
use crate::parser::{FunctionImplementation, SubImplementation, TopLevelToken, TopLevelTokenNode};

// Vec because:
// 1. we filter out DefType and UserDefinedType
// 2. a top level statement might expand into multiple due to implicit variables
impl<'a> Converter<TopLevelTokenNode, Vec<TopLevelTokenNode>> for ConverterImpl<'a> {
    fn convert(
        &mut self,
        top_level_token_node: TopLevelTokenNode,
    ) -> Result<Vec<TopLevelTokenNode>, QErrorNode> {
        let Locatable {
            element: top_level_token,
            pos,
        } = top_level_token_node;
        match top_level_token {
            TopLevelToken::DefType(d) => {
                self.resolver.borrow_mut().set(&d);
                Ok(vec![])
            }
            TopLevelToken::FunctionDeclaration(_, _) | TopLevelToken::SubDeclaration(_, _) => {
                Ok(vec![])
            }
            TopLevelToken::FunctionImplementation(FunctionImplementation {
                name,
                params,
                body,
            }) => self
                .convert_function_implementation(name, params, body)
                .map(|top_level_token| vec![top_level_token.at(pos)]),
            TopLevelToken::SubImplementation(SubImplementation { name, params, body }) => self
                .convert_sub_implementation(name, params, body)
                .map(|top_level_token| vec![top_level_token.at(pos)]),
            TopLevelToken::Statement(statement) => {
                let statement_node = statement.at(pos);
                self.convert(statement_node)
                    .map(|converted_statement_nodes| {
                        converted_statement_nodes
                            .into_iter()
                            .map(|Locatable { element, pos }| {
                                TopLevelToken::Statement(element).at(pos)
                            })
                            .collect()
                    })
            }
            TopLevelToken::UserDefinedType(_) => {
                // already handled by first pass
                Ok(vec![])
            }
        }
    }
}
