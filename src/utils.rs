use std::{env, io::Write, path::PathBuf, usize::MAX};

use anyhow::{Error, Result};
// Copyright © 2016 Felix Obenhuber
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
use termcolor::{Buffer, BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};
use which::which_in;

use crate::record::{Level, LogcatRecord};
pub fn adb() -> Result<PathBuf> {
    which_in("adb", env::var_os("PATH"), env::current_dir()?).map_err(Into::into)
}

pub fn terminal_width() -> Option<usize> {
    match term_size::dimensions() {
        Some((width, _)) => Some(width),
        None => env::var("COLUMNS")
            .ok()
            .and_then(|e| e.parse::<usize>().ok()),
    }
}
#[cfg(target_os = "windows")]
fn hashed_color(i: &str) -> Color {
    let v = i.bytes().fold(42u8, |c, x| c ^ x) % 7;
    match v {
        0 => Color::Blue,
        1 => Color::Green,
        2 => Color::Red,
        3 => Color::Cyan,
        4 => Color::Magenta,
        5 => Color::Yellow,
        _ => Color::White,
    }
}

#[cfg(not(target_os = "windows"))]
fn hashed_color(i: &str) -> Color {
    // Some colors are hard to read on (at least) dark terminals
    // and I consider some others as ugly.
    Color::Ansi256(match i.bytes().fold(42u8, |c, x| c ^ x) {
        c @ 0..=1 => c + 2,
        c @ 16..=21 => c + 6,
        c @ 52..=55 | c @ 126..=129 => c + 4,
        c @ 163..=165 | c @ 200..=201 => c + 3,
        c @ 207 => c + 1,
        c @ 232..=240 => c + 9,
        c => c,
    })
}

pub struct Terminal {
    pub width: usize,
    pub buffer: BufferWriter,
    pub tag_width: usize,
    pub process_name_width: usize,
    pub hide_timestamp: bool,
    pub hide_date: bool,
    pub use_process_name: bool,
    pub bright_colors: bool,
    pub pid_width: usize,
}
impl Default for Terminal {
    fn default() -> Self {
        let width = terminal_width().unwrap_or(80);
        let buffer = BufferWriter::stdout(ColorChoice::Auto);
        let tag_width = 30;
        let process_name_width = 20;
        let pid_width = 10;
        Self {
            width,
            buffer,
            tag_width,
            pid_width,
            process_name_width,
            hide_timestamp: false,
            hide_date: true,
            use_process_name: true,
            bright_colors: true,
        }
    }
}
impl Drop for Terminal {
    fn drop(&mut self) {
        let mut buffer = self.buffer.buffer();
        buffer.reset().and_then(|_| self.buffer.print(&buffer)).ok();
    }
}
impl Terminal {
    pub fn print(&mut self, record: &LogcatRecord) -> Result<()> {
        let datetime = {
            if self.hide_timestamp {
                String::new()
            } else if self.hide_date {
                record
                    .timestamp
                    .map(|t| t.format("%H:%M:%S").to_string())
                    .unwrap_or(" ".repeat(12))
            } else {
                record
                    .timestamp
                    .map(|t| t.format("%m-%d %H:%M:%S").to_string())
                    .unwrap_or(" ".repeat(17))
            }
        };
        let tag_chars = record.tag.chars().count();
        let tag = if self.hide_timestamp {
            format!(
                "{:<width$}",
                record
                    .tag
                    .chars()
                    .take(std::cmp::min(self.tag_width, tag_chars))
                    .collect::<String>(),
                width = self.tag_width
            )
        } else {
            format!(
                "{:>width$}",
                record
                    .tag
                    .chars()
                    .take(std::cmp::min(self.tag_width, tag_chars))
                    .collect::<String>(),
                width = self.tag_width
            )
        };
        let process_name = if self.use_process_name {
            let chars = record.process_name.chars().count();
            format!(
                "{:>width$}",
                record
                    .process_name
                    .chars()
                    .take(std::cmp::min(self.process_name_width, chars))
                    .collect::<String>(),
                width = self.process_name_width
            )
        } else {
            format!(
                "{:>width$}",
                format!("{}:{}", record.pid, record.pid),
                width = self.pid_width
            )
        };
        let preamble_width = datetime.chars().count()
            + 1 //Space
            + tag.chars().count()
            + 2 // " ["
            + process_name.chars().count()
            + 2 // "] "
            + 3; //" D "
        let timestamp_color = None;
        let tag_color = hashed_color(&record.tag);
        let pid_color = hashed_color(&process_name);
        let level_color = match record.level {
            Level::Info => Some(Color::Green),
            Level::Warn => Some(Color::Yellow),
            Level::Error | Level::Fatal | Level::Assert => Some(Color::Red),
            _ => None,
        };
        let write_preamble = |buffer: &mut Buffer| -> Result<(), Error> {
            let mut spec = ColorSpec::new();
            buffer.set_color(spec.set_fg(timestamp_color))?;
            buffer.write_all(datetime.as_bytes())?;
            buffer.write_all(b" ")?;

            buffer.set_color(spec.set_fg(Some(tag_color)))?;
            buffer.write_all(tag.as_bytes())?;
            buffer.set_color(spec.set_fg(None))?;

            buffer.write_all(b" [")?;
            buffer.set_color(spec.set_fg(Some(pid_color)))?;
            buffer.write_all(process_name.as_bytes())?;
            buffer.set_color(spec.set_fg(None))?;
            buffer.write_all(b"] ")?;

            buffer.set_color(
                spec.set_bg(level_color)
                    .set_fg(level_color.map(|_| Color::Black)), // Set fg only if bg is set
            )?;
            write!(buffer, " {} ", record.level)?;
            buffer.set_color(&ColorSpec::new())?;

            Ok(())
        };
        let payload_len = terminal_width().unwrap_or(MAX) - preamble_width - 3;
        let message = record.message.replace('\t', "");
        let message_len = message.chars().count();
        let chunks = message_len / payload_len + 1;
        {
            let mut buffer = self.buffer.buffer();
            for i in 0..chunks {
                write_preamble(&mut buffer)?;

                let c = if chunks == 1 {
                    "   "
                } else if i == 0 {
                    " ┌ "
                } else if i == chunks - 1 {
                    " └ "
                } else {
                    " ├ "
                };

                buffer.write_all(c.as_bytes())?;

                let chunk = message
                    .chars()
                    .skip(i * payload_len)
                    .take(payload_len)
                    .collect::<String>();
                buffer.set_color(
                    ColorSpec::new()
                        .set_intense(self.bright_colors)
                        .set_fg(level_color),
                )?;
                buffer.write_all(chunk.as_bytes())?;
                buffer.write_all(b"\n")?;
            }
            self.buffer.print(&buffer).map_err(Into::into)
        }
    }
}
