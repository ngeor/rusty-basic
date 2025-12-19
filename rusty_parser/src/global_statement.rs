use crate::def_type;
use crate::implementation;
use crate::pc::*;
use crate::pc_specific::*;
use crate::statement;
use crate::types::*;
use crate::user_defined_type;
use crate::{declaration, ParseError};

// TODO this is a complex parser, revisit it

// [ws|eol|col]*
// [main-program]?
// [ws|eol|col]*
// EOF
//
// main-program: statement [ws* next-statement ws*]*
// next-statement: comment | separator [ws|eol|col]* statement
// comment: ' comment
// separator: eol|col

pub fn program_parser_p<I: Tokenizer + 'static>() -> impl Parser<I, Output = Program> {
    ws_eol_col_zero_or_more()
        .and_opt_keep_right(main_program())
        .and_opt_keep_left(ws_eol_col_zero_or_more())
        .and_opt_keep_left(demand_eof())
        .map(|opt| opt.unwrap_or_default())
}

fn main_program<I: Tokenizer + 'static>() -> impl Parser<I, Output = Program> {
    global_statement_pos_p().and_opt(next_statements(), |first, opt_next| {
        let mut program = vec![first];
        if let Some(next) = opt_next {
            program.extend(next);
        }
        program
    })
}

fn next_statements<I: Tokenizer + 'static>() -> impl Parser<I, Output = Program> {
    OptAndPC::new(
        whitespace(),
        next_statement().and_opt_keep_left(whitespace()),
    )
    .keep_right()
    .zero_or_more()
}

fn next_statement<I: Tokenizer + 'static>() -> impl Parser<I, Output = GlobalStatementPos> {
    separator::separator()
        .and_without_undo_keep_right(OrParser::new(vec![
            // need to detect EOF, because the separator we detected might have been the last EOL of the file
            Box::new(detect_eof().map(|_| None)),
            // otherwise it must be a statement
            Box::new(
                global_statement_pos_p()
                    .or_syntax_error("Expected statement")
                    .map(Some),
            ),
        ]))
        .flat_map(|opt| match opt {
            // map the statement
            Some(s) => ParseResult::Ok(s),
            // map the EOF back to an incomplete result
            None => ParseResult::None,
        })
}

mod separator {
    use crate::statement_separator::no_separator_needed_before_comment;

    use super::*;

    pub fn separator<I: Tokenizer + 'static>() -> impl Parser<I, Output = ()> {
        OrParser::new(vec![
            // EOL or colon separator
            Box::new(eol_or_colon_separator()),
            // peek to see if we have a comment coming up, which is the only statement that does not need a separator
            Box::new(no_separator_needed_before_comment()),
            // otherwise raise an error, unless we're at EOF
            Box::new(raise_err()),
        ])
    }

    fn eol_or_colon_separator<I: Tokenizer + 'static>() -> impl Parser<I, Output = ()> {
        eol_col_one_or_more().and_opt(ws_eol_col_zero_or_more(), |_, _| ())
    }

    fn raise_err<I: Tokenizer + 'static>() -> impl Parser<I, Output = ()> {
        any_token().flat_map(|t| {
            ParseResult::Err(ParseError::SyntaxError(format!("No separator: {}", t.text)))
        })
    }
}

/// Parses one or more tokens that are end of line or colon.
fn eol_col_one_or_more<I: Tokenizer + 'static>() -> impl Parser<I, Output = ()> {
    any_token()
        .filter(|t| TokenType::Colon.matches(t) || TokenType::Eol.matches(t))
        .one_or_more()
        .map(|_| ())
}

/// Parses zero or more tokens that are whitespace, end of line, or colon.
fn ws_eol_col_zero_or_more<I: Tokenizer + 'static>() -> impl Parser<I, Output = ()> {
    any_token()
        .filter(|t| {
            TokenType::Colon.matches(t)
                || TokenType::Eol.matches(t)
                || TokenType::Whitespace.matches(t)
        })
        .zero_or_more()
        .map(|_| ())
}

/// Fails unless the input is fully consumed.
/// If we're at EOF, the parser returns a happy empty result.
/// Otherwise it returns a syntax error.
/// This is a failsafe to ensure we have parsed the entire input.
fn demand_eof<I: Tokenizer + 'static>() -> impl Parser<I, Output = ()> {
    any_token().flat_map_negate_none(|t| {
        ParseResult::Err(ParseError::SyntaxError(format!(
            "Cannot parse, expected EOF {:?}",
            t
        )))
    })
}

/// Parses a global statement.
/// This includes regular statements, but also DEF types,
/// declarations, implementations, and user-defined types.
fn global_statement_pos_p<I: Tokenizer + 'static>() -> impl Parser<I, Output = GlobalStatementPos> {
    OrParser::new(vec![
        Box::new(def_type::def_type_p().map(GlobalStatement::DefType)),
        Box::new(declaration::declaration_p()),
        Box::new(implementation::implementation_p()),
        Box::new(statement::statement_p().map(GlobalStatement::Statement)),
        Box::new(user_defined_type::user_defined_type_p().map(GlobalStatement::UserDefinedType)),
    ])
    .with_pos()
}
