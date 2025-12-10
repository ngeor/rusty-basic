use rusty_common::Position;

pub fn create_row_col_view(chars: &Vec<char>) -> Vec<Position> {
    let mut data = Vec::new();
    let mut row = 1;
    let mut col = 1;
    let mut i = 0;
    while i < chars.len() {
        data.push(Position::new(row, col));

        if chars[i] == '\r' {
            if i < chars.len() - 1 && chars[i + 1] == '\n' {
                // do not increment
            } else {
                row += 1;
                col = 1;
            }
        } else if chars[i] == '\n' {
            row += 1;
            col = 1;
        } else {
            col += 1;
        }

        i += 1;
    }

    data
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_row_col_view_from_str(s: &str) -> Vec<Position> {
        let chars: Vec<char> = s.chars().collect();
        create_row_col_view(&chars)
    }

    #[test]
    fn test_empty_string() {
        let view = create_row_col_view_from_str("");
        assert_eq!(view.len(), 0);
    }

    #[test]
    fn test_without_new_lines() {
        let view = create_row_col_view_from_str("hello");
        assert_eq!(view.len(), 5);
        assert_eq!(view[0], Position::new(1, 1));
        assert_eq!(view[1], Position::new(1, 2));
        assert_eq!(view[2], Position::new(1, 3));
        assert_eq!(view[3], Position::new(1, 4));
        assert_eq!(view[4], Position::new(1, 5));
    }

    #[test]
    fn test_with_new_line() {
        let view = create_row_col_view_from_str("hello\nworld");
        assert_eq!(view.len(), 11);
        assert_eq!(view[0], Position::new(1, 1));
        assert_eq!(view[1], Position::new(1, 2));
        assert_eq!(view[2], Position::new(1, 3));
        assert_eq!(view[3], Position::new(1, 4));
        assert_eq!(view[4], Position::new(1, 5));
        assert_eq!(view[5], Position::new(1, 6));
        assert_eq!(view[6], Position::new(2, 1));
        assert_eq!(view[7], Position::new(2, 2));
        assert_eq!(view[8], Position::new(2, 3));
        assert_eq!(view[9], Position::new(2, 4));
        assert_eq!(view[10], Position::new(2, 5));
    }

    #[test]
    fn test_with_cr_lf() {
        let view = create_row_col_view_from_str("abc\r\ndef");
        assert_eq!(view.len(), 8);
        assert_eq!(view[0], Position::new(1, 1));
        assert_eq!(view[1], Position::new(1, 2));
        assert_eq!(view[2], Position::new(1, 3));
        assert_eq!(view[3], Position::new(1, 4));
        assert_eq!(view[4], Position::new(1, 4));
        assert_eq!(view[5], Position::new(2, 1));
        assert_eq!(view[6], Position::new(2, 2));
        assert_eq!(view[7], Position::new(2, 3));
    }

    #[test]
    fn test_with_cr() {
        let view = create_row_col_view_from_str("abc\rdef");
        assert_eq!(view.len(), 7);
        assert_eq!(view[0], Position::new(1, 1));
        assert_eq!(view[1], Position::new(1, 2));
        assert_eq!(view[2], Position::new(1, 3));
        assert_eq!(view[3], Position::new(1, 4));
        assert_eq!(view[4], Position::new(2, 1));
        assert_eq!(view[5], Position::new(2, 2));
        assert_eq!(view[6], Position::new(2, 3));
    }
}
