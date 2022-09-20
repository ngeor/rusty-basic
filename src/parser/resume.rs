use crate::parser::base::and_pc::AndDemandTrait;
use crate::parser::base::or_pc::OrTrait;
use crate::parser::base::parsers::{FnMapTrait, Parser};
use crate::parser::name::bare_name_p;
use crate::parser::specific::whitespace::WhitespaceTrait;
use crate::parser::specific::{keyword_p, OrSyntaxErrorTrait};
use crate::parser::statement_separator::peek_eof_or_statement_separator;
use crate::parser::{Keyword, ResumeOption, Statement};

// RESUME
// RESUME NEXT
// RESUME label

pub fn statement_resume_p() -> impl Parser<Output = Statement> {
    keyword_p(Keyword::Resume)
        .and_demand(
            resume_option_p().or_syntax_error("Expected: label or NEXT or end-of-statement"),
        )
        .fn_map(|(_, r)| Statement::Resume(r))
}

fn resume_option_p() -> impl Parser<Output = ResumeOption> {
    blank_resume().or(resume_next()).or(resume_label())
}

fn blank_resume() -> impl Parser<Output = ResumeOption> {
    peek_eof_or_statement_separator().fn_map(|_| ResumeOption::Bare)
}

fn resume_next() -> impl Parser<Output = ResumeOption> {
    keyword_p(Keyword::Next)
        .preceded_by_req_ws()
        .fn_map(|_| ResumeOption::Next)
}

fn resume_label() -> impl Parser<Output = ResumeOption> {
    bare_name_p()
        .preceded_by_req_ws()
        .fn_map(ResumeOption::Label)
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
