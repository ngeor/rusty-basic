/// The position of a token within a text file, expressed in row and column.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Position {
    row: u32,
    col: u32,
}

impl Position {
    pub fn new(row: u32, col: u32) -> Self {
        Self { row, col }
    }

    pub fn inc_col(&mut self) {
        self.col += 1
    }

    pub fn inc_row(&mut self) {
        self.row += 1;
        self.col = 1;
    }

    pub fn start() -> Self {
        Self::new(1, 1)
    }

    pub fn zero() -> Self {
        Self::new(0, 0)
    }

    pub fn row(&self) -> u32 {
        self.row
    }

    pub fn col(&self) -> u32 {
        self.col
    }
}
