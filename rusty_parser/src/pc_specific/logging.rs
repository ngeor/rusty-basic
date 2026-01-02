#[cfg(debug_assertions)]
use rusty_pc::ParseResult;
use rusty_pc::Parser;

use crate::error::ParseError;
use crate::input::RcStringView;

#[allow(dead_code)]
pub trait Logging: Parser<RcStringView, Error = ParseError>
where
    Self: Sized,
    Self::Output: std::fmt::Debug,
{
    fn logging(self, tag: &str) -> impl Parser<RcStringView, Output = Self::Output> {
        LoggingParser::new(self, tag.to_owned())
    }
}

impl<P> Logging for P
where
    P: Parser<RcStringView, Error = ParseError>,
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

impl<P> Parser<RcStringView> for LoggingParser<P>
where
    P: Parser<RcStringView, Error = ParseError>,
    P::Output: std::fmt::Debug,
{
    type Output = P::Output;
    type Error = ParseError;

    fn parse(
        &self,
        tokenizer: RcStringView,
    ) -> ParseResult<RcStringView, Self::Output, ParseError> {
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
            Ok((input, value)) => {
                println!(
                    "{}{} Success. value={:?}, current position {:?}",
                    indentation(),
                    self.tag,
                    value,
                    input.position(),
                );
                Ok((input, value))
            }
            Err((fatal, i, err)) => {
                println!(
                    "{}{} Err fatal={} {:?}, current position {:?}",
                    indentation(),
                    self.tag,
                    fatal,
                    err,
                    i.position()
                );
                Err((fatal, i, err))
            }
        }
    }
}
