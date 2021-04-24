use crate::common::{AtLocation, Locatable, QErrorNode};
use crate::linter::converter::conversion_traits::OneToManyConverter;
use crate::linter::converter::ConverterImpl;
use crate::parser::{TopLevelToken, TopLevelTokenNode};

// Vec because:
// 1. we filter out DefType and UserDefinedType
// 2. a top level statement might expand into multiple due to implicit variables
impl<'a> OneToManyConverter<TopLevelTokenNode> for ConverterImpl<'a> {
    fn convert_to_many(
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
            TopLevelToken::FunctionImplementation(function_implementation) => self
                .convert_function_implementation(function_implementation)
                .map(|top_level_token| vec![top_level_token.at(pos)]),
            TopLevelToken::SubImplementation(sub_implementation) => self
                .convert_sub_implementation(sub_implementation)
                .map(|top_level_token| vec![top_level_token.at(pos)]),
            TopLevelToken::Statement(statement) => {
                let statement_node = statement.at(pos);
                self.convert_to_many(statement_node)
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
