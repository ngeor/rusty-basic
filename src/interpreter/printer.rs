pub trait Printer {
    fn print(&self, s: &str) -> std::io::Result<usize>;

    fn println(&self) -> std::io::Result<usize>;

    fn move_to_next_print_zone(&self) -> std::io::Result<usize>;
}
