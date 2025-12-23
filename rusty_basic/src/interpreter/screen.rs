//! For text mode applications.

use crate::RuntimeError;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::style::{Color, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{Clear, ClearType};
use crossterm::Command;
use crossterm::ExecutableCommand;
use std::io::stdout;

pub trait Screen {
    fn cls(&self) -> Result<(), RuntimeError>;

    fn background_color(&self, color: i32) -> Result<(), RuntimeError>;

    fn foreground_color(&self, color: i32) -> Result<(), RuntimeError>;

    fn move_to(&self, row: u16, col: u16) -> Result<(), RuntimeError>;

    fn show_cursor(&self) -> Result<(), RuntimeError>;

    fn hide_cursor(&self) -> Result<(), RuntimeError>;

    fn get_view_print(&self) -> Option<(usize, usize)>;

    fn set_view_print(&mut self, start_row: usize, end_row: usize);

    fn reset_view_print(&mut self);
}

#[cfg(test)]
pub struct HeadlessScreen {}

#[cfg(test)]
impl Screen for HeadlessScreen {
    fn cls(&self) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn background_color(&self, _color: i32) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn foreground_color(&self, _color: i32) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn move_to(&self, _row: u16, _col: u16) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn show_cursor(&self) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn hide_cursor(&self) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn get_view_print(&self) -> Option<(usize, usize)> {
        None
    }

    fn set_view_print(&mut self, _start_row: usize, _end_row: usize) {}

    fn reset_view_print(&mut self) {}
}

/// Implements the `Screen` trait using the "crossterm" crate.
#[derive(Default)]
pub struct CrossTermScreen {
    view_print: Option<(usize, usize)>,
}

impl Screen for CrossTermScreen {
    fn cls(&self) -> Result<(), RuntimeError> {
        run(Clear(ClearType::All)).and_then(|_| run(MoveTo(0, 0)))
    }

    fn background_color(&self, color: i32) -> Result<(), RuntimeError> {
        run(SetBackgroundColor(qbcolor_to_crossterm_color(color)?))
    }

    fn foreground_color(&self, color: i32) -> Result<(), RuntimeError> {
        run(SetForegroundColor(qbcolor_to_crossterm_color(color)?))
    }

    fn move_to(&self, row: u16, col: u16) -> Result<(), RuntimeError> {
        run(MoveTo(col, row))
    }

    fn show_cursor(&self) -> Result<(), RuntimeError> {
        run(Show)
    }

    fn hide_cursor(&self) -> Result<(), RuntimeError> {
        run(Hide)
    }

    fn get_view_print(&self) -> Option<(usize, usize)> {
        self.view_print
    }

    fn set_view_print(&mut self, start_row: usize, end_row: usize) {
        self.view_print = Some((start_row, end_row));
    }

    fn reset_view_print(&mut self) {
        self.view_print = None;
    }
}

fn run(cmd: impl Command) -> Result<(), RuntimeError> {
    let mut stdout = stdout();
    stdout.execute(cmd).map(|_| ()).map_err(RuntimeError::from)
}

fn qbcolor_to_crossterm_color(color: i32) -> Result<Color, RuntimeError> {
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
        Err(RuntimeError::IllegalFunctionCall)
    }
}
