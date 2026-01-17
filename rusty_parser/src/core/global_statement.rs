use rusty_common::*;
use rusty_pc::*;

use crate::core::declaration::declaration_p;
use crate::core::def_type::def_type_p;
use crate::core::implementation::implementation_p;
use crate::core::{statement, user_defined_type};
use crate::error::ParserError;
use crate::input::RcStringView;
use crate::pc_specific::*;
use crate::tokens::{TokenType, any_token, any_token_of, peek_token, whitespace_ignoring};
use crate::*;

pub type Program = Vec<GlobalStatementPos>;
pub type GlobalStatementPos = Positioned<GlobalStatement>;

/// Represents a parsed token that can appear as a top-level element of the
/// parsing tree.
///
/// Syntax reference
///
/// ```txt
/// <program> ::= <global-statement> | <global-statement><program>
///
/// <global-statement> ::= <comment>
///     | <def-type>
///     | <declaration>
///     | <statement>
///     | <function>
///     | <sub>
///     | <user-defined-type>
///
/// <statement> ::= <comment>
///     | <dim>
///     | <const>
///     | <built-in>
///     | <label>
///     | <sub-call>
///     | <assignment>
///     | <if-block>
///     | <for-loop>
///     | <select-case>
///     | <while-wend>
///     | <go-to>
///     | <on-error-go-to>
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum GlobalStatement {
    /// A default type definition, e.g. `DEFINT A-Z.`
    DefType(DefType),

    /// A function declaration, e.g. `DECLARE FUNCTION Add(A, B)`
    FunctionDeclaration(FunctionDeclaration),

    /// A function implementation
    FunctionImplementation(FunctionImplementation),

    /// A simple or compound statement
    Statement(Statement),

    /// A sub declaration, e.g. `DECLARE SUB Connect`
    SubDeclaration(SubDeclaration),

    /// A sub implementation
    SubImplementation(SubImplementation),

    /// A user defined type definition
    UserDefinedType(UserDefinedType),
}

impl GlobalStatement {
    pub fn function_declaration(name: NamePos, parameters: Parameters) -> Self {
        Self::FunctionDeclaration(FunctionDeclaration::new(name, parameters))
    }

    pub fn sub_declaration(name: BareNamePos, parameters: Parameters) -> Self {
        Self::SubDeclaration(SubDeclaration::new(name, parameters))
    }
}

impl From<Statement> for GlobalStatement {
    fn from(s: Statement) -> Self {
        Self::Statement(s)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubprogramDeclaration<T> {
    pub name: Positioned<T>,
    pub parameters: Parameters,
}

impl<T> SubprogramDeclaration<T> {
    pub fn new(name: Positioned<T>, parameters: Parameters) -> Self {
        Self { name, parameters }
    }
}

pub type SubDeclaration = SubprogramDeclaration<BareName>;

pub type FunctionDeclaration = SubprogramDeclaration<Name>;

/// The implementation of a subprogram (FUNCTION or SUB).
#[derive(Clone, Debug, PartialEq)]
pub struct SubprogramImplementation<T> {
    /// The name of the subprogram.
    /// It can be [BareName] for SUBs or [Name] for FUNCTIONs.
    pub name: Positioned<T>,

    /// The parameters of the subprogram.
    pub params: Parameters,

    /// The body (statements) of the subprogram.
    pub body: Statements,

    /// Determines if the subprogram is static. Static subprograms retain their
    /// variable values between calls.
    pub is_static: bool,
}

/// The implementation of a SUB.
/// The name type is [BareName] as SUBs don't have a return type.
pub type SubImplementation = SubprogramImplementation<BareName>;

/// The implementation of a FUNCTION.
/// Functions have a built-in return type.
/// The name type is [Name] because the name is not resolved yet.
/// After linting, the name will be resolved.
pub type FunctionImplementation = SubprogramImplementation<Name>;

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

pub fn program_parser_p() -> impl Parser<RcStringView, Output = Program, Error = ParserError> {
    ws_eol_col_zero_or_more()
        .and_opt_keep_right(main_program())
        .and_opt_keep_left(ws_eol_col_zero_or_more())
        .and_opt_keep_left(demand_eof())
        .map(|opt| opt.unwrap_or_default())
}

fn main_program() -> impl Parser<RcStringView, Output = Program, Error = ParserError> {
    global_statement_pos_p().and_opt(next_statements(), |first, opt_next| {
        let mut program = vec![first];
        if let Some(next) = opt_next {
            program.extend(next);
        }
        program
    })
}

fn next_statements() -> impl Parser<RcStringView, Output = Program, Error = ParserError> {
    opt_and_keep_right(
        whitespace_ignoring(),
        next_statement().and_opt_keep_left(whitespace_ignoring()),
    )
    .zero_or_more()
}

fn next_statement() -> impl Parser<RcStringView, Output = GlobalStatementPos, Error = ParserError> {
    separator::separator()
        .and_keep_right(OrParser::new(vec![
            // need to detect EOF, because the separator we detected might have been the last EOL of the file
            Box::new(detect_eof().map(|_| None)),
            // otherwise it must be a statement
            Box::new(global_statement_pos_p().or_expected("Statement").map(Some)),
        ]))
        .flat_map(|input, opt| match opt {
            // map the statement
            Some(s) => Ok((input, s)),
            // map the EOF back to an incomplete result
            None => default_parse_error(input),
        })
}

/// Returns Ok(()) if we're at EOF,
/// otherwise an incomplete result,
/// without modifying the input.
fn detect_eof() -> impl Parser<RcStringView, Output = (), Error = ParserError> {
    peek_token().flat_map_negate_none(|i, _| default_parse_error(i))
}

mod separator {
    use super::*;
    use crate::core::statement_separator::no_separator_needed_before_comment;

    pub fn separator() -> impl Parser<RcStringView, Output = (), Error = ParserError> {
        OrParser::new(vec![
            // EOL or colon separator
            Box::new(eol_or_colon_separator()),
            // peek to see if we have a comment coming up, which is the only statement that does not need a separator
            Box::new(no_separator_needed_before_comment()),
            // otherwise raise an error, unless we're at EOF
            Box::new(raise_err()),
        ])
    }

    fn eol_or_colon_separator() -> impl Parser<RcStringView, Output = (), Error = ParserError> {
        eol_col_one_or_more().and_opt(ws_eol_col_zero_or_more(), |_, _| ())
    }

    fn raise_err() -> impl Parser<RcStringView, Output = (), Error = ParserError> {
        any_token().flat_map(|input, _t| {
            Err((input, ParserError::expected("end-of-statement").to_fatal()))
        })
    }
}

/// Parses one or more tokens that are end of line or colon.
fn eol_col_one_or_more() -> impl Parser<RcStringView, Output = (), Error = ParserError> {
    any_token_of!(TokenType::Eol ; symbols = ':').many(IgnoringManyCombiner)
}

/// Parses zero or more tokens that are whitespace, end of line, or colon.
fn ws_eol_col_zero_or_more() -> impl Parser<RcStringView, Output = (), Error = ParserError> {
    any_token_of!(TokenType::Whitespace, TokenType::Eol ; symbols = ':')
        .many_allow_none(IgnoringManyCombiner)
}

/// Fails unless the input is fully consumed.
/// If we're at EOF, the parser returns a happy empty result.
/// Otherwise it returns a syntax error.
/// This is a failsafe to ensure we have parsed the entire input.
fn demand_eof() -> impl Parser<RcStringView, Output = (), Error = ParserError> {
    any_token().flat_map_negate_none(|input, t| {
        Err((
            input,
            ParserError::syntax_error(&format!("Cannot parse, expected EOF {:?}", t)),
        ))
    })
}

/// Parses a global statement.
/// This includes regular statements, but also DEF types,
/// declarations, implementations, and user-defined types.
fn global_statement_pos_p()
-> impl Parser<RcStringView, Output = GlobalStatementPos, Error = ParserError> {
    OrParser::new(vec![
        Box::new(def_type_p().map(GlobalStatement::DefType)),
        Box::new(declaration_p()),
        Box::new(implementation_p()),
        Box::new(statement::statement_p().map(GlobalStatement::Statement)),
        Box::new(user_defined_type::user_defined_type_p().map(GlobalStatement::UserDefinedType)),
    ])
    .with_pos()
}
