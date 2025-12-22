#[cfg(debug_assertions)]
use crate::pc::parsers::Parser;
use crate::pc::{ParseResult, RcStringView};
use crate::{parser_declaration, ParseError};

#[allow(dead_code)]
pub trait Logging: Parser<RcStringView>
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
    P: Parser<RcStringView>,
    P::Output: std::fmt::Debug,
{
}

parser_declaration!(
    struct LoggingParser {
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

impl<P> Parser<RcStringView> for LoggingParser<P>
where
    P: Parser<RcStringView>,
    P::Output: std::fmt::Debug,
{
    type Output = P::Output;
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
