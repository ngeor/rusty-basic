#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Recognition {
    Negative,
    Partial,
    Positive,
}

pub trait Recognizer {
    fn recognize(&self, buffer: &str) -> Recognition;
}

struct AnySingleCharRecognizer {}

impl Recognizer for AnySingleCharRecognizer {
    fn recognize(&self, buffer: &str) -> Recognition {
        if buffer.len() == 1 {
            Recognition::Positive
        } else {
            Recognition::Negative
        }
    }
}

struct SingleNewLineRecognizer {}

impl Recognizer for SingleNewLineRecognizer {
    fn recognize(&self, buffer: &str) -> Recognition {
        if buffer == "\r" || buffer == "\n" || buffer == "\r\n" {
            Recognition::Positive
        } else {
            Recognition::Negative
        }
    }
}

struct ManyPredicateRecognizer<T: Fn(char) -> bool> {
    predicate: T,
}

impl<T: Fn(char) -> bool> Recognizer for ManyPredicateRecognizer<T> {
    fn recognize(&self, buffer: &str) -> Recognition {
        if !buffer.is_empty() && buffer.chars().all(&self.predicate) {
            Recognition::Positive
        } else {
            Recognition::Negative
        }
    }
}

struct LeadingRemainingPredicateRecognizer<T: Fn(char) -> bool, U: Fn(char) -> bool> {
    leading_predicate: T,
    remaining_predicate: U,
}

impl<T: Fn(char) -> bool, U: Fn(char) -> bool> Recognizer
    for LeadingRemainingPredicateRecognizer<T, U>
{
    fn recognize(&self, buffer: &str) -> Recognition {
        let mut idx: usize = 0;
        for ch in buffer.chars() {
            let matches = if idx == 0 {
                (self.leading_predicate)(ch)
            } else {
                (self.remaining_predicate)(ch)
            };
            if !matches {
                return Recognition::Negative;
            }
            idx += 1;
        }
        if idx > 0 {
            Recognition::Positive
        } else {
            Recognition::Negative
        }
    }
}

struct SingleCharRecognizer {
    needle: char,
}

impl Recognizer for SingleCharRecognizer {
    fn recognize(&self, buffer: &str) -> Recognition {
        if buffer.len() == 1 && buffer.chars().all(|c| c == self.needle) {
            Recognition::Positive
        } else {
            Recognition::Negative
        }
    }
}

struct StrRecognizer<'a> {
    needle: &'a str,
}

impl<'a> StrRecognizer<'a> {
    pub fn new(needle: &'a str) -> Self {
        Self { needle }
    }
}

impl<'a> Recognizer for StrRecognizer<'a> {
    fn recognize(&self, buffer: &str) -> Recognition {
        let mut needle_iter = self.needle.chars();
        let mut buffer_iter = buffer.chars();
        loop {
            let needle_next = needle_iter.next();
            let buffer_next = buffer_iter.next();
            match needle_next {
                Some(ch) => {
                    match buffer_next {
                        Some(ch2) => {
                            if ch.eq_ignore_ascii_case(&ch2) {
                                // continue
                            } else {
                                // abort
                                return Recognition::Negative;
                            }
                        }
                        None => {
                            // buffer too short but maybe it will match later?
                            return Recognition::Partial;
                        }
                    }
                }
                None => {
                    return match buffer_next {
                        Some(_) => {
                            // buffer still has input so we don't match
                            Recognition::Negative
                        }
                        None => {
                            // buffer ended same time as us and everything matched yay
                            Recognition::Positive
                        }
                    };
                }
            }
        }
    }
}

struct KeywordRecognizer<'a> {
    keywords: &'a [&'a str],
}

impl<'a> KeywordRecognizer<'a> {
    pub fn new(keywords: &'a [&'a str]) -> Self {
        Self { keywords }
    }
}

impl<'a> Recognizer for KeywordRecognizer<'a> {
    fn recognize(&self, buffer: &str) -> Recognition {
        // TODO use binary search
        for keyword in self.keywords {
            if keyword.eq_ignore_ascii_case(buffer) {
                return Recognition::Positive;
            }

            if keyword
                .to_uppercase()
                .starts_with(buffer.to_uppercase().as_str())
            {
                return Recognition::Partial;
            }
        }

        Recognition::Negative
    }
}

pub fn any_single_char_recognizer() -> impl Recognizer {
    AnySingleCharRecognizer {}
}

pub fn single_new_line_recognizer() -> impl Recognizer {
    SingleNewLineRecognizer {}
}

pub fn many_digits_recognizer() -> impl Recognizer {
    ManyPredicateRecognizer {
        predicate: is_digit,
    }
}

pub fn many_white_space_recognizer() -> impl Recognizer {
    ManyPredicateRecognizer {
        predicate: |ch| ch == ' ' || ch == '\t',
    }
}

pub fn many_letters_recognizer() -> impl Recognizer {
    ManyPredicateRecognizer {
        predicate: is_letter,
    }
}

pub fn is_letter(ch: char) -> bool {
    (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z')
}

pub fn is_digit(ch: char) -> bool {
    ch >= '0' && ch <= '9'
}

pub fn leading_remaining_recognizer<T: Fn(char) -> bool, U: Fn(char) -> bool>(
    leading_predicate: T,
    remaining_predicate: U,
) -> impl Recognizer {
    LeadingRemainingPredicateRecognizer {
        leading_predicate,
        remaining_predicate,
    }
}

pub fn single_char_recognizer(needle: char) -> impl Recognizer {
    SingleCharRecognizer { needle }
}

pub fn str_recognizer(needle: &str) -> impl Recognizer + '_ {
    StrRecognizer::new(needle)
}

pub fn keyword_recognizer<'a>(keywords: &'a [&'a str]) -> impl Recognizer + 'a {
    KeywordRecognizer::new(keywords)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digits() {
        let recognizer = many_digits_recognizer();
        assert_eq!(Recognition::Positive, recognizer.recognize("123"));
        assert_eq!(Recognition::Negative, recognizer.recognize(""));
        assert_eq!(Recognition::Negative, recognizer.recognize("12a"));
    }
}
