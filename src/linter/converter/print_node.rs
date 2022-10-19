use crate::common::*;
use crate::linter::converter::converter::Context;
use crate::linter::converter::expr_rules::ExprStateful;
use crate::linter::converter::ExprContext;
use crate::parser::{PrintArg, PrintNode};

pub fn on_print_node(
    a: PrintNode,
) -> impl Stateful<Output = PrintNode, Error = QErrorNode, State = Context> {
    let PrintNode {
        file_number,
        lpt1,
        format_string,
        args,
    } = a;
    let format_string =
        Unit::new(format_string).opt_flat_map(|e| ExprStateful::new(e, ExprContext::Default));
    let args = Unit::new(args).vec_flat_map(on_print_arg);
    (format_string, args).map(move |(format_string, args)| PrintNode {
        file_number,
        lpt1,
        format_string,
        args,
    })
}

fn on_print_arg(
    a: PrintArg,
) -> impl Stateful<Output = PrintArg, Error = QErrorNode, State = Context> {
    FnStateful::new(move |ctx: &mut Context| match a {
        PrintArg::Expression(e) => ExprStateful::new(e, ExprContext::Default)
            .map(PrintArg::Expression)
            .unwrap(ctx),
        _ => Ok(a),
    })
}
