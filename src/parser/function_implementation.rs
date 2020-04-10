use super::*;

impl<T: BufRead> Parser<T> {
    pub fn try_parse_function_implementation(
        &mut self,
    ) -> Result<Option<TopLevelTokenNode>, LexerError> {
        let opt_pos = self.buf_lexer.try_consume_word("FUNCTION")?;
        if let Some(pos) = opt_pos {
            // function name
            self.buf_lexer.demand_whitespace()?;
            let name = self.demand_name_with_type_qualifier()?;
            // function parameters
            self.buf_lexer.skip_whitespace()?;
            let function_arguments: Vec<NameNode> = self.parse_declaration_parameters()?;
            self.buf_lexer.demand_eol_or_eof()?;
            let block = self.parse_block()?;
            self.buf_lexer.demand_specific_word("END")?;
            self.buf_lexer.demand_whitespace()?;
            self.buf_lexer.demand_specific_word("FUNCTION")?;
            self.buf_lexer.demand_eol_or_eof()?;

            Ok(Some(TopLevelTokenNode::FunctionImplementation(
                FunctionImplementationNode::new(name, function_arguments, block, pos),
            )))
        } else {
            Ok(None)
        }
    }
}
