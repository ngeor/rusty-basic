pub trait Input {
    fn eof(&mut self) -> std::io::Result<bool>;

    fn input(&mut self) -> std::io::Result<String>;

    fn line_input(&mut self) -> std::io::Result<String>;
}
