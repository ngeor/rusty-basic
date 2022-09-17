use crate::parser::base::parsers::{AndDemandTrait, AndTrait, Parser};
use crate::parser::name::bare_name_p;
use crate::parser::specific::{keyword_p, whitespace};
use crate::parser::statement_separator::EofOrStatementSeparator;
use crate::parser::{Keyword, ResumeOption, Statement};

// RESUME
// RESUME NEXT
// RESUME label

pub fn statement_resume_p() -> impl Parser<Output = Statement> {
    keyword_p(Keyword::Resume)
        .and_demand(
            resume_option_p().or_syntax_error("Expected: label or NEXT or end-of-statement"),
        )
        .map(|(_, r)| Statement::Resume(r))
}

fn resume_option_p() -> impl Parser<Output = ResumeOption> {
    blank_resume().or(resume_next()).or(resume_label())
}

fn blank_resume() -> impl Parser<Output = ResumeOption> {
    EofOrStatementSeparator::new()
        .peek()
        .map(|_| ResumeOption::Bare)
}

fn resume_next() -> impl Parser<Output = ResumeOption> {
    whitespace()
        .and(keyword_p(Keyword::Next))
        .map(|_| ResumeOption::Next)
}

fn resume_label() -> impl Parser<Output = ResumeOption> {
    whitespace()
        .and(bare_name_p())
        .map(|(_, r)| ResumeOption::Label(r))
}

#[cfg(test)]
mod tests {
    use crate::assert_parser_err;
    use crate::common::QError;

    #[test]
    fn resume_with_invalid_option() {
        assert_parser_err!(
            "RESUME FOR",
            QError::syntax_error("Expected: label or NEXT or end-of-statement")
        );
    }
}
