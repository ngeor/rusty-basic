pub trait InputSource {
    fn input(&mut self) -> std::io::Result<String>;

    fn line_input(&mut self) -> std::io::Result<String>;
}
