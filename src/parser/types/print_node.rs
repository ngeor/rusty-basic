use crate::common::FileHandle;
use crate::parser::ExpressionNode;

/// A call to the PRINT sub.
/// As separators are important (even trailing needs to be preserved), PRINT has
/// its own type.
///
/// ```basic
/// PRINT [#file-number%,] [expression list] [{; | ,}]
/// LPRINT [expression list] [{; | ,}]
/// ```
///
/// Formatted output:
///
/// ```basic
/// PRINT [#file-number%,] USING format-string$; [expression list] [{; | ,}]
/// LPRINT USING format-string$; [expression list] [{; | ,}]
/// ```
///
/// `{; | ,}` Determines where the next output begins:
///
/// - `;` means print immediately after the last value.
/// - `,` means print at the start of the next print zone.
///   Print zones are 14 characters wide.
///
/// # Format specifiers
///
/// ## Characters that format a numeric expression
///
/// - `#` Digit position.
/// - `.` Decimal point position.
/// - `,` Placed left of the decimal point, prints a comma every third digit.
/// - `+` Position of number sign.
/// - `^^^^` Prints in exponential format.
/// - `-` Placed after digit, prints trailing sign for negative numbers.
/// - `$$` Prints leading $.
/// - `**` Fills leading spaces with *.
/// - `**$` Combines ** and $$.
///
/// ## Characters used to format a string expression
///
/// - `&` Prints entire string.
/// - `!` Prints only the first character of the string.
/// - `\ \` Prints first n characters, where n is the number of blanks between slashes + 2.
///
/// ## Characters used to print literal characters
///
/// - `_` Prints the following formatting character as a literal.
///
/// Any other character is printed as a literal.
#[derive(Clone, Debug, PartialEq)]
pub struct PrintNode {
    pub file_number: Option<FileHandle>,
    pub lpt1: bool,
    pub format_string: Option<ExpressionNode>,
    pub args: Vec<PrintArg>,
}

impl PrintNode {
    pub fn one(expression_node: ExpressionNode) -> Self {
        Self {
            file_number: None,
            lpt1: false,
            format_string: None,
            args: vec![PrintArg::Expression(expression_node)],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PrintArg {
    Comma,
    Semicolon,
    Expression(ExpressionNode),
}
