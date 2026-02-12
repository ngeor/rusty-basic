use rusty_pc::*;

use crate::core::name::bare_name_p;
use crate::core::statement_separator::peek_eof_or_statement_separator;
use crate::input::StringView;
use crate::pc_specific::*;
use crate::{Keyword, ParserError, ResumeOption, Statement};

// RESUME
// RESUME NEXT
// RESUME label

pub fn statement_resume_p() -> impl Parser<StringView, Output = Statement, Error = ParserError> {
    keyword(Keyword::Resume)
        .and_keep_right(resume_option_p().or_expected("label or NEXT or end-of-statement"))
        .map(Statement::Resume)
}

fn resume_option_p() -> impl Parser<StringView, Output = ResumeOption, Error = ParserError> {
    OrParser::new(vec![
        Box::new(blank_resume()),
        Box::new(resume_next()),
        Box::new(resume_label()),
    ])
}

fn blank_resume() -> impl Parser<StringView, Output = ResumeOption, Error = ParserError> {
    peek_eof_or_statement_separator().map(|_| ResumeOption::Bare)
}

fn resume_next() -> impl Parser<StringView, Output = ResumeOption, Error = ParserError> {
    whitespace_ignoring().and(keyword_ignoring(Keyword::Next), |_, _| ResumeOption::Next)
}

fn resume_label() -> impl Parser<StringView, Output = ResumeOption, Error = ParserError> {
    whitespace_ignoring().and(bare_name_p(), |_, r| ResumeOption::Label(r))
}

#[cfg(test)]
mod tests {
    use crate::assert_parser_err;

    #[test]
    fn resume_with_invalid_option() {
        assert_parser_err!("RESUME FOR", expected("label or NEXT or end-of-statement"));
    }
}
