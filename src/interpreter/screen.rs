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

    fn background_color(&self, color: i32) -> Result<(), QError> {
        run(SetBackgroundColor(qbcolor_to_crossterm_color(color)?))
    }

    fn foreground_color(&self, color: i32) -> Result<(), QError> {
        run(SetForegroundColor(qbcolor_to_crossterm_color(color)?))
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

fn qbcolor_to_crossterm_color(color: i32) -> Result<Color, QError> {
    let colors = [
        Color::Black,
        Color::DarkBlue,
        Color::DarkGreen,
        Color::DarkCyan,
        Color::DarkRed,
        Color::DarkMagenta,
        Color::DarkYellow,
        Color::Grey,
        Color::DarkGrey,
        Color::Blue,
        Color::Green,
        Color::Cyan,
        Color::Red,
        Color::Magenta,
        Color::Yellow,
        Color::White,
    ];
    if color >= 0 && color < colors.len() as i32 {
        Ok(colors[color as usize])
    } else {
        Err(QError::IllegalFunctionCall)
    }
}
