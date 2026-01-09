/// Creates a parser that matches the specific value.
/// Typically used to match primitives such as characters and strings.
pub trait ToSpecificParser {
    /// The type of the parser that will be created.
    type Parser;

    /// Creates a parser that matches this specific value.
    fn to_specific_parser(self) -> Self::Parser;
}
