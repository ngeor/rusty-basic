//! For text mode applications.

use crate::common::QError;
use crossterm::cursor::MoveTo;
use crossterm::style::{Color, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{Clear, ClearType};
use crossterm::Command;
use crossterm::{ErrorKind, ExecutableCommand};
use std::io::stdout;

pub trait Screen {
    fn cls(&self) -> Result<(), QError>;

    fn background_color(&self, color: i32) -> Result<(), QError>;

    fn foreground_color(&self, color: i32) -> Result<(), QError>;
}

pub struct HeadlessScreen {}

impl Screen for HeadlessScreen {
    fn cls(&self) -> Result<(), QError> {
        Ok(())
    }

    fn background_color(&self, _color: i32) -> Result<(), QError> {
        Ok(())
    }

    fn foreground_color(&self, _color: i32) -> Result<(), QError> {
        Ok(())
    }
}

/// Implements the `Screen` trait using the "crossterm" crate.
pub struct CrossTermScreen {}

impl Screen for CrossTermScreen {
    fn cls(&self) -> Result<(), QError> {
        run(Clear(ClearType::All)).and_then(|_| run(MoveTo(0, 0)))
    }

    fn background_color(&self, _color: i32) -> Result<(), QError> {
        run(SetBackgroundColor(Color::Blue))
    }

    fn foreground_color(&self, _color: i32) -> Result<(), QError> {
        run(SetForegroundColor(Color::Yellow))
    }
}

impl From<ErrorKind> for QError {
    fn from(error_kind: ErrorKind) -> Self {
        QError::InternalError(format!("{}", error_kind))
    }
}

fn run<A: std::fmt::Display>(cmd: impl Command<AnsiType = A>) -> Result<(), QError> {
    let mut stdout = stdout();
    stdout.execute(cmd).map(|_| ()).map_err(QError::from)
}
