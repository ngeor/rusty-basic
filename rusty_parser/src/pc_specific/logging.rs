#[cfg(debug_assertions)]
use rusty_pc::{Parser, SetContext};

use crate::error::ParserError;
use crate::input::StringView;

#[allow(dead_code)]
pub trait Logging: Parser<StringView, Error = ParserError>
where
    Self: Sized,
    Self::Output: std::fmt::Debug,
{
    fn logging(self, tag: &str) -> impl Parser<StringView, Output = Self::Output> {
        LoggingParser::new(self, tag.to_owned())
    }
}

impl<P> Logging for P
where
    P: Parser<StringView, Error = ParserError>,
    P::Output: std::fmt::Debug,
{
}

struct LoggingParser<P> {
    parser: P,
    tag: String,
}
impl<P> LoggingParser<P> {
    pub fn new(parser: P, tag: String) -> Self {
        Self { parser, tag }
    }
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

impl<P> Parser<StringView> for LoggingParser<P>
where
    P: Parser<StringView, Error = ParserError>,
    P::Output: std::fmt::Debug,
{
    type Output = P::Output;
    type Error = ParserError;

    fn parse(&mut self, tokenizer: &mut StringView) -> Result<Self::Output, ParserError> {
        println!(
            "{}{} Parsing at position {:?}",
            indentation(),
            self.tag,
            tokenizer.position(),
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
                    "{}{} Success. value={:?}, current position {:?}",
                    indentation(),
                    self.tag,
                    value,
                    tokenizer.position(),
                );
                Ok(value)
            }
            Err(err) => {
                println!(
                    "{}{} Err {:?}, current position {:?}",
                    indentation(),
                    self.tag,
                    err,
                    tokenizer.position()
                );
                Err(err)
            }
        }
    }
}

impl<P> SetContext<()> for LoggingParser<P>
where
    P: SetContext<()>,
{
    fn set_context(&mut self, _ctx: ()) {
        self.parser.set_context(_ctx);
    }
}
