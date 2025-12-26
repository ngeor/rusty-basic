/// The position of a token within a text file, expressed in row and column.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Position {
    row: u32,
    col: u32,
}

impl Position {
    pub fn new(row: u32, col: u32) -> Self {
        debug_assert!(row > 0);
        debug_assert!(col > 0);
        Self { row, col }
    }

    pub fn inc_col(self) -> Self {
        Self::new(self.row, self.col + 1)
    }

    pub fn inc_row(self) -> Self {
        Self::new(self.row + 1, 1)
    }

    pub fn start() -> Self {
        Self::new(1, 1)
    }

    pub fn row(&self) -> u32 {
        self.row
    }

    pub fn col(&self) -> u32 {
        self.col
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::start()
    }
}
