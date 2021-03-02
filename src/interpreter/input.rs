pub trait Input {
    fn eof(&self) -> std::io::Result<bool>;

    fn input(&self) -> std::io::Result<String>;

    fn line_input(&self) -> std::io::Result<String>;
}
