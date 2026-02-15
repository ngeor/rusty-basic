use rusty_pc::and::TupleCombiner;
use rusty_pc::*;

use crate::core::{name_as_tokens_p, token_to_type_qualifier};
use crate::error::ParserError;
use crate::input::StringView;
use crate::pc_specific::OrExpected;
use crate::tokens::dot;
use crate::{BareName, Expression, ExpressionPos, ExpressionType, Name, NameAsTokens};

// property ::= <expr> "." <property-name>
// property-name ::= <identifier-without-dot>
//                 | <identifier-without-dot> <type-qualifier>
//
// expr must not be qualified

pub fn parser() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    base_expr_pos_p()
        .then_with_in_context(
            ctx_dot_property(),
            |first_expr_pos| is_qualified(&first_expr_pos.element),
            TupleCombiner,
        )
        .and_then(|(first_expr_pos, properties)| {
            // not possible to have properties for qualified first expr
            // therefore either we don't have properties
            // or if we do then the first expr is bare
            debug_assert!(properties.is_none() || !is_qualified(&first_expr_pos.element));

            match properties {
                Some((property_name, opt_q_token)) => {
                    let text = property_name.as_str();
                    let mut it = text.split('.').peekable();
                    let mut result = first_expr_pos;
                    while let Some(name) = it.next() {
                        if name.is_empty() {
                            // detected something like X = Y(1).A..B
                            return Err(ParserError::expected("identifier").to_fatal());
                        }

                        let property_name = if it.peek().is_some() {
                            Name::bare(BareName::new(name.to_owned()))
                        } else {
                            Name::new(
                                BareName::new(name.to_owned()),
                                opt_q_token.as_ref().map(token_to_type_qualifier),
                            )
                        };

                        result = result.map(|prev_expr| {
                            Expression::Property(
                                Box::new(prev_expr),
                                property_name,
                                ExpressionType::Unresolved,
                            )
                        });
                    }
                    Ok(result)
                }
                None => Ok(first_expr_pos),
            }
        })
}

/// Parses an optional `.property` after the first expression was parsed.
/// The boolean context indicates whether the previously parsed expression
/// was qualified or not.
/// If it was qualified, we return Ok(None) without trying to parse,
/// because qualified names can't have properties.
fn ctx_dot_property()
-> impl Parser<StringView, bool, Output = Option<NameAsTokens>, Error = ParserError> {
    ctx_parser()
        .and_then(|was_first_expr_qualified| {
            if was_first_expr_qualified {
                // fine, don't parse anything further
                // i.e. A$(1).Name won't attempt to parse the .Name part
                Ok(None)
            } else {
                // it wasn't qualified, therefore let the dot_property continue
                default_parse_error()
            }
        })
        .or(dot_property().no_context())
}

fn dot_property() -> impl Parser<StringView, Output = Option<NameAsTokens>, Error = ParserError> {
    dot()
        .and_keep_right(name_as_tokens_p().or_expected("property name after dot"))
        .to_option()
}

/// Returns a name expression which could be a function call (array)
/// e.g. `A$(1)`,
/// or a variable e.g. `A.B.C$` (which can be Variable or Property).
/// can't use expression_pos_p because it will stack overflow
fn base_expr_pos_p() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    // order is important, variable matches anything that function_call_or_array_element matches
    OrParser::new(vec![
        Box::new(super::function_call_or_array_element::parser()),
        Box::new(super::variable::parser()),
    ])
}

pub fn is_qualified(expr: &Expression) -> bool {
    match expr {
        Expression::Variable(name, _)
        | Expression::FunctionCall(name, _)
        | Expression::Property(_, name, _) => !name.is_bare(),
        _ => {
            panic!("Unexpected property type {:?}", expr)
        }
    }
}
