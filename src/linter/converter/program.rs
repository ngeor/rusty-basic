use crate::common::QErrorNode;
use crate::linter::converter::converter::{Converter, ConverterImpl};
use crate::linter::{ProgramNode, TopLevelTokenNode};
use crate::parser;

impl<'a> Converter<parser::ProgramNode, ProgramNode> for ConverterImpl<'a> {
    fn convert(&mut self, program: parser::ProgramNode) -> Result<ProgramNode, QErrorNode> {
        let mut result: Vec<TopLevelTokenNode> = vec![];
        for top_level_token_node in program.into_iter() {
            let mut converted_top_level_token_nodes: Vec<TopLevelTokenNode> =
                self.convert(top_level_token_node)?;
            result.append(&mut converted_top_level_token_nodes);
        }
        Ok(result)
    }
}
