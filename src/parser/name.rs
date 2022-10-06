use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::type_qualifier::{TypeQualifierParser, TypeQualifierPostGuardParser};
use crate::parser::{BareName, Name, TypeQualifier};

pub fn name_with_dot_p() -> impl Parser<Output = Name> {
    name_as_tokens().map(|(name_token, opt_q_token)| {
        Name::new(BareName::new(name_token.text), opt_q_token.map(|(_, q)| q))
    })
}

/// `result ::= <identifier-with-dots> <type-qualifier> | <keyword> "$"`
///
/// not followed by `<type-qualifier> | "."`
pub fn name_as_tokens() -> impl Parser<Output = (Token, Option<(Token, TypeQualifier)>)> {
    Alt2::new(identifier_and_opt_q(), keyword_and_dollar_string())
}

/// `result ::= <identifier-with-dots> <type-qualifier>`
///
/// not followed by `<type-qualifier> | "."`
fn identifier_and_opt_q() -> impl Parser<Output = (Token, Option<(Token, TypeQualifier)>)> {
    identifier_with_dots().and_opt(TypeQualifierParser)
}

/// `result ::= <keyword> "$"`
///
/// not followed by `<type-qualifier> | "."`
fn keyword_and_dollar_string() -> impl Parser<Output = (Token, Option<(Token, TypeQualifier)>)> {
    seq2(
        any_token_of(TokenType::Keyword).and(dollar_sign()),
        TypeQualifierPostGuardParser,
        |(name, type_token), _| (name, Some((type_token, TypeQualifier::DollarString))),
    )
}

/// The same as [identifier_with_dots], mapped to a [BareName].
pub fn bare_name_p() -> impl Parser<Output = BareName> {
    identifier_with_dots().map(|token| token.text.into())
}

impl Undo for (Token, TypeQualifier) {
    fn undo(self, tokenizer: &mut impl Tokenizer) {
        tokenizer.unread(self.0);
    }
}

impl Undo for (Token, Option<(Token, TypeQualifier)>) {
    fn undo(self, tokenizer: &mut impl Tokenizer) {
        self.1.undo(tokenizer);
        tokenizer.unread(self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::pc_specific::test_helper::create_string_tokenizer;

    #[test]
    fn test_any_word_with_dot() {
        let inputs = ["abc", "abc1", "abc.def"];
        let expected_outputs = ["abc", "abc1", "abc.def"];
        for i in 0..inputs.len() {
            let input = inputs[i];
            let expected_output = expected_outputs[i];
            let mut eol_reader = create_string_tokenizer(input);
            let parser = bare_name_p();
            let result = parser.parse(&mut eol_reader).expect("Should succeed");
            assert_eq!(result, BareName::from(expected_output));
        }
    }
}
