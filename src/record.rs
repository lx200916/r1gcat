use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

// const LEVEL_VALUES: &[&str] = &[
//     "trace", "debug", "info", "warn", "error", "fatal", "assert", "T", "D", "I", "W", "E", "F", "A",
// ];
#[derive(Clone, Debug, Deserialize, PartialOrd, PartialEq, Serialize, Default)]
pub enum Level {
    #[default]
    None,
    Trace,
    Verbose,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
    Assert,
}
impl Display for Level {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Level::None => "-",
                Level::Trace => "T",
                Level::Verbose => "V",
                Level::Debug => "D",
                Level::Info => "I",
                Level::Warn => "W",
                Level::Error => "E",
                Level::Fatal => "F",
                Level::Assert => "A",
            }
        )
    }
}

impl<'a> From<&'a str> for Level {
    fn from(s: &str) -> Self {
        match s {
            "T" | "trace" => Level::Trace,
            "V" | "verbose" => Level::Verbose,
            "D" | "debug" => Level::Debug,
            "I" | "info" => Level::Info,
            "W" | "warn" => Level::Warn,
            "E" | "error" => Level::Error,
            "F" | "fatal" => Level::Fatal,
            "A" | "assert" => Level::Assert,
            _ => Level::None,
        }
    }
}

// impl Level {
//     pub fn values() -> &'static [&'static str] {
//         LEVEL_VALUES
//     }
// }
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LogcatRecord {
    pub level: Level,
    pub tag: String,
    pub message: String,
    pub pid: u32,
    pub tid: u32,
    pub raw: String,
    pub process_name: String,
    pub timestamp: Option<DateTime<Local>>,
}
impl Display for LogcatRecord {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(
            f,
            "{timestamp} {pid:>5} {tid:>10} {level:>5} {tag:>20}: {message}",
            timestamp = self
                .timestamp
                .map(|t| t.format("%m-%d %H:%M:%S%.3f").to_string())
                .unwrap_or_default(),
            pid = self.pid,
            tid = self.process_name,
            level = self.level,
            tag = self.tag,
            message = self.message,
        )
    }
}
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct ProcessRecord {
    pub user: String,
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
    pub rss: u32,
    pub pc: String,
}
