use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::type_qualifier::{TypeQualifierParser, TypeQualifierPostGuardParser};
use crate::parser::{BareName, Name, TypeQualifier};

pub fn name_with_dot_p() -> impl Parser<Output = Name> {
    name_as_tokens().map(Name::from)
}

/// `result ::= <identifier-with-dots> <type-qualifier> | <keyword> "$"`
///
/// not followed by `<type-qualifier> | "."`
pub fn name_as_tokens() -> impl Parser<Output = NameAsTokens> {
    Alt2::new(identifier_and_opt_q(), keyword_and_dollar_string())
}

pub type OptTypeQualifierAsToken = Option<(Token, TypeQualifier)>;

impl Undo for (Token, TypeQualifier) {
    fn undo(self, tokenizer: &mut impl Tokenizer) {
        tokenizer.unread(self.0);
    }
}

pub enum NameAsTokens {
    Identifier {
        name: Token,
        opt_q: OptTypeQualifierAsToken,
    },
    KeywordDollar(Token),
}

impl From<NameAsTokens> for Name {
    fn from(n: NameAsTokens) -> Self {
        match n {
            NameAsTokens::Identifier { name, opt_q } => {
                Name::new(BareName::new(name.text), opt_q.map(|(_, q)| q))
            }
            NameAsTokens::KeywordDollar(token) => {
                let mut text = token.text;
                text.pop(); // remove trailing dollar sign
                Name::new(BareName::new(text), Some(TypeQualifier::DollarString))
            }
        }
    }
}

impl Undo for NameAsTokens {
    fn undo(self, tokenizer: &mut impl Tokenizer) {
        match self {
            Self::Identifier { name, opt_q } => {
                opt_q.undo(tokenizer);
                tokenizer.unread(name);
            }
            Self::KeywordDollar(token) => {
                tokenizer.unread(token);
            }
        }
    }
}

/// `result ::= <identifier-with-dots> <type-qualifier>`
///
/// not followed by `<type-qualifier> | "."`
fn identifier_and_opt_q() -> impl Parser<Output = NameAsTokens> {
    identifier_with_dots()
        .and_opt(TypeQualifierParser)
        .map(|(name, opt_q)| NameAsTokens::Identifier { name, opt_q })
}

/// `result ::= <keyword> "$"`
///
/// not followed by `<type-qualifier> | "."`
fn keyword_and_dollar_string() -> impl Parser<Output = NameAsTokens> {
    seq2(
        any_token_of(TokenType::KeywordWithDollarString),
        TypeQualifierPostGuardParser,
        |token, _| NameAsTokens::KeywordDollar(token),
    )
}

/// The same as [identifier_with_dots], mapped to a [BareName].
pub fn bare_name_p() -> impl Parser<Output = BareName> {
    identifier_with_dots().map(|token| token.text.into())
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
