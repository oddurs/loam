// loam — colour, when it is wanted.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.
//
// Only on a terminal, never with NO_COLOR set (https://no-color.org), and
// never in anything a program is meant to read.

use std::io::IsTerminal;
use std::sync::OnceLock;

fn enabled(stderr: bool) -> bool {
    static OUT: OnceLock<bool> = OnceLock::new();
    static ERR: OnceLock<bool> = OnceLock::new();
    let decide = |tty: bool| tty && std::env::var_os("NO_COLOR").is_none_or(|v| v.is_empty());
    if stderr {
        *ERR.get_or_init(|| decide(std::io::stderr().is_terminal()))
    } else {
        *OUT.get_or_init(|| decide(std::io::stdout().is_terminal()))
    }
}

fn paint(code: &str, s: &str, stderr: bool) -> String {
    if enabled(stderr) {
        format!("\x1b[{code}m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}

pub fn red(s: &str) -> String {
    paint("31", s, true)
}
pub fn yellow(s: &str) -> String {
    paint("33", s, true)
}
pub fn green(s: &str) -> String {
    paint("32", s, false)
}
pub fn dim(s: &str) -> String {
    paint("2", s, false)
}
pub fn bold(s: &str) -> String {
    paint("1", s, false)
}
