use crate::common::QError;
use crate::parser::pc::parsers::{NonOptParser, OptParser, ParserBase};
use crate::parser::pc::tokenizers::Tokenizer;
use crate::parser_decorator;

parser_decorator!(
    struct LoggingPC {
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

impl<P> OptParser for LoggingPC<P>
where
    P: OptParser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        println!(
            "{}{} Parsing current position {:?} peek token {}",
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
            Ok(Some(value)) => {
                println!(
                    "{}{} Success current position {:?} peek token {}",
                    indentation(),
                    self.tag,
                    tokenizer.position(),
                    peek_token(tokenizer)
                );
                Ok(Some(value))
            }
            Ok(None) => {
                println!(
                    "{}{} None current position {:?} peek token {}",
                    indentation(),
                    self.tag,
                    tokenizer.position(),
                    peek_token(tokenizer)
                );
                Ok(None)
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

impl<P> NonOptParser for LoggingPC<P>
where
    P: NonOptParser,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
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
        let result = self.parser.parse_non_opt(tokenizer);
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
        Ok(None) => {
            format!("[None]")
        }
        Err(_) => {
            format!("[ERR!]")
        }
    }
}
