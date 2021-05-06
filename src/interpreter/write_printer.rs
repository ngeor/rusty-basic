use crate::interpreter::io::Printer;
use std::cell::RefCell;
use std::io::Write;

pub struct WritePrinter<T: Write> {
    writer: RefCell<T>,
    last_column: RefCell<usize>,
}

impl<T: Write> WritePrinter<T> {
    pub fn new(writer: T) -> Self {
        Self {
            writer: RefCell::new(writer),
            last_column: RefCell::new(0),
        }
    }

    #[cfg(test)]
    pub fn inner(&self) -> T
    where
        T: Clone,
    {
        (*self.writer.borrow()).clone()
    }

    fn print_as_is(&self, s: &str) -> std::io::Result<usize> {
        let bytes_written = self.writer.borrow_mut().write(s.as_bytes())?;
        self.writer.borrow_mut().flush()?;
        let old_column: usize = *self.last_column.borrow();
        *self.last_column.borrow_mut() = old_column + s.len();
        Ok(bytes_written)
    }
}

impl<T: Write> Printer for WritePrinter<T> {
    fn print(&self, s: &str) -> std::io::Result<usize> {
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

    fn println(&self) -> std::io::Result<usize> {
        *self.last_column.borrow_mut() = 0;
        self.writer.borrow_mut().write("\r\n".as_bytes())
    }

    fn move_to_next_print_zone(&self) -> std::io::Result<usize> {
        let col: usize = *self.last_column.borrow();
        let len = 14 - col % 14;
        let s: String = (0..len).map(|_| ' ').collect();
        self.print(s.as_str())
    }
}
