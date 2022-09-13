#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Recognition {
    Negative,
    Partial,
    Positive
}

pub trait Recognizer {
    fn recognize(&self, buffer: &str) -> Recognition;
}

struct AnyRecognizer {}

impl Recognizer for AnyRecognizer {
    fn recognize(&self, buffer: &str) -> Recognition {
        if buffer.len() == 1 {
            Recognition::Positive
        } else {
            Recognition::Negative
        }
    }
}

struct NewLineRecognizer {}

impl Recognizer for NewLineRecognizer {
    fn recognize(&self, buffer: &str) -> Recognition {
        if buffer == "r" || buffer == "\n" || buffer == "\r\n" {
            Recognition::Positive
        } else {
            Recognition::Negative
        }
    }
}

struct PredicateRecognizer<T: Fn(char) -> bool> {
    predicate: T
}

impl<T: Fn(char) -> bool> Recognizer for PredicateRecognizer<T> {
    fn recognize(&self, buffer: &str) -> Recognition {
        if !buffer.is_empty() && buffer.chars().all(&self.predicate) {
            Recognition::Positive
        } else {
            Recognition::Negative
        }
    }
}

pub fn digits_recognizer() -> impl Recognizer {
    PredicateRecognizer {
        predicate: |ch| ch >= '0' && ch <= '9'
    }
}

pub fn letters_recognizer() -> impl Recognizer {
    PredicateRecognizer {
        predicate: |ch| (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digits() {
        let recognizer = digits_recognizer();
        assert_eq!(Recognition::Positive, recognizer.recognize("123"));
        assert_eq!(Recognition::Negative, recognizer.recognize(""));
        assert_eq!(Recognition::Negative, recognizer.recognize("12a"));
    }
}
