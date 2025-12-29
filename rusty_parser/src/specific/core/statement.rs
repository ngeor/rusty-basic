use crate::error::ParseError;
use crate::input::RcStringView;
use crate::lazy_parser;
use crate::pc::*;
use crate::specific::built_ins::built_in_sub_call_p;
use crate::specific::core::comment::comment_p;
use crate::specific::core::constant::constant_p;
use crate::specific::core::dim::{dim_p, redim_p};
use crate::specific::core::do_loop::do_loop_p;
use crate::specific::core::exit::statement_exit_p;
use crate::specific::core::for_loop::for_loop_p;
use crate::specific::core::go_sub::{statement_go_sub_p, statement_return_p};
use crate::specific::core::if_block::if_block_p;
use crate::specific::core::macros::bi_tuple;
use crate::specific::core::name::{
    bare_name_with_dots, identifier_with_dots, token_list_to_bare_name,
};
use crate::specific::core::on_error::statement_on_error_go_to_p;
use crate::specific::core::print::{parse_lprint_p, parse_print_p};
use crate::specific::core::resume::statement_resume_p;
use crate::specific::core::select_case::select_case_p;
use crate::specific::core::sub_call::sub_call_or_assignment_p;
use crate::specific::core::while_wend::while_wend_p;
use crate::specific::pc_specific::*;
use crate::specific::{
    BareName, DimVars, Expression, ExpressionPos, Expressions, Keyword, NamePos, Operator, Print,
};
use crate::BuiltInSub;
use rusty_common::*;

pub type StatementPos = Positioned<Statement>;
pub type Statements = Vec<StatementPos>;

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Comment(String),

    /// Assignment to a variable.
    ///
    /// Examples:
    ///
    /// ```bas
    /// A = 42
    /// A.Hello = 42
    /// A$ = "hello"
    /// ```
    ///
    /// The validity of the assignment is determined at the linting phase.
    Assignment(Assignment),

    Const(Constant),

    /// Declares a variable.
    ///
    /// Examples:
    ///
    /// ```bas
    /// DIM A                    ' (Bare)
    /// DIM A$                   ' (Compact)
    /// DIM A AS INTEGER         ' (ExtendedBuiltIn)
    /// DIM A AS STRING          ' (without length)
    /// DIM A AS STRING * 4      ' (with fixed length)
    /// DIM A AS UserDefinedType
    /// DIM SHARED A
    /// DIM A(1 TO 2)
    /// DIM A, B, C
    /// ```
    ///
    /// Parsing syntax reference:
    ///
    /// ```txt
    /// <dim> ::= "DIM"<ws><dim-name>
    /// <dim-name> ::= <bare-dim-name> | <compact-dim-name> | <extended-dim-name> | <user-defined-dim-name>
    ///
    /// (* DIM A, DIM A.B.C, DIM A.., DIM A(1 TO 2) *)
    /// <bare-dim-name> ::= <bare-name-with-dots-not-keyword><opt-array-dimensions>
    ///
    /// (* DIM A$, DIM A.B.C!, DIM A..% *)
    /// <compact-dim-name> ::= <compact-dim-name-left><opt-array-dimensions>
    /// (* it is allowed to have a keyword qualified by a string qualifier *)
    /// <compact-dim-name-left> ::= <bare-name-with-dots-not-keyword> ( "!" | "#" | "%" | "&")
    ///     | <bare-name-with-dots> "$"
    ///
    /// <extended-dim-name> ::= <bare-name-with-dots-not-keyword><opt-array-dimensions> <ws> "AS" <ws> <extended-dim-type>
    /// <extended-dim-type> ::= "INTEGER"
    ///     | "LONG"
    ///     | "SINGLE"
    ///     | "DOUBLE"
    ///     | <extended-dim-string>
    /// <extended-dim-string> ::= "STRING" <opt-ws> "*" <opt-ws> <expression> | "STRING"
    ///
    /// (* user defined type variable cannot have dots *)
    /// <user-defined-dim-name> ::= <bare-name-not-keyword><opt-array-dimensions> <ws> "AS" <ws> <user-defined-type>
    /// <user-defined-type> ::= <bare-name-not-keyword>
    ///
    /// <opt-array-dimensions> ::= "" | "(" <opt-ws> <array-dimensions> <opt-ws> ")"
    /// <array-dimensions> ::= <array-dimension> | <array-dimension> <opt-ws> "," <opt-ws> <array-dimensions>
    /// <array-dimension> ::= <expression> | <expression> <ws> "TO" <ws> <expression>
    /// ```
    Dim(DimList),

    Redim(DimList),

    SubCall(SubCall),
    BuiltInSubCall(BuiltInSubCall),

    /*
     * Decision flow
     */
    IfBlock(IfBlock),
    SelectCase(SelectCase),

    /*
     * Loops
     */
    ForLoop(ForLoop),
    While(ConditionalBlock),
    DoLoop(DoLoop),

    /*
     * Unstructured flow control
     */
    Label(CaseInsensitiveString),
    GoTo(CaseInsensitiveString),

    OnError(OnErrorOption),
    Resume(ResumeOption),

    GoSub(CaseInsensitiveString),
    Return(Option<CaseInsensitiveString>),

    Exit(ExitObject),

    End,
    System,

    /*
     * Special statements
     */
    Print(Print),
}

bi_tuple!(
    /// A constant declaration.
    Constant(name: NamePos, value: ExpressionPos)
);

bi_tuple!(
    /// An assignment statement.
    Assignment(lvalue: Expression, rvalue: ExpressionPos)
);

bi_tuple!(
    /// A call to a user defined SUB.
    SubCall(sub_name: BareName, args: Expressions)
);

bi_tuple!(
    /// A call to a built-in SUB.
    BuiltInSubCall(built_in_sub: BuiltInSub, args: Expressions)
);

/// A list of variables defined in a DIM statement.
#[derive(Clone, Debug, PartialEq)]
pub struct DimList {
    /// Specifies if the variables are shared. Can only be used on the global
    /// module. If shared, the variables are available in functions/subs.
    pub shared: bool,

    /// The variables defined in the DIM statement.
    pub variables: DimVars,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitObject {
    Function,
    Sub,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResumeOption {
    Bare,
    Next,
    Label(CaseInsensitiveString),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OnErrorOption {
    Next,
    Label(CaseInsensitiveString),
    Zero,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForLoop {
    pub variable_name: ExpressionPos,
    pub lower_bound: ExpressionPos,
    pub upper_bound: ExpressionPos,
    pub step: Option<ExpressionPos>,
    pub statements: Statements,
    pub next_counter: Option<ExpressionPos>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConditionalBlock {
    pub condition: ExpressionPos,
    pub statements: Statements,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfBlock {
    pub if_block: ConditionalBlock,
    pub else_if_blocks: Vec<ConditionalBlock>,
    pub else_block: Option<Statements>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectCase {
    /// The expression been matched
    pub expr: ExpressionPos,
    /// The case statements
    pub case_blocks: Vec<CaseBlock>,
    /// An optional CASE ELSE block
    pub else_block: Option<Statements>,
    /// Holds an optional inline comment after SELECT CASE X e.g. SELECT CASE X ' make a choice
    pub inline_comments: Vec<Positioned<String>>,
}

bi_tuple!(
    /// A case block can have one or more condition expressions and
    /// the statements to execute if the condition is met.
    CaseBlock(conditions: Vec<CaseExpression>, statements: Statements)
);

impl CaseBlock {
    pub fn has_conditions(&self) -> bool {
        // the CASE ELSE block does not have conditions
        !self.conditions().is_empty()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CaseExpression {
    Simple(ExpressionPos),
    Is(Operator, ExpressionPos),
    Range(ExpressionPos, ExpressionPos),
}

#[derive(Clone, Debug, PartialEq)]
pub struct DoLoop {
    pub condition: ExpressionPos,
    pub statements: Statements,
    pub position: DoLoopConditionPosition,
    pub kind: DoLoopConditionKind,
}

/// Indicates where the condition expression of
/// the DO LOOP is located.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoLoopConditionPosition {
    /// The condition is placed after the DO keyword, e.g.
    /// DO WHILE A > 0 ... LOOP
    Top,

    /// The condition is placed after the LOOP keyword, e.g.
    /// DO ... LOOP WHILE A > 0
    Bottom,
}

/// Specifies if a DO LOOP is using an
/// UNTIL or WHILE in its condition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoLoopConditionKind {
    Until,
    While,
}

impl Statement {
    pub fn constant(name: NamePos, value: ExpressionPos) -> Self {
        Self::Const(Constant::new(name, value))
    }

    pub fn assignment(left: Expression, right: ExpressionPos) -> Self {
        Self::Assignment(Assignment::new(left, right))
    }

    pub fn sub_call(name: BareName, args: Expressions) -> Self {
        Self::SubCall(SubCall::new(name, args))
    }

    pub fn built_in_sub_call(name: BuiltInSub, args: Expressions) -> Self {
        Self::BuiltInSubCall(BuiltInSubCall::new(name, args))
    }
}

lazy_parser!(pub fn statement_p<I = RcStringView, Output = Statement> ; struct LazyStatementP ; OrParser::new(vec![
    Box::new(statement_label_p()),
    Box::new(single_line_statement_p()),
    Box::new(if_block_p()),
    Box::new(for_loop_p()),
    Box::new(select_case_p()),
    Box::new(while_wend_p()),
    Box::new(do_loop_p()),
    Box::new(illegal_starting_keywords()),
]));

// Tries to read a statement that is allowed to be on a single line IF statement,
// excluding comments.
lazy_parser!(pub fn single_line_non_comment_statement_p<I = RcStringView, Output = Statement> ; struct SingleLineNonCommentStatement ; OrParser::new(vec![
    Box::new(dim_p()),
    Box::new(redim_p()),
    Box::new(constant_p()),
    Box::new(built_in_sub_call_p()),
    Box::new(parse_print_p()),
    Box::new(parse_lprint_p()),
    Box::new(sub_call_or_assignment_p()),
    Box::new(statement_go_to_p()),
    Box::new(statement_go_sub_p()),
    Box::new(statement_return_p()),
    Box::new(statement_exit_p()),
    Box::new(statement_on_error_go_to_p()),
    Box::new(statement_resume_p()),
    Box::new(end::parse_end_p()),
    Box::new(system::parse_system_p())
]));

/// Tries to read a statement that is allowed to be on a single line IF statement,
/// including comments.
pub fn single_line_statement_p() -> impl Parser<RcStringView, Output = Statement> {
    comment_p().or(single_line_non_comment_statement_p())
}

fn statement_label_p() -> impl Parser<RcStringView, Output = Statement> {
    // labels can have dots
    identifier_with_dots().and(colon(), |tokens, _| {
        Statement::Label(token_list_to_bare_name(tokens))
    })
}

fn statement_go_to_p() -> impl Parser<RcStringView, Output = Statement> {
    keyword_followed_by_whitespace_p(Keyword::GoTo)
        .and_keep_right(bare_name_with_dots().or_syntax_error("Expected: label"))
        .map(Statement::GoTo)
}

/// A parser that fails if an illegal starting keyword is found.
fn illegal_starting_keywords() -> impl Parser<RcStringView, Output = Statement> {
    keyword_map(&[
        (Keyword::Wend, ParseError::WendWithoutWhile),
        (Keyword::Else, ParseError::ElseWithoutIf),
        (Keyword::Loop, ParseError::LoopWithoutDo),
        (Keyword::Next, ParseError::NextWithoutFor),
    ])
    .flat_map(|input, err| Err((true, input, err)))
}

mod end {
    use crate::input::RcStringView;
    use crate::pc::*;
    use crate::specific::pc_specific::*;
    use crate::specific::{Keyword, Statement};

    pub fn parse_end_p() -> impl Parser<RcStringView, Output = Statement> {
        keyword(Keyword::End).map(|_| Statement::End)
    }

    #[cfg(test)]
    mod tests {
        use crate::assert_parser_err;
        use crate::error::ParseError;

        #[test]
        fn test_sub_call_end_no_args_allowed() {
            assert_parser_err!(
                "END 42",
                ParseError::syntax_error(
                    // TODO FIXME this was originally like this:
                    // "Expected: DEF or FUNCTION or IF or SELECT or SUB or TYPE or end-of-statement"
                    "No separator: 42"
                )
            );
        }
    }
}

mod system {
    use crate::input::RcStringView;
    use crate::pc::*;
    use crate::specific::core::statement_separator::peek_eof_or_statement_separator;
    use crate::specific::pc_specific::*;
    use crate::specific::{Keyword, Statement};

    pub fn parse_system_p() -> impl Parser<RcStringView, Output = Statement> {
        keyword(Keyword::System).and(
            opt_and_tuple(whitespace(), peek_eof_or_statement_separator())
                .or_syntax_error("Expected: end-of-statement"),
            |_, _| Statement::System,
        )
    }

    #[cfg(test)]
    mod tests {
        use crate::assert_parser_err;
        use crate::error::ParseError;

        #[test]
        fn test_sub_call_system_no_args_allowed() {
            assert_parser_err!(
                "SYSTEM 42",
                ParseError::syntax_error("Expected: end-of-statement"),
                1,
                7
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::specific::*;
    use crate::test_utils::*;
    use crate::*;
    use rusty_common::*;

    #[test]
    fn test_global_comment() {
        let input = "' closes the file";
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                GlobalStatement::Statement(Statement::Comment(" closes the file".to_string(),))
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
                GlobalStatement::Statement(Statement::Print(Print::one(42.as_lit_expr(1, 9))))
                    .at_rc(1, 3)
            ]
        );
    }
}
