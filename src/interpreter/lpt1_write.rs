use std::io::Write;

pub struct Lpt1Write {}

impl Write for Lpt1Write {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        unimplemented!()
    }

    fn flush(&mut self) -> std::io::Result<()> {
        unimplemented!()
    }
}
