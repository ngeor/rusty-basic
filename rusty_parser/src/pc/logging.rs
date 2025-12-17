use crate::pc::parsers::Parser;
use crate::pc::tokenizers::Tokenizer;
use crate::pc::ParseResult;
use crate::{parser_declaration, ParseError};

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

impl<I: Tokenizer + 'static, P> Parser<I> for LoggingPC<P>
where
    P: Parser<I>,
    P::Output: std::fmt::Debug,
{
    type Output = P::Output;
    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
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
            ParseResult::Ok(value) => {
                println!(
                    "{}{} Success. value={:?}, current position {:?}, peek token {}",
                    indentation(),
                    self.tag,
                    value,
                    tokenizer.position(),
                    peek_token(tokenizer)
                );
                ParseResult::Ok(value)
            }
            ParseResult::Err(err) => {
                println!(
                    "{}{} Err {:?} current position {:?} peek token {}",
                    indentation(),
                    self.tag,
                    err,
                    tokenizer.position(),
                    peek_token(tokenizer)
                );
                ParseResult::Err(err)
            }
        }
    }
}

fn peek_token(tokenizer: &mut impl Tokenizer) -> String {
    match tokenizer.read() {
        Some(token) => {
            let result = format!("kind {} text {}", token.kind, token.text);
            tokenizer.unread();
            result
        }
        None => "[None]".to_string(),
    }
}
