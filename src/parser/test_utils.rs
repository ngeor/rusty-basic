use super::*;

pub fn parse<T>(input: T) -> Program
where
    T: AsRef<[u8]>,
{
    let mut parser = Parser::from(input);
    parser.parse().expect("Could not parse program")
}

pub fn parse_file<S: AsRef<str>>(filename: S) -> Program {
    let file_path = format!("fixtures/{}", filename.as_ref());
    let mut parser = Parser::from(File::open(file_path).expect("Could not read bas file"));
    parser.parse().expect("Could not parse program")
}
