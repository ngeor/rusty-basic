use crate::common::QError;
use crate::parser::base::parsers::{HasOutput, NonOptParser, Parser};
use crate::parser::base::tokenizers::Tokenizer;

pub struct LoggingPC<P>(P, String);

impl<P> HasOutput for LoggingPC<P>
where
    P: HasOutput,
{
    type Output = P::Output;
}

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
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        println!(
            "{}{} Parsing current position {} peek token {}",
            indentation(),
            self.1,
            tokenizer.position(),
            peek_token(tokenizer)
        );
        unsafe {
            INDENTATION_LEVEL += 1;
        }
        let result = self.0.parse(tokenizer);
        unsafe {
            INDENTATION_LEVEL -= 1;
        }
        match result {
            Ok(Some(value)) => {
                println!(
                    "{}{} Success current position {} peek token {}",
                    indentation(),
                    self.1,
                    tokenizer.position(),
                    peek_token(tokenizer)
                );
                Ok(Some(value))
            }
            Ok(None) => {
                println!(
                    "{}{} None current position {} peek token {}",
                    indentation(),
                    self.1,
                    tokenizer.position(),
                    peek_token(tokenizer)
                );
                Ok(None)
            }
            Err(err) => {
                println!(
                    "{}{} Err current position {} peek token {}",
                    indentation(),
                    self.1,
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
            "{}{} Parsing non-opt current position {} peek token {}",
            indentation(),
            self.1,
            tokenizer.position(),
            peek_token(tokenizer)
        );
        unsafe {
            INDENTATION_LEVEL += 1;
        }
        let result = self.0.parse_non_opt(tokenizer);
        unsafe {
            INDENTATION_LEVEL -= 1;
        }
        match result {
            Ok(value) => {
                println!(
                    "{}{} Success current position {} peek token {}",
                    indentation(),
                    self.1,
                    tokenizer.position(),
                    peek_token(tokenizer)
                );
                Ok(value)
            }
            Err(err) => {
                println!(
                    "{}{} Err current position {} peek token {}",
                    indentation(),
                    self.1,
                    tokenizer.position(),
                    peek_token(tokenizer)
                );
                Err(err)
            }
        }
    }
}

pub trait LoggingTrait
where
    Self: Sized,
{
    fn logging(self, name: &str) -> LoggingPC<Self>;
}

impl<P> LoggingTrait for P
where
    P: Sized,
{
    fn logging(self, name: &str) -> LoggingPC<Self> {
        LoggingPC(self, name.to_owned())
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
