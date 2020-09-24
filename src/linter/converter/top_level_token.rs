use crate::common::QErrorNode;
use crate::linter::converter::converter::{Converter, ConverterImpl};
use crate::linter::TopLevelToken;
use crate::parser;

// Option because we filter out DefType and UserDefinedType
impl<'a> Converter<parser::TopLevelToken, Option<TopLevelToken>> for ConverterImpl<'a> {
    fn convert(&mut self, a: parser::TopLevelToken) -> Result<Option<TopLevelToken>, QErrorNode> {
        match a {
            parser::TopLevelToken::DefType(d) => {
                self.resolver.set(&d);
                Ok(None)
            }
            parser::TopLevelToken::FunctionDeclaration(_, _)
            | parser::TopLevelToken::SubDeclaration(_, _) => Ok(None),
            parser::TopLevelToken::FunctionImplementation(n, params, block) => {
                self.convert_function_implementation(n, params, block)
            }
            parser::TopLevelToken::SubImplementation(n, params, block) => {
                self.convert_sub_implementation(n, params, block)
            }
            parser::TopLevelToken::Statement(s) => {
                Ok(Some(TopLevelToken::Statement(self.convert(s)?)))
            }
            parser::TopLevelToken::UserDefinedType(_) => {
                // already handled by first pass
                Ok(None)
            }
        }
    }
}
