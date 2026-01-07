mod row_col_view;
mod string_view;

use std::fs::File;

pub use self::string_view::*;

pub fn create_file_tokenizer(input: File) -> Result<RcStringView, std::io::Error> {
    let rc_string_view: RcStringView = input.try_into()?;
    Ok(rc_string_view)
}

pub fn create_string_tokenizer(input: String) -> RcStringView {
    input.into()
}
