use crate::common::{HasLocation, QError};
use crate::parser::name::bare_name_p;
use crate::parser::pc::{whitespace_p, BinaryParser, Parser, Reader, UnaryFnParser, UnaryParser};
use crate::parser::pc_specific::{keyword_p, PcSpecific};
use crate::parser::statement_separator::EofOrStatementSeparator;
use crate::parser::{Keyword, ResumeOption, Statement};

// RESUME
// RESUME NEXT
// RESUME label

pub fn statement_resume_p<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_p(Keyword::Resume)
        .and_demand(
            resume_option_p().or_syntax_error("Expected: label or NEXT or end-of-statement"),
        )
        .map(|(_, r)| Statement::Resume(r))
}

fn resume_option_p<R>() -> impl Parser<R, Output = ResumeOption>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    blank_resume().or(resume_next()).or(resume_label())
}

fn blank_resume<R>() -> impl Parser<R, Output = ResumeOption>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    EofOrStatementSeparator::<R>::new()
        .peek()
        .map(|_| ResumeOption::Bare)
}

fn resume_next<R>() -> impl Parser<R, Output = ResumeOption>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    whitespace_p()
        .and(keyword_p(Keyword::Next))
        .map(|_| ResumeOption::Next)
}

fn resume_label<R>() -> impl Parser<R, Output = ResumeOption>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    whitespace_p()
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
