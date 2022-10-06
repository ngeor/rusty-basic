use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::type_qualifier::type_qualifier_as_token;
use crate::parser::{BareName, Name, TypeQualifier};

/// Parses a name. The name must start with a letter and can include
/// letters, digits or dots. The name can optionally be qualified by a type
/// qualifier.
///
/// The parser validates the maximum length of the name and checks that the name
/// is not a keyword (with the exception of strings, e.g. `end$`).
pub fn name_with_dot_p() -> impl Parser<Output = Name> {
    Alt2::new(identifier_and_opt_q(), keyword_and_dollar_string())
}

fn identifier_and_opt_q() -> impl Parser<Output = Name> {
    identifier_with_dots()
        .and_opt(type_qualifier_as_token())
        .map(|(l, r)| Name::new(BareName::new(l.text), TypeQualifier::from_opt_token(&r)))
}

fn keyword_and_dollar_string() -> impl Parser<Output = Name> {
    any_token_of(TokenType::Keyword)
        .and(dollar_sign())
        .map(|(l, _)| Name::new(BareName::new(l.text), Some(TypeQualifier::DollarString)))
}

// bare name node

pub fn bare_name_as_token() -> impl Parser<Output = Token> {
    identifier_with_dots().unless_followed_by(type_qualifier_as_token())
}

pub fn bare_name_p() -> impl Parser<Output = BareName> {
    bare_name_as_token().map(|x| x.text.into()) // TODO make a parser for simpler .into() cases
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
