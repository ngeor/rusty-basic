/// A letter range that is used in a default type definition, e.g. A-Z
#[derive(Clone, Debug, PartialEq)]
pub enum LetterRange {
    /// A range of letters, e.g. A-Z.
    Range(char, char),
    /// A single letter, e.g. A, used as a shorthand for A-A.
    Single(char),
}
