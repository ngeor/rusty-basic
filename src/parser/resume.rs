use crate::parser::name::bare_name_p;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::statement_separator::peek_eof_or_statement_separator;
use crate::parser::{Keyword, ResumeOption, Statement};

// RESUME
// RESUME NEXT
// RESUME label

pub fn statement_resume_p() -> impl Parser<Output = Statement> {
    keyword(Keyword::Resume)
        .and_demand(
            resume_option_p().or_syntax_error("Expected: label or NEXT or end-of-statement"),
        )
        .map(|(_, r)| Statement::Resume(r))
}

fn resume_option_p() -> impl Parser<Output = ResumeOption> {
    Alt3::new(blank_resume(), resume_next(), resume_label())
}

fn blank_resume() -> impl Parser<Output = ResumeOption> {
    peek_eof_or_statement_separator().map(|_| ResumeOption::Bare)
}

fn resume_next() -> impl Parser<Output = ResumeOption> {
    keyword(Keyword::Next)
        .preceded_by_req_ws()
        .map(|_| ResumeOption::Next)
}

fn resume_label() -> impl Parser<Output = ResumeOption> {
    bare_name_p().preceded_by_req_ws().map(ResumeOption::Label)
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
