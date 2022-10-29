use crate::interpreter::io::Printer;
use std::io::Write;

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

    #[cfg(test)]
    pub fn inner(&self) -> &T {
        &self.writer
    }

    fn print_as_is(&mut self, s: &str) -> std::io::Result<usize> {
        let bytes_written = self.writer.write(s.as_bytes())?;
        self.writer.flush()?;
        self.last_column += s.len();
        Ok(bytes_written)
    }
}

impl<T: Write> Printer for WritePrinter<T> {
    fn print(&mut self, s: &str) -> std::io::Result<usize> {
        // This isn't a bug: it seems QBasic does not split on CRLF,
        // but separately on CR and LF for this particular case.
        // Might be a bug on QBasic side arguably.
        let split = s.split(|ch| ch == '\r' || ch == '\n');
        let mut is_first = true;
        let mut bytes_written: usize = 0;
        for part in split {
            if is_first {
                is_first = false;
            } else {
                bytes_written += self.println()?;
            }
            bytes_written += self.print_as_is(part)?;
        }
        Ok(bytes_written)
    }

    fn println(&mut self) -> std::io::Result<usize> {
        self.last_column = 0;
        self.writer.write("\r\n".as_bytes())
    }

    fn move_to_next_print_zone(&mut self) -> std::io::Result<usize> {
        let col: usize = self.last_column;
        let len = 14 - col % 14;
        let s: String = (0..len).map(|_| ' ').collect();
        self.print(s.as_str())
    }
}
