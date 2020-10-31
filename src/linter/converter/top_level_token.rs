use crate::common::{AtLocation, Locatable, QErrorNode};
use crate::linter::converter::converter::{Converter, ConverterImpl};
use crate::linter::{TopLevelToken, TopLevelTokenNode};
use crate::parser;

// Vec because:
// 1. we filter out DefType and UserDefinedType
// 2. a top level statement might expand into multiple due to implicit variables
impl<'a> Converter<parser::TopLevelTokenNode, Vec<TopLevelTokenNode>> for ConverterImpl<'a> {
    fn convert(
        &mut self,
        top_level_token_node: parser::TopLevelTokenNode,
    ) -> Result<Vec<TopLevelTokenNode>, QErrorNode> {
        let Locatable {
            element: top_level_token,
            pos,
        } = top_level_token_node;
        match top_level_token {
            parser::TopLevelToken::DefType(d) => {
                self.resolver.set(&d);
                Ok(vec![])
            }
            parser::TopLevelToken::FunctionDeclaration(_, _)
            | parser::TopLevelToken::SubDeclaration(_, _) => Ok(vec![]),
            parser::TopLevelToken::FunctionImplementation(n, params, block) => self
                .convert_function_implementation(n, params, block)
                .map(|top_level_token| vec![top_level_token.at(pos)]),
            parser::TopLevelToken::SubImplementation(n, params, block) => self
                .convert_sub_implementation(n, params, block)
                .map(|top_level_token| vec![top_level_token.at(pos)]),
            parser::TopLevelToken::Statement(statement) => {
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
            parser::TopLevelToken::UserDefinedType(_) => {
                // already handled by first pass
                Ok(vec![])
            }
        }
    }
}
