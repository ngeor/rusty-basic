use crate::common::{AtLocation, Locatable, PatchErrPos, QErrorNode};
use crate::linter::converter::converter::{Converter, ConverterImpl};
use crate::linter::{ProgramNode, TopLevelToken, TopLevelTokenNode};
use crate::parser;

impl<'a> Converter<parser::ProgramNode, ProgramNode> for ConverterImpl<'a> {
    fn convert(&mut self, a: parser::ProgramNode) -> Result<ProgramNode, QErrorNode> {
        let mut result: Vec<TopLevelTokenNode> = vec![];
        for top_level_token_node in a.into_iter() {
            // will contain None where DefInt and declarations used to be
            let Locatable { element, pos } = top_level_token_node;
            let opt: Option<TopLevelToken> = self.convert(element).patch_err_pos(pos)?;
            match opt {
                Some(t) => {
                    let r: TopLevelTokenNode = t.at(pos);
                    result.push(r);
                }
                _ => (),
            }
        }
        Ok(result)
    }
}
