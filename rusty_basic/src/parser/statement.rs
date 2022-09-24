use crate::common::*;
use crate::parser::base::and_pc::{AndDemandTrait, AndTrait};
use crate::parser::base::and_then_pc::AndThenTrait;
use crate::parser::base::or_pc::{alt2, alt3, alt4, OrTrait};
use crate::parser::base::parsers::{FnMapTrait, KeepLeftTrait, Parser};
use crate::parser::comment;
use crate::parser::constant;
use crate::parser::dim;
use crate::parser::do_loop;
use crate::parser::exit::statement_exit_p;
use crate::parser::for_loop;
use crate::parser::go_sub::{statement_go_sub_p, statement_return_p};
use crate::parser::if_block;
use crate::parser::name::{bare_name_as_token, bare_name_p};
use crate::parser::on_error::statement_on_error_go_to_p;
use crate::parser::print;
use crate::parser::resume::statement_resume_p;
use crate::parser::select_case;
use crate::parser::specific::keyword_choice::keyword_choice;
use crate::parser::specific::{item_p, keyword_followed_by_whitespace_p, OrErrorTrait};
use crate::parser::sub_call;
use crate::parser::types::*;
use crate::parser::while_wend;

pub fn statement_p() -> impl Parser<Output = Statement> {
    alt3(
        alt3(
            statement_label_p(),
            single_line_statement_p(),
            if_block::if_block_p(),
        ),
        alt3(
            for_loop::for_loop_p(),
            select_case::select_case_p(),
            while_wend::while_wend_p(),
        ),
        alt2(do_loop::do_loop_p(), illegal_starting_keywords()),
    )
}

/// Tries to read a statement that is allowed to be on a single line IF statement,
/// excluding comments.
pub fn single_line_non_comment_statement_p() -> impl Parser<Output = Statement> {
    alt4(
        alt4(
            dim::dim_p(),
            dim::redim_p(),
            constant::constant_p(),
            crate::built_ins::parser::parse(),
        ),
        alt4(
            print::parse_print_p(),
            print::parse_lprint_p(),
            sub_call::sub_call_or_assignment_p(),
            statement_go_to_p(),
        ),
        alt4(
            statement_go_sub_p(),
            statement_return_p(),
            statement_exit_p(),
            statement_on_error_go_to_p(),
        ),
        alt3(
            statement_resume_p(),
            end::parse_end_p(),
            system::parse_system_p(),
        ),
    )
}

/// Tries to read a statement that is allowed to be on a single line IF statement,
/// including comments.
pub fn single_line_statement_p() -> impl Parser<Output = Statement> {
    comment::comment_p().or(single_line_non_comment_statement_p())
}

fn statement_label_p() -> impl Parser<Output = Statement> {
    // labels can have dots
    bare_name_as_token()
        .and(item_p(':'))
        .keep_left()
        .fn_map(|l| Statement::Label(l.text.into()))
}

fn statement_go_to_p() -> impl Parser<Output = Statement> {
    keyword_followed_by_whitespace_p(Keyword::GoTo)
        .and_demand(bare_name_p().or_syntax_error("Expected: label"))
        .fn_map(|(_, l)| Statement::GoTo(l))
}

fn illegal_starting_keywords() -> impl Parser<Output = Statement> {
    keyword_choice(&[Keyword::Wend, Keyword::Else, Keyword::Loop]).and_then(|(k, _)| match k {
        Keyword::Wend => Err(QError::WendWithoutWhile),
        Keyword::Else => Err(QError::ElseWithoutIf),
        Keyword::Loop => Err(QError::syntax_error("LOOP without DO")),
        _ => panic!("Parser should not have parsed {}", k),
    })
}

mod end {
    use tokenizers::{Token, Tokenizer};
    use crate::parser::base::parsers::{FnMapTrait, HasOutput, NonOptParser};
    use crate::parser::base::undo_pc::Undo;
    use crate::parser::specific::keyword;
    use crate::parser::specific::keyword_choice::keyword_choice;
    use crate::parser::specific::whitespace::whitespace;
    use crate::parser::statement_separator::peek_eof_or_statement_separator;

    use super::*;

    pub fn parse_end_p() -> impl Parser<Output = Statement> {
        keyword(Keyword::End)
            .and_demand(AfterEndSeparator)
            .fn_map(|_| Statement::End)
    }

    /// Parses the next token after END. If it is one of the valid keywords that
    /// can follow END, it is undone so that the entire parsing will be undone.
    /// Otherwise, it demands that we find an end-of-statement terminator.
    struct AfterEndSeparator;

    impl HasOutput for AfterEndSeparator {
        type Output = ();
    }

    impl NonOptParser for AfterEndSeparator {
        fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
            let opt_ws = whitespace().parse(tokenizer)?;
            if opt_ws.is_some() {
                // maybe it is followed by a legit keyword after END
                let opt_k = allowed_keywords_after_end().parse(tokenizer)?;
                if opt_k.is_some() {
                    // got it
                    tokenizer.unread(opt_k.unwrap().1);
                    tokenizer.unread(opt_ws.unwrap());
                    return Ok(());
                }
            }

            // is the next token eof or end of statement?
            let opt_sep = peek_eof_or_statement_separator().parse(tokenizer)?;

            // put back the ws if we read it
            opt_ws.undo(tokenizer);

            opt_sep.ok_or(QError::syntax_error(
                "Expected: DEF or FUNCTION or IF or SELECT or SUB or TYPE or end-of-statement",
            ))
        }
    }

    fn allowed_keywords_after_end() -> impl Parser<Output = (Keyword, Token)> {
        keyword_choice(&[
            Keyword::Function,
            Keyword::If,
            Keyword::Select,
            Keyword::Sub,
            Keyword::Type,
        ])
    }

    #[cfg(test)]
    mod tests {
        use crate::assert_parser_err;

        use super::*;

        #[test]
        fn test_sub_call_end_no_args_allowed() {
            assert_parser_err!(
                "END 42",
                QError::syntax_error(
                    "Expected: DEF or FUNCTION or IF or SELECT or SUB or TYPE or end-of-statement"
                )
            );
        }
    }
}

mod system {
    use crate::parser::base::parsers::FnMapTrait;
    use crate::parser::specific::keyword;
    use crate::parser::specific::whitespace::WhitespaceTrait;
    use crate::parser::statement_separator::peek_eof_or_statement_separator;

    use super::*;

    pub fn parse_system_p() -> impl Parser<Output = Statement> {
        keyword(Keyword::System)
            .and_demand(
                peek_eof_or_statement_separator()
                    .preceded_by_opt_ws()
                    .or_syntax_error("Expected: end-of-statement"),
            )
            .fn_map(|_| Statement::System)
    }

    #[cfg(test)]
    mod tests {
        use crate::assert_parser_err;
        use crate::common::*;

        #[test]
        fn test_sub_call_system_no_args_allowed() {
            assert_parser_err!(
                "SYSTEM 42",
                QError::syntax_error("Expected: end-of-statement"),
                1,
                7
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::common::*;
    use crate::parser::{PrintNode, Statement, TopLevelToken};

    use super::super::test_utils::*;

    #[test]
    fn test_top_level_comment() {
        let input = "' closes the file";
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                TopLevelToken::Statement(Statement::Comment(" closes the file".to_string(),))
                    .at_rc(1, 1)
            ]
        );
    }

    #[test]
    fn colon_separator_at_start_of_line() {
        let input = ": PRINT 42";
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                TopLevelToken::Statement(Statement::Print(PrintNode::one(42.as_lit_expr(1, 9))))
                    .at_rc(1, 3)
            ]
        );
    }
}
