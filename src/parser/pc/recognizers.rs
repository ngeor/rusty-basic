#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Recognition {
    Negative,
    Partial,
    Positive,
}

pub trait Recognizer {
    fn recognize(&self, buffer: &str) -> Recognition;
}

// blanket implementation for functions, so that any `fn(&str) -> Recognition`
// automatically implements the [Recognizer] trait
impl<F> Recognizer for F
where
    F: Fn(&str) -> Recognition,
{
    fn recognize(&self, buffer: &str) -> Recognition {
        (self)(buffer)
    }
}

/// A recognizer that matches a single character. It can be used as a fallback
/// mechanism when nothing else matches.
pub fn any_single_char_recognizer(buffer: &str) -> Recognition {
    if buffer.chars().count() == 1 {
        Recognition::Positive
    } else {
        Recognition::Negative
    }
}

/// A recognizer that matches a single new line.
pub fn single_new_line_recognizer(buffer: &str) -> Recognition {
    if buffer == "\r" || buffer == "\n" || buffer == "\r\n" {
        Recognition::Positive
    } else {
        Recognition::Negative
    }
}

/// A recognizer that matches any non-empty string where all characters meet
/// the given predicate.
pub fn all_chars_are<F>(predicate: F) -> impl Recognizer
where
    F: Fn(char) -> bool,
{
    move |buffer: &str| {
        if !buffer.is_empty() && buffer.chars().all(&predicate) {
            Recognition::Positive
        } else {
            Recognition::Negative
        }
    }
}

/// A recognizer that matches a string that starts with a character satisfying
/// the [leading_predicate] and continues with characters that satisfy
/// the [remaining_predicate].
pub fn leading_remaining_recognizer<T: Fn(char) -> bool, U: Fn(char) -> bool>(
    leading_predicate: T,
    remaining_predicate: U,
) -> impl Recognizer {
    move |buffer: &str| {
        let mut idx: usize = 0;
        for ch in buffer.chars() {
            let matches = if idx == 0 {
                (leading_predicate)(ch)
            } else {
                (remaining_predicate)(ch)
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

// A recognizer that matches a specific character.
impl Recognizer for char {
    fn recognize(&self, buffer: &str) -> Recognition {
        if buffer.len() == 1 && buffer.chars().all(|c| c == *self) {
            Recognition::Positive
        } else {
            Recognition::Negative
        }
    }
}

// A recognizer that matches a specific string.
impl<'a> Recognizer for &'a str {
    fn recognize(&self, buffer: &str) -> Recognition {
        let mut needle_iter = self.chars();
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

/// Recognizes keywords from the given list (case insensitive).
pub fn keyword_recognizer<'a>(keywords: &'a [&'a str]) -> impl Recognizer + 'a {
    move |buffer: &str| {
        // TODO use binary search
        for keyword in keywords {
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

pub fn many_digits_recognizer() -> impl Recognizer {
    all_chars_are(is_digit)
}

pub fn many_white_space_recognizer() -> impl Recognizer {
    all_chars_are(|ch| ch == ' ' || ch == '\t')
}

pub fn many_letters_recognizer() -> impl Recognizer {
    all_chars_are(is_letter)
}

pub fn is_letter(ch: char) -> bool {
    ('a'..='z').contains(&ch) || ('A'..='Z').contains(&ch)
}

pub fn is_digit(ch: char) -> bool {
    ('0'..='9').contains(&ch)
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

// TODO error codes e.g. QB001
// TODO break down the project to libraries (?) to improve module encapsulation
