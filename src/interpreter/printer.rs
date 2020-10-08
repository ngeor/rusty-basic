use std::io::Write;

pub trait Printer {
    fn print(&mut self, s: &str) -> std::io::Result<usize>;

    fn println(&mut self) -> std::io::Result<usize>;

    fn move_to_next_print_zone(&mut self) -> std::io::Result<usize>;
}

pub struct WritePrinter<T: Write> {
    writer: T,
    last_column: usize,
}

impl<T: Write> WritePrinter<T> {
    pub fn new(writer: T) -> Self {
        Self {
            writer,
            last_column: 0,
        }
    }

    pub fn into_inner(self) -> (T, usize) {
        (self.writer, self.last_column)
    }
}

impl<T: Write> Printer for WritePrinter<T> {
    fn print(&mut self, s: &str) -> std::io::Result<usize> {
        let len = s.len();
        self.last_column += len;
        // TODO what if s contains \r\n
        self.writer.write(s.as_bytes())
    }

    fn println(&mut self) -> std::io::Result<usize> {
        self.last_column = 0;
        self.writer.write("\r\n".as_bytes())
    }

    fn move_to_next_print_zone(&mut self) -> std::io::Result<usize> {
        let col = self.last_column;
        let len = 14 - col % 14;
        let s: String = (0..len).map(|_| ' ').collect();
        self.print(s.as_str())
    }
}
