#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RowCol {
    row: u32,
    col: u32,
}

pub struct RowColView {
    /// Holds the row - col per character of the string.
    data: Vec<RowCol>,
}

pub fn create_row_col_view(s: &str) -> RowColView {
    let chars: Vec<char> = s.chars().collect();
    let mut data = Vec::new();
    let mut row = 1;
    let mut col = 1;
    let mut i = 0;
    while i < chars.len() {
        data.push(RowCol { row, col });

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

    RowColView { data }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_string() {
        let view = create_row_col_view("");
        assert_eq!(view.data.len(), 0);
    }

    #[test]
    fn test_without_new_lines() {
        let view = create_row_col_view("hello");
        assert_eq!(view.data.len(), 5);
        assert_eq!(view.data[0], RowCol { row: 1, col: 1 });
        assert_eq!(view.data[1], RowCol { row: 1, col: 2 });
        assert_eq!(view.data[2], RowCol { row: 1, col: 3 });
        assert_eq!(view.data[3], RowCol { row: 1, col: 4 });
        assert_eq!(view.data[4], RowCol { row: 1, col: 5 });
    }

    #[test]
    fn test_with_new_line() {
        let view = create_row_col_view("hello\nworld");
        assert_eq!(view.data.len(), 11);
        assert_eq!(view.data[0], RowCol { row: 1, col: 1 });
        assert_eq!(view.data[1], RowCol { row: 1, col: 2 });
        assert_eq!(view.data[2], RowCol { row: 1, col: 3 });
        assert_eq!(view.data[3], RowCol { row: 1, col: 4 });
        assert_eq!(view.data[4], RowCol { row: 1, col: 5 });
        assert_eq!(view.data[5], RowCol { row: 1, col: 6 });
        assert_eq!(view.data[6], RowCol { row: 2, col: 1 });
        assert_eq!(view.data[7], RowCol { row: 2, col: 2 });
        assert_eq!(view.data[8], RowCol { row: 2, col: 3 });
        assert_eq!(view.data[9], RowCol { row: 2, col: 4 });
        assert_eq!(view.data[10], RowCol { row: 2, col: 5 });
    }

    #[test]
    fn test_with_cr_lf() {
        let view = create_row_col_view("abc\r\ndef");
        assert_eq!(view.data.len(), 8);
        assert_eq!(view.data[0], RowCol { row: 1, col: 1 });
        assert_eq!(view.data[1], RowCol { row: 1, col: 2 });
        assert_eq!(view.data[2], RowCol { row: 1, col: 3 });
        assert_eq!(view.data[3], RowCol { row: 1, col: 4 });
        assert_eq!(view.data[4], RowCol { row: 1, col: 4 });
        assert_eq!(view.data[5], RowCol { row: 2, col: 1 });
        assert_eq!(view.data[6], RowCol { row: 2, col: 2 });
        assert_eq!(view.data[7], RowCol { row: 2, col: 3 });
    }

    #[test]
    fn test_with_cr() {
        let view = create_row_col_view("abc\rdef");
        assert_eq!(view.data.len(), 7);
        assert_eq!(view.data[0], RowCol { row: 1, col: 1 });
        assert_eq!(view.data[1], RowCol { row: 1, col: 2 });
        assert_eq!(view.data[2], RowCol { row: 1, col: 3 });
        assert_eq!(view.data[3], RowCol { row: 1, col: 4 });
        assert_eq!(view.data[4], RowCol { row: 2, col: 1 });
        assert_eq!(view.data[5], RowCol { row: 2, col: 2 });
        assert_eq!(view.data[6], RowCol { row: 2, col: 3 });
    }
}
