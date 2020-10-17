pub trait Printer {
    fn print(&mut self, s: &str) -> std::io::Result<usize>;

    fn println(&mut self) -> std::io::Result<usize>;

    fn move_to_next_print_zone(&mut self) -> std::io::Result<usize>;
}
