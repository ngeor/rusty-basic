use super::*;
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn try_parse_function_implementation(&mut self) -> Result<Option<TopLevelToken>> {
        if self.buf_lexer.try_consume_word("FUNCTION")? {
            // function name
            self.buf_lexer.demand_whitespace()?;
            let name = self.demand_name_with_type_qualifier()?;
            // function parameters
            self.buf_lexer.skip_whitespace()?;
            let function_arguments: Vec<QName> = self.parse_declaration_parameters()?;
            self.buf_lexer.demand_eol_or_eof()?;
            let block = self.parse_block()?;
            self.buf_lexer.demand_specific_word("END")?;
            self.buf_lexer.demand_whitespace()?;
            self.buf_lexer.demand_specific_word("FUNCTION")?;
            self.buf_lexer.demand_eol_or_eof()?;

            Ok(Some(TopLevelToken::FunctionImplementation(
                name,
                function_arguments,
                block
            )))
        } else {
            Ok(None)
        }
    }
}
