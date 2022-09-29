use crate::parser::name::bare_name_p;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::statement_separator::peek_eof_or_statement_separator;
use crate::parser::{Keyword, ResumeOption, Statement};

// RESUME
// RESUME NEXT
// RESUME label

pub fn statement_resume_p() -> impl OptParser<Output = Statement> {
    keyword(Keyword::Resume)
        .then_use(resume_option_p().or_syntax_error("Expected: label or NEXT or end-of-statement"))
        .map(Statement::Resume)
}

fn resume_option_p() -> impl OptParser<Output = ResumeOption> {
    Alt3::new(blank_resume(), resume_next(), resume_label())
}

fn blank_resume() -> impl OptParser<Output = ResumeOption> {
    peek_eof_or_statement_separator().map(|_| ResumeOption::Bare)
}

fn resume_next() -> impl OptParser<Output = ResumeOption> {
    keyword(Keyword::Next)
        .preceded_by_req_ws()
        .map(|_| ResumeOption::Next)
}

fn resume_label() -> impl OptParser<Output = ResumeOption> {
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
