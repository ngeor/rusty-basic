use crate::input::RcStringView;
use crate::pc::*;
use crate::specific::core::name::bare_name_with_dots;
use crate::specific::core::statement_separator::peek_eof_or_statement_separator;
use crate::specific::pc_specific::*;
use crate::specific::{Keyword, ResumeOption, Statement};

// RESUME
// RESUME NEXT
// RESUME label

pub fn statement_resume_p() -> impl Parser<RcStringView, Output = Statement> {
    keyword(Keyword::Resume)
        .and_keep_right(
            resume_option_p().or_syntax_error("Expected: label or NEXT or end-of-statement"),
        )
        .map(Statement::Resume)
}

fn resume_option_p() -> impl Parser<RcStringView, Output = ResumeOption> {
    OrParser::new(vec![
        Box::new(blank_resume()),
        Box::new(resume_next()),
        Box::new(resume_label()),
    ])
}

fn blank_resume() -> impl Parser<RcStringView, Output = ResumeOption> {
    peek_eof_or_statement_separator().map(|_| ResumeOption::Bare)
}

fn resume_next() -> impl Parser<RcStringView, Output = ResumeOption> {
    whitespace().and(keyword(Keyword::Next), |_, _| ResumeOption::Next)
}

fn resume_label() -> impl Parser<RcStringView, Output = ResumeOption> {
    whitespace().and(bare_name_with_dots(), |_, r| ResumeOption::Label(r))
}

#[cfg(test)]
mod tests {
    use crate::assert_parser_err;
    use crate::error::ParseError;

    #[test]
    fn resume_with_invalid_option() {
        assert_parser_err!(
            "RESUME FOR",
            ParseError::syntax_error("Expected: label or NEXT or end-of-statement")
        );
    }
}
