use crate::parser_declaration;
use crate::pc::parsers::Parser;
use crate::pc::tokenizers::Tokenizer;
use rusty_common::*;

parser_declaration!(
    #[allow(dead_code)]
    pub struct LoggingPC {
        tag: String,
    }
);

static mut INDENTATION_LEVEL: i32 = 0;

fn indentation() -> String {
    let mut s = String::new();
    unsafe {
        for _ in 0..INDENTATION_LEVEL {
            s.push('\t');
        }
    }
    s
}

impl<P> Parser for LoggingPC<P>
where
    P: Parser,
{
    type Output = P::Output;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        println!(
            "{}{} Parsing non-opt current position {:?} peek token {}",
            indentation(),
            self.tag,
            tokenizer.position(),
            peek_token(tokenizer)
        );
        unsafe {
            INDENTATION_LEVEL += 1;
        }
        let result = self.parser.parse(tokenizer);
        unsafe {
            INDENTATION_LEVEL -= 1;
        }
        match result {
            Ok(value) => {
                println!(
                    "{}{} Success current position {:?} peek token {}",
                    indentation(),
                    self.tag,
                    tokenizer.position(),
                    peek_token(tokenizer)
                );
                Ok(value)
            }
            Err(err) => {
                println!(
                    "{}{} Err current position {:?} peek token {}",
                    indentation(),
                    self.tag,
                    tokenizer.position(),
                    peek_token(tokenizer)
                );
                Err(err)
            }
        }
    }
}

fn peek_token(tokenizer: &mut impl Tokenizer) -> String {
    match tokenizer.read() {
        Ok(Some(token)) => {
            let result = format!("kind {} text {}", token.kind, token.text);
            tokenizer.unread(token);
            result
        }
        Ok(None) => "[None]".to_string(),
        Err(_) => "[ERR!]".to_string(),
    }
}
