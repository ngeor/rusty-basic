use crate::common::QErrorNode;
use crate::linter::converter::{Converter, ConverterImpl};
use crate::parser::{ProgramNode, TopLevelTokenNode};

impl<'a> Converter<ProgramNode, ProgramNode> for ConverterImpl<'a> {
    fn convert(&mut self, program: ProgramNode) -> Result<ProgramNode, QErrorNode> {
        let mut result: Vec<TopLevelTokenNode> = vec![];
        for top_level_token_node in program.into_iter() {
            let mut converted_top_level_token_nodes: Vec<TopLevelTokenNode> =
                self.convert(top_level_token_node)?;
            result.append(&mut converted_top_level_token_nodes);
        }
        Ok(result)
    }
}
