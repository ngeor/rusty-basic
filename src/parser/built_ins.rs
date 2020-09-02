use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::expression;
use crate::parser::pc::common::*;
use crate::parser::pc::map::{map, opt_map};
use crate::parser::pc::misc::*;
use crate::parser::pc::*;
use crate::parser::types::*;
use std::io::BufRead;

/// Parses built-in subs which have a special syntax.
pub fn parse_built_in<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, crate::parser::Statement, QError>> {
    or_vec(vec![
        input::parse_input(),
        line_input::parse_line_input(),
        name::parse_name(),
        open::parse_open(),
    ])
}

mod input {
    use super::*;
    pub fn parse_input<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
        map(parse_input_args(), |r| {
            Statement::SubCall("INPUT".into(), r)
        })
    }

    pub fn parse_input_args<T: BufRead + 'static>() -> Box<
        dyn Fn(
            EolReader<T>,
        ) -> ReaderResult<EolReader<T>, Vec<crate::parser::ExpressionNode>, QError>,
    > {
        drop_left(crate::parser::pc::ws::seq2(
            try_read_keyword(Keyword::Input),
            demand(
                // TODO demand variable expression directly
                map_default_to_not_found(csv_zero_or_more(expression::expression_node())),
                QError::syntax_error_fn("Expected: at least one variable"),
            ),
            QError::syntax_error_fn("Expected: whitespace after INPUT"),
        ))
    }
}

mod line_input {
    use super::*;
    pub fn parse_line_input<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
        map(
            crate::parser::pc::ws::seq2(
                try_read_keyword(Keyword::Line),
                demand(
                    super::input::parse_input_args(),
                    QError::syntax_error_fn("Expected: INPUT after LINE"),
                ),
                QError::syntax_error_fn("Expected: whitespace after LINE"),
            ),
            |(_, r)| Statement::SubCall("LINE INPUT".into(), r),
        )
    }
}

mod name {
    use super::*;
    pub fn parse_name<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
        map(
            seq4(
                try_read_keyword(Keyword::Name),
                expression::demand_back_guarded_expression_node(),
                demand_keyword(Keyword::As),
                expression::demand_guarded_expression_node(),
            ),
            |(_, l, _, r)| Statement::SubCall("NAME".into(), vec![l, r]),
        )
    }
}

mod open {
    use super::*;

    pub fn parse_open<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
        // TODO support OPEN("file.txt")FOR INPUT ACCESS READ AS 1
        // TODO support OPEN("file.txt")ACCESS READ AS 1
        // TODO support OPEN("file.txt")AS 1
        map(
            crate::parser::pc::ws::seq2(
                opt_seq3(parse_filename(), parse_open_mode(), parse_open_access()),
                demand(
                    parse_file_number(),
                    QError::syntax_error_fn("Expected: AS file-number"),
                ),
                QError::syntax_error_fn("Expected: whitespace before AS"),
            ),
            |((file_name, opt_file_mode, opt_file_access), file_number)| {
                Statement::SubCall(
                    "OPEN".into(),
                    vec![
                        file_name,
                        // TODO take actual pos
                        Expression::IntegerLiteral(opt_file_mode.unwrap_or(FileMode::Input).into())
                            .at(Location::start()),
                        // TODO take actual pos
                        Expression::IntegerLiteral(
                            opt_file_access.unwrap_or(FileAccess::Unspecified).into(),
                        )
                        .at(Location::start()),
                        file_number,
                    ],
                )
            },
        )
    }

    fn parse_filename<T: BufRead + 'static>() -> Box<
        dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, crate::parser::ExpressionNode, QError>,
    > {
        drop_left(crate::parser::pc::ws::seq2(
            try_read_keyword(Keyword::Open),
            demand(
                expression::expression_node(),
                QError::syntax_error_fn("Expected: filename after OPEN"),
            ),
            QError::syntax_error_fn("Expected: whitespace after OPEN"),
        ))
    }

    fn parse_open_mode<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, FileMode, QError>> {
        drop_left(crate::parser::pc::ws::seq2(
            crate::parser::pc::ws::one_or_more_leading(try_read_keyword(Keyword::For)),
            demand(
                opt_map(read_any_keyword(), |(k, _)| match k {
                    Keyword::Append => Some(FileMode::Append),
                    Keyword::Input => Some(FileMode::Input),
                    Keyword::Output => Some(FileMode::Output),
                    _ => None,
                }),
                QError::syntax_error_fn("Invalid file mode"),
            ),
            QError::syntax_error_fn("Expected: whitespace after FOR"),
        ))
    }

    fn parse_open_access<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, FileAccess, QError>> {
        drop_left(crate::parser::pc::ws::seq2(
            crate::parser::pc::ws::one_or_more_leading(try_read_keyword(Keyword::Access)),
            demand(
                opt_map(read_any_keyword(), |(k, _)| match k {
                    Keyword::Read => Some(FileAccess::Read),
                    _ => None,
                }),
                QError::syntax_error_fn("Invalid file access"),
            ),
            QError::syntax_error_fn("Expected: whitespace after ACCESS"),
        ))
    }

    fn parse_file_number<T: BufRead + 'static>() -> Box<
        dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, crate::parser::ExpressionNode, QError>,
    > {
        drop_left(crate::parser::pc::ws::seq2(
            try_read_keyword(Keyword::As),
            demand(
                expression::expression_node(),
                QError::syntax_error_fn("Expected: file number"),
            ),
            QError::syntax_error_fn("Expected: whitespace after AS"),
        ))
    }
}
