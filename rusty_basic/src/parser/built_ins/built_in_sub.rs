#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuiltInSub {
    Beep,
    CallAbsolute,
    Close,
    Cls,
    Color,
    Data,
    DefSeg,
    Environ,
    Field,
    Get,

    /// `INPUT [;] ["prompt"{; | ,}] variable-list`
    ///
    /// `INPUT #file-number%, variable-list`
    ///
    /// prompt - An optional string that is displayed before the user
    /// enters data. A semicolon after prompt appends a question mark to the
    /// prompt string.
    ///
    /// variable names can consist of up to 40 characters and must begin
    /// with a letter. Valid characters are a-z, 0-9 and period (.).
    ///
    /// A semicolon immediately after INPUT keeps the cursor on the same line
    /// after the user presses the Enter key.
    ///
    Input,

    /// `KILL file-spec$` -> deletes files from disk
    Kill,

    /// `LINE INPUT` -> see [INPUT](Self::Input)
    ///
    /// `LINE INPUT [;] ["prompt";] variable$`
    ///
    /// `LINE INPUT #file-number%, variable$`
    LineInput,

    /// LOCATE moves the cursor to a specified position on the screen
    ///
    /// `LOCATE [row%] [,[column%] [,[cursor%] [,start% [,stop%]]]]`
    ///
    /// cursor 0 = invisible 1 = visible
    ///
    /// start and stop are 0..31 indicating scan lines
    Locate,
    LSet,

    /// `NAME old$ AS new$` Renames a file or directory.
    Name,

    /// `OPEN file$ [FOR mode] [ACCESS access] [lock] AS [#]file-number% [LEN=rec-len%]`
    ///
    /// mode: APPEND, BINARY, INPUT, OUTPUT, RANDOM
    ///
    /// access: READ, WRITE, READ WRITE
    ///
    /// lock: SHARED, LOCK READ, LOCK WRITE, LOCK READ WRITE
    ///
    /// file-number a number in the range 1 through 255
    ///
    /// rec-len%: For random access files, the record length (default is 128 bytes)
    ///           For sequential files, the number of characters buffered (default is 512 bytes)
    Open,

    /// `POKE`
    Poke,

    Put,
    Read,
    Screen,
    ViewPrint,
    Width,
}

impl BuiltInSub {
    /// Parses a built-in sub name which isn't implemented with a keyword.
    /// This sub would appear as a user defined SUB on the parser layer.
    ///
    /// Some statements are implemented a built-in subs (e.g. `CLOSE`, `OPEN`), but
    /// they can't hit this function, as they are represented by keywords and are
    /// parsed by custom parsers.
    pub fn parse_non_keyword_sub(s: &str) -> Option<BuiltInSub> {
        if s.eq_ignore_ascii_case("Beep") {
            Some(BuiltInSub::Beep)
        } else if s.eq_ignore_ascii_case("Call") {
            Some(BuiltInSub::CallAbsolute)
        } else if s.eq_ignore_ascii_case("Cls") {
            Some(BuiltInSub::Cls)
        } else if s.eq_ignore_ascii_case("Color") {
            Some(BuiltInSub::Color)
        } else if s.eq_ignore_ascii_case("Environ") {
            Some(BuiltInSub::Environ)
        } else if s.eq_ignore_ascii_case("Kill") {
            Some(BuiltInSub::Kill)
        } else if s.eq_ignore_ascii_case("Poke") {
            Some(BuiltInSub::Poke)
        } else if s.eq_ignore_ascii_case("Screen") {
            Some(BuiltInSub::Screen)
        } else {
            None
        }
    }
}
