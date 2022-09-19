use crate::common::QError;
use crate::parser::base::parsers::{HasOutput, Parser};
use crate::parser::base::tokenizers::Tokenizer;

pub struct LoggingPC<P>(P, String);

impl<P> HasOutput for LoggingPC<P> where P : HasOutput {
    type Output = P::Output;
}

impl<P> Parser for LoggingPC<P> where P : Parser {
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        println!("{} Parsing current position {} peek token {}", self.1, tokenizer.position(), peek_token(tokenizer));
        match self.0.parse(tokenizer) {
            Ok(Some(value)) => {
                println!("{} Success current position {} peek token {}", self.1, tokenizer.position(), peek_token(tokenizer));
                Ok(Some(value))
            }
            Ok(None) => {
                println!("{} None current position {} peek token {}", self.1, tokenizer.position(), peek_token(tokenizer));
                Ok(None)
            }
            Err(err) => {
                println!("{} Err current position {} peek token {}", self.1, tokenizer.position(), peek_token(tokenizer));
                Err(err)
            }
        }
    }
}

pub trait LoggingTrait where Self : Sized {
    fn logging(self, name: &str) -> LoggingPC<Self>;
}

impl<P> LoggingTrait for P where P : Sized {
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
