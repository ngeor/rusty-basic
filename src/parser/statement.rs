use crate::common::*;
use crate::parser::built_ins;
use crate::parser::comment;
use crate::parser::constant;
use crate::parser::dim;
use crate::parser::for_loop;
use crate::parser::if_block;
use crate::parser::name;
use crate::parser::name::bare_name_p;
use crate::parser::pc::binary::BinaryParser;
use crate::parser::pc::text::whitespace_p;
use crate::parser::pc::unary::UnaryParser;
use crate::parser::pc::unary_fn::UnaryFnParser;
use crate::parser::pc::{item_p, Parser, Reader};
use crate::parser::pc_specific::{keyword_p, PcSpecific};
use crate::parser::select_case;
use crate::parser::sub_call;
use crate::parser::types::*;
use crate::parser::while_wend;

pub fn statement_p<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    dim::dim_p()
        .box_dyn()
        .or(constant::constant_p().box_dyn())
        .or(comment::comment_p().box_dyn())
        .or(built_ins::parse_built_in_p().box_dyn())
        .or(statement_label_p().box_dyn())
        .or(sub_call::sub_call_or_assignment_p().box_dyn())
        .or(if_block::if_block_p().box_dyn())
        .or(for_loop::for_loop_p().box_dyn())
        .or(select_case::select_case_p().box_dyn())
        .or(while_wend::while_wend_p().box_dyn())
        .or(statement_go_to_p().box_dyn())
        .or(statement_on_error_go_to_p().box_dyn())
        .or(illegal_starting_keywords().box_dyn())
}

/// Tries to read a statement that is allowed to be on a single line IF statement,
/// excluding comments.
pub fn single_line_non_comment_statement_p<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    dim::dim_p()
        .box_dyn()
        .or(constant::constant_p().box_dyn())
        .or(built_ins::parse_built_in_p().box_dyn())
        .or(sub_call::sub_call_or_assignment_p().box_dyn())
        .or(statement_go_to_p().box_dyn())
        .or(statement_on_error_go_to_p().box_dyn())
}

/// Tries to read a statement that is allowed to be on a single line IF statement,
/// including comments.
pub fn single_line_statement_p<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    comment::comment_p()
        .box_dyn()
        .or(dim::dim_p().box_dyn())
        .or(constant::constant_p().box_dyn())
        .or(built_ins::parse_built_in_p().box_dyn())
        .or(sub_call::sub_call_or_assignment_p().box_dyn())
        .or(statement_go_to_p().box_dyn())
        .or(statement_on_error_go_to_p().box_dyn())
}

// TODO: remove 'static from as many Reader as possible

fn statement_label_p<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    name::bare_name_p()
        .and(item_p(':'))
        .keep_left()
        .map(|l| Statement::Label(l))
}

fn statement_go_to_p<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_p(Keyword::GoTo)
        .and_demand(whitespace_p().or_syntax_error("Expected: whitespace after GOTO"))
        .and_demand(bare_name_p().or_syntax_error("Expected: label"))
        .map(|(_, l)| Statement::GoTo(l))
}

fn statement_on_error_go_to_p<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_p(Keyword::On)
        .and_demand(whitespace_p().or_syntax_error("Expected: whitespace after ON"))
        .and_demand(keyword_p(Keyword::Error).or_syntax_error("Expected: ERROR"))
        .and_demand(whitespace_p().or_syntax_error("Expected: whitespace after ERROR"))
        .and_demand(keyword_p(Keyword::GoTo).or_syntax_error("Expected: GOTO"))
        .and_demand(whitespace_p().or_syntax_error("Expected: whitespace after GOTO"))
        .and_demand(name::bare_name_p().or_syntax_error("Expected: label"))
        .map(|(_, l)| Statement::ErrorHandler(l))
}

fn illegal_starting_keywords<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_p(Keyword::Wend)
        .or(keyword_p(Keyword::Else))
        .and_then(|(k, _)| match k {
            Keyword::Wend => Err(QError::WendWithoutWhile),
            Keyword::Else => Err(QError::ElseWithoutIf),
            _ => panic!("Parser should not have parsed {}", k),
        })
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
