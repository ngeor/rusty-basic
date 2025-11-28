use crate::name::bare_name_with_dots;
use crate::pc::*;
use crate::pc_specific::*;
use crate::statement_separator::peek_eof_or_statement_separator;
use crate::{Keyword, ResumeOption, Statement};

// RESUME
// RESUME NEXT
// RESUME label

pub fn statement_resume_p<I: Tokenizer + 'static>() -> impl Parser<I, Output = Statement> {
    keyword(Keyword::Resume)
        .then_demand(
            resume_option_p().or_syntax_error("Expected: label or NEXT or end-of-statement"),
        )
        .map(Statement::Resume)
}

fn resume_option_p<I: Tokenizer + 'static>() -> impl Parser<I, Output = ResumeOption> {
    OrParser::new(vec![
        Box::new(blank_resume()),
        Box::new(resume_next()),
        Box::new(resume_label()),
    ])
}

fn blank_resume<I: Tokenizer + 'static>() -> impl Parser<I, Output = ResumeOption> {
    peek_eof_or_statement_separator().map(|_| ResumeOption::Bare)
}

fn resume_next<I: Tokenizer + 'static>() -> impl Parser<I, Output = ResumeOption> {
    whitespace()
        .and(keyword(Keyword::Next))
        .map(|_| ResumeOption::Next)
}

fn resume_label<I: Tokenizer + 'static>() -> impl Parser<I, Output = ResumeOption> {
    whitespace()
        .and(bare_name_with_dots())
        .keep_right()
        .map(ResumeOption::Label)
}

#[cfg(test)]
mod tests {
    use crate::assert_parser_err;
    use crate::ParseError;

    #[test]
    fn resume_with_invalid_option() {
        assert_parser_err!(
            "RESUME FOR",
            ParseError::syntax_error("Expected: label or NEXT or end-of-statement")
        );
    }
}
